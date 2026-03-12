#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tinywhispr::db::Database;

fn main() {
    let db = Database::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            tinywhispr::commands::get_settings,
            tinywhispr::commands::save_settings,
            tinywhispr::commands::get_history,
            tinywhispr::commands::delete_transcription,
            tinywhispr::commands::clear_history,
            tinywhispr::commands::search_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
