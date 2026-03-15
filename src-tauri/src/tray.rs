use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
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

    let mic_submenu = build_mic_submenu(app)?;

    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep3 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[&toggle_item, &sep1, &mic_submenu, &sep2, &settings_item, &history_item, &sep3, &quit_item],
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
            let id = event.id().as_ref();
            match id {
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
                _ if id.starts_with("mic:") => {
                    let device_id = &id[4..]; // strip "mic:" prefix
                    let mut settings = crate::settings::load_settings();
                    settings.microphone_device_id = device_id.to_string();
                    let _ = crate::settings::save_settings_to_file(&settings);
                    // Rebuild entire tray to update mic checkmarks
                    let _ = rebuild_tray(app);
                    let _ = app.emit("settings-changed", ());
                }
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

/// Rebuilds the tray menu to reflect updated settings (e.g. mic selection).
fn rebuild_tray(app: &AppHandle) -> Result<(), String> {
    let toggle_item = MenuItemBuilder::with_id("toggle", "Start Recording")
        .build(app).map_err(|e| e.to_string())?;
    let settings_item = MenuItemBuilder::with_id("settings", "Open Settings")
        .build(app).map_err(|e| e.to_string())?;
    let history_item = MenuItemBuilder::with_id("history", "Open History")
        .build(app).map_err(|e| e.to_string())?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit")
        .build(app).map_err(|e| e.to_string())?;
    let mic_submenu = build_mic_submenu(app)?;
    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep3 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[&toggle_item, &sep1, &mic_submenu, &sep2, &settings_item, &history_item, &sep3, &quit_item],
    ).map_err(|e| e.to_string())?;

    // Update managed toggle item
    let managed = app.state::<TrayManagedState>();
    *managed.toggle_item.lock().map_err(|e| format!("Lock error: {e}"))? = toggle_item;

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
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

fn build_mic_submenu(app: &AppHandle) -> Result<tauri::menu::Submenu<tauri::Wry>, String> {
    let settings = crate::settings::load_settings();
    let devices = crate::audio::list_input_devices();

    let mut builder = SubmenuBuilder::with_id(app, "mic-submenu", "Microphone");

    // "System Default" option
    let default_label = if settings.microphone_device_id.is_empty() {
        "System Default  \u{2713}"
    } else {
        "System Default"
    };
    builder = builder.item(
        &MenuItemBuilder::with_id("mic:", default_label)
            .build(app)
            .map_err(|e| e.to_string())?,
    );

    // Separator
    builder = builder.separator();

    // Individual devices
    for device in &devices {
        let is_selected = device.id == settings.microphone_device_id;
        let label = if is_selected {
            format!("{}  \u{2713}", device.name)
        } else {
            device.name.clone()
        };
        let id = format!("mic:{}", device.id);
        builder = builder.item(
            &MenuItemBuilder::with_id(id, label)
                .build(app)
                .map_err(|e| e.to_string())?,
        );
    }

    builder.build().map_err(|e| e.to_string())
}
