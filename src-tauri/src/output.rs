use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Copies text to the system clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {e}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to copy to clipboard: {e}"))?;
    Ok(())
}

/// Copies text to clipboard and simulates Ctrl+V to paste it into the active window.
pub fn auto_paste(text: &str) -> Result<(), String> {
    copy_to_clipboard(text)?;

    // Small delay to ensure clipboard is ready
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut enigo = Enigo::new(&EnigoSettings::default())
        .map_err(|e| format!("Failed to initialize enigo: {e}"))?;

    // Press Ctrl+V
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| format!("Failed to press Ctrl: {e}"))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("Failed to press V: {e}"))?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| format!("Failed to release Ctrl: {e}"))?;

    Ok(())
}

/// Dispatches text output based on the specified mode.
///
/// - `"auto_paste"`: copies to clipboard and simulates Ctrl+V
/// - `"clipboard"` (or any other value): copies to clipboard only
pub fn output_text(text: &str, mode: &str) -> Result<(), String> {
    match mode {
        "auto_paste" => auto_paste(text),
        _ => copy_to_clipboard(text),
    }
}

/// Dispatches text output and sends a notification in clipboard mode.
///
/// - `"auto_paste"`: copies to clipboard and simulates Ctrl+V
/// - `"clipboard"`: copies to clipboard and shows a notification
pub fn output_text_with_notify(app: &AppHandle, text: &str, mode: &str) -> Result<(), String> {
    output_text(text, mode)?;

    if mode != "auto_paste" {
        let _ = app
            .notification()
            .builder()
            .title("TinyWhispr")
            .body("Transcription copied to clipboard")
            .show();
    }

    Ok(())
}
