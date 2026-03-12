use serde::Serialize;
use tauri::State;

use crate::db::{Database, Transcription};
use crate::settings::{load_settings, mask_api_key, save_settings_merged, Settings};

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
pub fn save_settings(settings: Settings) -> Result<MaskedSettings, String> {
    let saved = save_settings_merged(settings)?;
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
pub fn search_history(db: State<'_, Database>, query: String) -> Result<Vec<Transcription>, String> {
    db.search(&query)
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    crate::output::copy_to_clipboard(&text)
}
