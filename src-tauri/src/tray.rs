use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::{image::Image, AppHandle, Emitter, Manager};

/// Represents the current state of the tray icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayState {
    Idle,
    Recording,
    Processing,
}

/// The tray icon id used throughout the application.
const TRAY_ID: &str = "tinywhispr-tray";

/// Holds a reference to the toggle menu item so its text can be updated.
pub struct TrayMenuState {
    toggle_item: Mutex<MenuItem<tauri::Wry>>,
}

/// Creates the system tray icon with a context menu.
///
/// Menu items:
/// - "Start Recording" (id: toggle)
/// - Separator
/// - "Open Settings" (id: settings)
/// - "Open History" (id: history)
/// - Separator
/// - "Quit" (id: quit)
///
/// The toggle `MenuItem` is stored in Tauri's managed state as `TrayMenuState`
/// so that `update_tray_menu_text` can later change its text.
pub fn create_tray(app: &AppHandle) -> Result<(), String> {
    let toggle_item = MenuItemBuilder::with_id("toggle", "Start Recording")
        .build(app)
        .map_err(|e| format!("Failed to create toggle menu item: {e}"))?;

    let settings_item = MenuItemBuilder::with_id("settings", "Open Settings")
        .build(app)
        .map_err(|e| format!("Failed to create settings menu item: {e}"))?;

    let history_item = MenuItemBuilder::with_id("history", "Open History")
        .build(app)
        .map_err(|e| format!("Failed to create history menu item: {e}"))?;

    let quit_item = MenuItemBuilder::with_id("quit", "Quit")
        .build(app)
        .map_err(|e| format!("Failed to create quit menu item: {e}"))?;

    let sep1 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to create separator: {e}"))?;

    let sep2 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to create separator: {e}"))?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle_item,
            &sep1,
            &settings_item,
            &history_item,
            &sep2,
            &quit_item,
        ],
    )
    .map_err(|e| format!("Failed to create tray menu: {e}"))?;

    // Store the toggle item in managed state for later text updates
    app.manage(TrayMenuState {
        toggle_item: Mutex::new(toggle_item),
    });

    // Use the default app icon for the tray
    let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
        .map_err(|e| format!("Failed to load tray icon: {e}"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("TinyWhispr")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "toggle" => {
                    let _ = app.emit("tray-toggle-recording", ());
                }
                "settings" => {
                    show_main_window(app);
                    let _ = app.emit("navigate", "settings");
                }
                "history" => {
                    show_main_window(app);
                    let _ = app.emit("navigate", "history");
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            // Single left-click toggles recording
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = tray.app_handle().emit("tray-toggle-recording", ());
            }
        })
        .build(app)
        .map_err(|e| format!("Failed to build tray icon: {e}"))?;

    Ok(())
}

/// Shows the main window and brings it to focus.
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Updates the tray icon based on the current application state.
pub fn update_tray_icon(app: &AppHandle, state: TrayState) -> Result<(), String> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "Tray icon not found".to_string())?;

    let icon_bytes: &[u8] = match state {
        TrayState::Idle => include_bytes!("../icons/icon.png"),
        TrayState::Recording => include_bytes!("../icons/icon-recording.png"),
        TrayState::Processing => include_bytes!("../icons/icon-processing.png"),
    };

    let icon =
        Image::from_bytes(icon_bytes).map_err(|e| format!("Failed to load tray icon: {e}"))?;

    let tooltip = match state {
        TrayState::Idle => "TinyWhispr — Click to record",
        TrayState::Recording => "TinyWhispr — Recording...",
        TrayState::Processing => "TinyWhispr — Processing...",
    };

    tray.set_tooltip(Some(tooltip))
        .map_err(|e| format!("Failed to set tooltip: {e}"))?;

    tray.set_icon(Some(icon))
        .map_err(|e| format!("Failed to set tray icon: {e}"))?;

    Ok(())
}

/// Updates the text of the toggle menu item based on recording state.
pub fn update_tray_menu_text(app: &AppHandle, recording: bool) -> Result<(), String> {
    let menu_state = app.state::<TrayMenuState>();
    let toggle = menu_state
        .toggle_item
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;

    let text = if recording {
        "Stop Recording"
    } else {
        "Start Recording"
    };

    toggle
        .set_text(text)
        .map_err(|e| format!("Failed to set menu text: {e}"))?;

    Ok(())
}
