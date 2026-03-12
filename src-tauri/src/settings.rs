use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "whisper-1".to_string(),
            language: "auto".to_string(),
            hotkey: "Ctrl+Shift+Space".to_string(),
            activation_mode: "toggle".to_string(),
            output_mode: "auto_paste".to_string(),
            launch_at_startup: false,
        }
    }
}

/// Returns the path to the settings file: `{config_dir}/tinywhispr/settings.json`
pub fn settings_path() -> PathBuf {
    let config = dirs::config_dir().expect("Could not determine config directory");
    config.join("tinywhispr").join("settings.json")
}

/// Loads settings from disk, falling back to defaults if missing or corrupt.
pub fn load_settings() -> Settings {
    let path = settings_path();
    if !path.exists() {
        return Settings::default();
    }
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

/// Writes settings as pretty JSON to the settings file.
pub fn save_settings_to_file(settings: &Settings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {e}"))?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings file: {e}"))?;
    Ok(())
}

/// Validates settings, returning `Ok(())` if valid or `Err(message)` if not.
pub fn validate_settings(settings: &Settings) -> Result<(), String> {
    // base_url must start with http:// or https://
    if !settings.base_url.starts_with("http://") && !settings.base_url.starts_with("https://") {
        return Err("base_url must start with http:// or https://".to_string());
    }

    // activation_mode must be "toggle" or "push_to_talk"
    if settings.activation_mode != "toggle" && settings.activation_mode != "push_to_talk" {
        return Err(format!(
            "activation_mode must be 'toggle' or 'push_to_talk', got '{}'",
            settings.activation_mode
        ));
    }

    // output_mode must be "auto_paste" or "clipboard"
    if settings.output_mode != "auto_paste" && settings.output_mode != "clipboard" {
        return Err(format!(
            "output_mode must be 'auto_paste' or 'clipboard', got '{}'",
            settings.output_mode
        ));
    }

    // hotkey must have at least one modifier (Ctrl/Alt/Shift) + a non-modifier key
    validate_hotkey(&settings.hotkey)?;

    Ok(())
}

fn validate_hotkey(hotkey: &str) -> Result<(), String> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
    let modifiers = ["Ctrl", "Alt", "Shift"];

    let has_modifier = parts.iter().any(|p| modifiers.contains(p));
    let has_non_modifier = parts.iter().any(|p| !modifiers.contains(p));

    if !has_modifier || !has_non_modifier {
        return Err(
            "hotkey must have at least one modifier (Ctrl/Alt/Shift) and a non-modifier key"
                .to_string(),
        );
    }

    Ok(())
}

/// Masks an API key for display. Returns `"sk-••••"` for non-empty keys, `""` for empty.
pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        String::new()
    } else {
        "sk-\u{2022}\u{2022}\u{2022}\u{2022}".to_string()
    }
}

/// Validates incoming settings, preserves the existing API key if the incoming one
/// is empty or matches the mask pattern, saves to disk, and returns the merged settings.
pub fn save_settings_merged(incoming: Settings) -> Result<Settings, String> {
    validate_settings(&incoming)?;

    let existing = load_settings();

    let api_key = if incoming.api_key.is_empty() || incoming.api_key == mask_api_key(&existing.api_key)
    {
        existing.api_key
    } else {
        incoming.api_key
    };

    let merged = Settings {
        api_key,
        ..incoming
    };

    save_settings_to_file(&merged)?;
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let s = Settings::default();
        assert_eq!(s.provider, "openai");
        assert_eq!(s.api_key, "");
        assert_eq!(s.base_url, "https://api.openai.com/v1");
        assert_eq!(s.model, "whisper-1");
        assert_eq!(s.language, "auto");
        assert_eq!(s.hotkey, "Ctrl+Shift+Space");
        assert_eq!(s.activation_mode, "toggle");
        assert_eq!(s.output_mode, "auto_paste");
        assert!(!s.launch_at_startup);
    }

    #[test]
    fn test_validate_valid_settings() {
        let s = Settings::default();
        assert!(validate_settings(&s).is_ok());
    }

    #[test]
    fn test_validate_bad_url() {
        let mut s = Settings::default();
        s.base_url = "ftp://example.com".to_string();
        let err = validate_settings(&s).unwrap_err();
        assert!(err.contains("base_url"));
    }

    #[test]
    fn test_validate_bad_activation_mode() {
        let mut s = Settings::default();
        s.activation_mode = "hold".to_string();
        let err = validate_settings(&s).unwrap_err();
        assert!(err.contains("activation_mode"));
    }

    #[test]
    fn test_validate_bad_output_mode() {
        let mut s = Settings::default();
        s.output_mode = "print".to_string();
        let err = validate_settings(&s).unwrap_err();
        assert!(err.contains("output_mode"));
    }

    #[test]
    fn test_validate_hotkey_no_modifier() {
        let mut s = Settings::default();
        s.hotkey = "Space".to_string();
        let err = validate_settings(&s).unwrap_err();
        assert!(err.contains("modifier"));
    }

    #[test]
    fn test_validate_hotkey_only_modifier() {
        let mut s = Settings::default();
        s.hotkey = "Ctrl+Shift".to_string();
        let err = validate_settings(&s).unwrap_err();
        assert!(err.contains("non-modifier"));
    }

    #[test]
    fn test_mask_api_key_non_empty() {
        let masked = mask_api_key("sk-abc123");
        assert_eq!(masked, "sk-\u{2022}\u{2022}\u{2022}\u{2022}");
    }

    #[test]
    fn test_mask_api_key_empty() {
        let masked = mask_api_key("");
        assert_eq!(masked, "");
    }
}
