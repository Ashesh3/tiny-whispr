#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;
use tinywhispr::audio::AudioRecorder;
use tinywhispr::db::Database;
use tinywhispr::hotkey::{register_from_settings, HotkeyManager};
use tinywhispr::settings::load_settings;
use tinywhispr::tray;

fn main() {
    let db = Database::new().expect("Failed to initialize database");
    let recorder = AudioRecorder::new();
    let hotkey_mgr = HotkeyManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(db)
        .manage(recorder)
        .manage(hotkey_mgr)
        .invoke_handler(tauri::generate_handler![
            tinywhispr::commands::get_settings,
            tinywhispr::commands::save_settings,
            tinywhispr::commands::get_history,
            tinywhispr::commands::delete_transcription,
            tinywhispr::commands::clear_history,
            tinywhispr::commands::search_history,
            tinywhispr::commands::copy_to_clipboard,
            tinywhispr::commands::start_recording,
            tinywhispr::commands::stop_recording,
            tinywhispr::commands::get_recording_state,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Create system tray
            tray::create_tray(&app_handle)
                .expect("Failed to create system tray");

            // Load settings and register global hotkey
            let settings = load_settings();
            let hotkey_mgr = app_handle.state::<HotkeyManager>();
            if let Err(e) = register_from_settings(&app_handle, &hotkey_mgr, &settings) {
                eprintln!("Failed to register hotkey: {e}");
            }

            // Set autostart state
            if settings.launch_at_startup {
                let _ = app.handle().autolaunch().enable();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide main window on close instead of quitting
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
