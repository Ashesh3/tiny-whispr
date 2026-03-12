use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{image::Image, AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayState {
    Idle,
    Recording,
    Processing,
}

const TRAY_ID: &str = "tinywhispr-tray";

/// Shared state for tray menu item + pulse animation control.
pub struct TrayAnimState {
    toggle_item: Mutex<tauri::menu::MenuItem<tauri::Wry>>,
    pulsing: Arc<AtomicBool>,
}

/// Creates the system tray icon with context menu and left-click toggle.
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

    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

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
    .map_err(|e| e.to_string())?;

    app.manage(TrayAnimState {
        toggle_item: Mutex::new(toggle_item),
        pulsing: Arc::new(AtomicBool::new(false)),
    });

    let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
        .map_err(|e| format!("Failed to load tray icon: {e}"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("TinyWhispr — Click to record")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
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
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
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

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Updates the tray icon and starts/stops the pulse animation.
pub fn update_tray_icon(app: &AppHandle, state: TrayState) -> Result<(), String> {
    let anim_state = app.state::<TrayAnimState>();

    // Stop any existing pulse
    anim_state.pulsing.store(false, Ordering::Relaxed);

    // Set the static icon first
    set_tray_icon_bytes(app, match state {
        TrayState::Idle => include_bytes!("../icons/icon.png"),
        TrayState::Recording => include_bytes!("../icons/icon-recording.png"),
        TrayState::Processing => include_bytes!("../icons/icon-processing.png"),
    })?;

    // Set tooltip
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = match state {
            TrayState::Idle => "TinyWhispr — Click to record",
            TrayState::Recording => "TinyWhispr — Recording...",
            TrayState::Processing => "TinyWhispr — Processing...",
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }

    // Start pulse animation for recording
    if state == TrayState::Recording {
        anim_state.pulsing.store(true, Ordering::Relaxed);
        let pulsing = anim_state.pulsing.clone();
        let handle = app.clone();

        std::thread::spawn(move || {
            let bright = include_bytes!("../icons/icon-recording.png");
            let dim = include_bytes!("../icons/icon-recording-dim.png");
            let mut show_bright = true;

            while pulsing.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(600));
                if !pulsing.load(Ordering::Relaxed) {
                    break;
                }
                show_bright = !show_bright;
                let _ = set_tray_icon_bytes(
                    &handle,
                    if show_bright { bright } else { dim },
                );
            }
        });
    }

    Ok(())
}

fn set_tray_icon_bytes(app: &AppHandle, bytes: &[u8]) -> Result<(), String> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "Tray icon not found".to_string())?;
    let icon = Image::from_bytes(bytes).map_err(|e| format!("Failed to load icon: {e}"))?;
    tray.set_icon(Some(icon))
        .map_err(|e| format!("Failed to set icon: {e}"))
}

/// Updates the toggle menu item text.
pub fn update_tray_menu_text(app: &AppHandle, recording: bool) -> Result<(), String> {
    let anim_state = app.state::<TrayAnimState>();
    let toggle = anim_state
        .toggle_item
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;

    toggle
        .set_text(if recording { "Stop Recording" } else { "Start Recording" })
        .map_err(|e| format!("Failed to set menu text: {e}"))
}
