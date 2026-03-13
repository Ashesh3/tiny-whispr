use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

use crate::audio::AudioRecorder;
use crate::db::{Database, Transcription};
use crate::hotkey::{register_from_settings, HotkeyManager};
use crate::settings::{
    load_settings, mask_api_key, merge_settings, save_settings_to_file, Settings,
};
use crate::tray;

/// A version of Settings where the API key is always masked for safe frontend display.
#[derive(Debug, Clone, Serialize)]
pub struct MaskedSettings {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub language: String,
    pub hotkey: String,
    pub activation_mode: String,
    pub output_mode: String,
    pub launch_at_startup: bool,
}

impl From<&Settings> for MaskedSettings {
    fn from(s: &Settings) -> Self {
        Self {
            provider: s.provider.clone(),
            api_key: mask_api_key(&s.api_key),
            base_url: s.base_url.clone(),
            model: s.model.clone(),
            language: s.language.clone(),
            hotkey: s.hotkey.clone(),
            activation_mode: s.activation_mode.clone(),
            output_mode: s.output_mode.clone(),
            launch_at_startup: s.launch_at_startup,
        }
    }
}

#[tauri::command]
pub fn get_settings() -> Result<MaskedSettings, String> {
    let settings = load_settings();
    Ok(MaskedSettings::from(&settings))
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    settings: Settings,
    hotkey_mgr: State<'_, HotkeyManager>,
) -> Result<MaskedSettings, String> {
    let old_settings = load_settings();
    let saved = merge_settings(settings, &old_settings)?;
    let hotkey_changed = saved.hotkey != old_settings.hotkey
        || saved.activation_mode != old_settings.activation_mode;

    if hotkey_changed {
        register_from_settings(&app, &hotkey_mgr, &saved).inspect_err(|_| {
            let _ = register_from_settings(&app, &hotkey_mgr, &old_settings);
        })?;
    }

    if let Err(e) = save_settings_to_file(&saved) {
        if hotkey_changed {
            let _ = register_from_settings(&app, &hotkey_mgr, &old_settings);
        }
        return Err(e);
    }

    // Toggle autostart if it changed
    if saved.launch_at_startup != old_settings.launch_at_startup {
        if saved.launch_at_startup {
            let _ = app.autolaunch().enable();
        } else {
            let _ = app.autolaunch().disable();
        }
    }

    Ok(MaskedSettings::from(&saved))
}

#[tauri::command]
pub fn get_history(db: State<'_, Database>) -> Result<Vec<Transcription>, String> {
    db.get_all()
}

#[tauri::command]
pub fn delete_transcription(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete(id)
}

#[tauri::command]
pub fn clear_history(db: State<'_, Database>) -> Result<(), String> {
    db.clear_all()
}

#[tauri::command]
pub fn search_history(
    db: State<'_, Database>,
    query: String,
) -> Result<Vec<Transcription>, String> {
    db.search(&query)
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    crate::output::copy_to_clipboard(&text)
}

#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    recorder: State<'_, AudioRecorder>,
) -> Result<(), String> {
    // Check if API key is set
    let settings = load_settings();
    if settings.api_key.is_empty() {
        let _ = app
            .notification()
            .builder()
            .title("TinyWhispr")
            .body("Set an API key in Settings first")
            .show();
        return Err("No API key configured".to_string());
    }

    recorder.start_recording()?;

    play_sound("Speech On");

    // Update tray state
    let _ = tray::set_tray_state(&app, tray::TrayState::Recording);

    let _ = app.emit("recording-started", ());
    spawn_recording_timeout_monitor(app.clone());

    Ok(())
}

/// Plays a Windows system sound by name from C:\Windows\Media\{name}.wav
fn play_sound(name: &'static str) {
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::ffi::OsStrExt;
            use std::ffi::OsStr;

            extern "system" {
                fn PlaySoundW(psz_sound: *const u16, hmod: *const core::ffi::c_void, fdw_sound: u32) -> i32;
            }

            const SND_FILENAME: u32 = 0x00020000;
            const SND_ASYNC: u32 = 0x0001;

            let path = format!("C:\\Windows\\Media\\{name}.wav");
            let wide: Vec<u16> = OsStr::new(&path).encode_wide().chain(Some(0)).collect();

            unsafe {
                PlaySoundW(wide.as_ptr(), std::ptr::null(), SND_FILENAME | SND_ASYNC);
            }
        }
    });
}

#[tauri::command]
pub async fn stop_recording(app: AppHandle) -> Result<(), String> {
    finalize_recording(app).await
}

async fn finalize_recording(app: AppHandle) -> Result<(), String> {
    let duration_ms = {
        let recorder = app.state::<AudioRecorder>();
        recorder.duration_ms() as i64
    };

    let wav_data = {
        let recorder = app.state::<AudioRecorder>();
        recorder.stop_recording()?
    };

    play_sound("Speech Off");

    // Update tray to processing state
    let _ = tray::set_tray_state(&app, tray::TrayState::Processing);

    let _ = app.emit("recording-stopped", ());

    // Load settings for transcription
    let settings = load_settings();

    // Transcribe
    match crate::transcribe::transcribe(
        &settings.base_url,
        &settings.api_key,
        &settings.model,
        &settings.language,
        wav_data,
    )
    .await
    {
        Ok(result) => {
            // Save to database
            let db = app.state::<Database>();
            let _ = db.insert(
                &result.text,
                &settings.provider,
                &settings.model,
                if settings.language == "auto" {
                    None
                } else {
                    Some(settings.language.as_str())
                },
                Some(duration_ms),
            );

            // Output text (auto-paste or clipboard with notification)
            let _ =
                crate::output::output_text_with_notify(&app, &result.text, &settings.output_mode);

            // Update tray back to idle
            let _ = tray::set_tray_state(&app, tray::TrayState::Idle);

            let _ = app.emit("transcription-complete", ());
        }
        Err(e) => {
            // Update tray back to idle
            let _ = tray::set_tray_state(&app, tray::TrayState::Idle);

            let error_msg = e.to_string();
            let _ = app.emit("transcription-error", &error_msg);

            let _ = app
                .notification()
                .builder()
                .title("TinyWhispr")
                .body(&error_msg)
                .show();

            return Err(error_msg);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_recording_state(recorder: State<'_, AudioRecorder>) -> String {
    if recorder.is_recording() {
        "recording".to_string()
    } else {
        "idle".to_string()
    }
}

fn spawn_recording_timeout_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;

            let limit_reached = {
                let recorder = app.state::<AudioRecorder>();
                recorder.limit_reached()
            };

            if limit_reached {
                let _ = app
                    .notification()
                    .builder()
                    .title("TinyWhispr")
                    .body("Maximum recording duration reached (5 min)")
                    .show();

                let _ = finalize_recording(app.clone()).await;
                break;
            }

            let still_recording = {
                let recorder = app.state::<AudioRecorder>();
                recorder.is_recording()
            };

            if !still_recording {
                break;
            }
        }
    });
}
