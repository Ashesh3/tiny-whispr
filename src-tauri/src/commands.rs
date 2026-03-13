use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

use crate::audio::AudioRecorder;
use crate::db::{Database, Transcription};
use crate::hotkey::HotkeyManager;
use crate::settings::{load_settings, mask_api_key, save_settings_merged, Settings};
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
    let saved = save_settings_merged(settings)?;

    // Re-register hotkey if it changed
    if saved.hotkey != old_settings.hotkey {
        let _ = hotkey_mgr.register(&app, &saved.hotkey, |app_handle| {
            let _ = app_handle.emit("tray-toggle-recording", ());
        });
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

    // Play Windows start sound
    play_system_sound();

    // Show brief overlay toast
    show_recording_overlay(&app);

    // Update tray state
    let _ = tray::set_tray_state(&app, tray::TrayState::Recording);

    let _ = app.emit("recording-started", ());

    Ok(())
}

/// Plays a short beep sound to indicate recording started.
fn play_system_sound() {
    // Use a brief high-pitched beep (800Hz for 150ms)
    std::thread::spawn(|| {
        #[cfg(target_os = "windows")]
        {
            // kernel32 Beep is always available
            extern "system" {
                fn Beep(dwFreq: u32, dwDuration: u32) -> i32;
            }
            unsafe {
                Beep(800, 150);
            }
        }
    });
}

/// Shows a small overlay that says "Recording..." then fades and closes itself.
fn show_recording_overlay(app: &AppHandle) {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;

    // Remove previous overlay if still open
    if let Some(w) = app.get_webview_window("recording-overlay") {
        let _ = w.close();
    }

    let html = r#"data:text/html,
    <!DOCTYPE html>
    <html>
    <head><style>
    * { margin:0; padding:0; }
    body {
        background: transparent;
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100vh;
        font-family: system-ui, sans-serif;
        overflow: hidden;
    }
    .toast {
        background: rgba(239, 68, 68, 0.9);
        color: white;
        padding: 8px 20px;
        border-radius: 24px;
        font-size: 13px;
        font-weight: 600;
        letter-spacing: 0.5px;
        display: flex;
        align-items: center;
        gap: 8px;
        animation: appear 0.2s ease-out, fade 0.4s ease-in 1.2s forwards;
        box-shadow: 0 4px 20px rgba(0,0,0,0.3);
    }
    .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: white;
        animation: pulse 1s ease-in-out infinite;
    }
    @keyframes appear {
        from { opacity: 0; transform: translateY(-8px); }
        to { opacity: 1; transform: translateY(0); }
    }
    @keyframes fade {
        to { opacity: 0; transform: translateY(-8px); }
    }
    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.4; }
    }
    </style></head>
    <body>
        <div class="toast">
            <span class="dot"></span>
            Recording...
        </div>
    </body>
    </html>"#;

    let builder = WebviewWindowBuilder::new(app, "recording-overlay", WebviewUrl::External(html.parse().unwrap()))
        .title("")
        .inner_size(200.0, 50.0)
        .center()
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false);

    if let Ok(window) = builder.build() {
        let _ = window.set_ignore_cursor_events(true);

        // Auto-close after the animation finishes (1.6s)
        let w = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1700));
            let _ = w.close();
        });
    }
}

#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    recorder: State<'_, AudioRecorder>,
    db: State<'_, Database>,
) -> Result<(), String> {
    let duration_ms = recorder.duration_ms() as i64;

    // Stop recording and get WAV data (this is blocking, so do it synchronously)
    let wav_data = recorder.stop_recording()?;

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
