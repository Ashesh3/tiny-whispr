use std::sync::atomic::{AtomicU64, Ordering};
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

static ICON_IDLE: &[u8] = include_bytes!("../icons/icon.png");
static ICON_RECORDING: &[u8] = include_bytes!("../icons/icon-recording.png");
static ICON_RECORDING_DIM: &[u8] = include_bytes!("../icons/icon-recording-dim.png");
static ICON_PROCESSING: &[u8] = include_bytes!("../icons/icon-processing.png");

/// Managed state for tray menu item and pulse animation control.
pub struct TrayManagedState {
    toggle_item: Mutex<tauri::menu::MenuItem<tauri::Wry>>,
    /// Generation counter: each call to set_tray_state increments this.
    /// Animation threads exit when the generation no longer matches.
    generation: Arc<AtomicU64>,
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
        &[&toggle_item, &sep1, &settings_item, &history_item, &sep2, &quit_item],
    )
    .map_err(|e| e.to_string())?;

    app.manage(TrayManagedState {
        toggle_item: Mutex::new(toggle_item),
        generation: Arc::new(AtomicU64::new(0)),
    });

    let icon = Image::from_bytes(ICON_IDLE)
        .map_err(|e| format!("Failed to load tray icon: {e}"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("TinyWhispr — Click to record")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "toggle" => { let _ = app.emit("tray-toggle-recording", ()); }
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

/// Sets the tray icon, tooltip, and menu text for the given state.
/// Starts pulse animation when recording, stops it for any other state.
pub fn set_tray_state(app: &AppHandle, state: TrayState) -> Result<(), String> {
    let managed = app.state::<TrayManagedState>();

    // Bump generation to stop any running animation thread
    let gen = managed.generation.fetch_add(1, Ordering::SeqCst) + 1;

    // Set static icon
    let icon_bytes = match state {
        TrayState::Idle => ICON_IDLE,
        TrayState::Recording => ICON_RECORDING,
        TrayState::Processing => ICON_PROCESSING,
    };
    set_tray_icon_bytes(app, icon_bytes)?;

    // Set tooltip
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = match state {
            TrayState::Idle => "TinyWhispr — Click to record",
            TrayState::Recording => "TinyWhispr — Recording...",
            TrayState::Processing => "TinyWhispr — Processing...",
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }

    // Set menu text
    {
        let toggle = managed.toggle_item.lock().map_err(|e| format!("Lock error: {e}"))?;
        let text = match state {
            TrayState::Recording => "Stop Recording",
            _ => "Start Recording",
        };
        let _ = toggle.set_text(text);
    }

    // Start pulse animation for recording
    if state == TrayState::Recording {
        let generation = managed.generation.clone();
        let handle = app.clone();

        std::thread::spawn(move || {
            // Pre-decode both images once
            let bright = match Image::from_bytes(ICON_RECORDING) {
                Ok(img) => img,
                Err(_) => return,
            };
            let dim = match Image::from_bytes(ICON_RECORDING_DIM) {
                Ok(img) => img,
                Err(_) => return,
            };

            let mut show_bright = true;
            while generation.load(Ordering::SeqCst) == gen {
                std::thread::sleep(Duration::from_millis(600));
                if generation.load(Ordering::SeqCst) != gen {
                    break;
                }
                show_bright = !show_bright;
                if let Some(tray) = handle.tray_by_id(TRAY_ID) {
                    let icon = if show_bright { bright.clone() } else { dim.clone() };
                    let _ = tray.set_icon(Some(icon));
                }
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
