use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::settings::Settings;

struct RegisteredHotkey {
    shortcut: String,
    activation_mode: String,
}

/// Manages a single global hotkey registration, replacing the previous one
/// whenever a new shortcut is registered.
pub struct HotkeyManager {
    current_shortcut: Mutex<Option<RegisteredHotkey>>,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyManager {
    /// Creates a new HotkeyManager with no shortcut registered.
    pub fn new() -> Self {
        Self {
            current_shortcut: Mutex::new(None),
        }
    }

    /// Registers a global hotkey using the requested activation mode.
    ///
    /// - `toggle`: emit a toggle event on key-down
    /// - `push_to_talk`: emit start on key-down and stop on key-up
    pub fn register(
        &self,
        app: &AppHandle,
        hotkey_str: &str,
        activation_mode: &str,
    ) -> Result<(), String> {
        // Unregister previous shortcut if any
        self.unregister(app)?;

        let gs = app.global_shortcut();

        let pressed_event = match activation_mode {
            "toggle" => "tray-toggle-recording",
            "push_to_talk" => "shortcut-start-recording",
            other => {
                return Err(format!(
                    "Unsupported activation mode '{other}'. Expected 'toggle' or 'push_to_talk'"
                ));
            }
        };

        gs.on_shortcut(hotkey_str, move |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                let _ = app.emit(pressed_event, ());
            }
        })
        .map_err(|e| format!("Failed to register hotkey '{}': {}", hotkey_str, e))?;

        #[cfg(target_os = "windows")]
        if activation_mode == "push_to_talk" {
            let vkey = parse_hotkey_virtual_key(hotkey_str)?;
            let app_handle = app.clone();
            ptt::install_key_up_hook(vkey, move || {
                let _ = app_handle.emit("shortcut-stop-recording", ());
            })?;
        }

        let mut current = self
            .current_shortcut
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;
        *current = Some(RegisteredHotkey {
            shortcut: hotkey_str.to_string(),
            activation_mode: activation_mode.to_string(),
        });

        Ok(())
    }

    /// Unregisters the currently registered shortcut, if any.
    pub fn unregister(&self, app: &AppHandle) -> Result<(), String> {
        let mut current = self
            .current_shortcut
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;

        if let Some(registered) = current.take() {
            let gs = app.global_shortcut();
            gs.unregister(registered.shortcut.as_str()).map_err(|e| {
                format!(
                    "Failed to unregister hotkey '{}': {}",
                    registered.shortcut, e
                )
            })?;

            #[cfg(target_os = "windows")]
            if registered.activation_mode == "push_to_talk" {
                ptt::uninstall_key_up_hook()?;
            }
        }

        Ok(())
    }
}

pub fn register_from_settings(
    app: &AppHandle,
    hotkey_mgr: &HotkeyManager,
    settings: &Settings,
) -> Result<(), String> {
    hotkey_mgr.register(app, &settings.hotkey, &settings.activation_mode)
}

#[cfg(target_os = "windows")]
fn parse_hotkey_virtual_key(hotkey: &str) -> Result<u32, String> {
    let key = hotkey
        .split('+')
        .map(str::trim)
        .filter(|part| !matches!(*part, "Ctrl" | "Alt" | "Shift"))
        .next_back()
        .ok_or_else(|| format!("Hotkey '{hotkey}' is missing a non-modifier key"))?;

    match key.to_uppercase().as_str() {
        "SPACE" => Ok(0x20),
        "ENTER" => Ok(0x0D),
        "TAB" => Ok(0x09),
        "ESC" | "ESCAPE" => Ok(0x1B),
        "BACKSPACE" => Ok(0x08),
        "DELETE" => Ok(0x2E),
        "INSERT" => Ok(0x2D),
        "HOME" => Ok(0x24),
        "END" => Ok(0x23),
        "PAGEUP" => Ok(0x21),
        "PAGEDOWN" => Ok(0x22),
        "UP" => Ok(0x26),
        "DOWN" => Ok(0x28),
        "LEFT" => Ok(0x25),
        "RIGHT" => Ok(0x27),
        key if key.len() == 1 => Ok(key.as_bytes()[0] as u32),
        key if key.starts_with('F') => {
            let function_number = key[1..]
                .parse::<u32>()
                .map_err(|_| format!("Unsupported hotkey key '{key}' for push-to-talk"))?;
            if (1..=24).contains(&function_number) {
                Ok(0x70 + function_number - 1)
            } else {
                Err(format!("Unsupported hotkey key '{key}' for push-to-talk"))
            }
        }
        _ => Err(format!(
            "Unsupported hotkey key '{key}' for push-to-talk. Use a letter, digit, function key, or a common navigation key."
        )),
    }
}

/// Push-to-talk support using a low-level Windows keyboard hook.
/// Detects key-up events for a specified virtual key code.
#[cfg(target_os = "windows")]
pub mod ptt {
    use std::sync::{Mutex, OnceLock};

    use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYUP,
    };

    struct HookState {
        vkey: u32,
        callback: Box<dyn Fn() + Send + Sync>,
        hook_handle: HHOOK,
    }

    // SAFETY: HHOOK is a raw pointer but we only access it from the hook thread
    // or to unhook. The actual hook callback runs on the hook thread.
    unsafe impl Send for HookState {}
    unsafe impl Sync for HookState {}

    static HOOK_STATE: OnceLock<Mutex<Option<HookState>>> = OnceLock::new();

    fn hook_state() -> &'static Mutex<Option<HookState>> {
        HOOK_STATE.get_or_init(|| Mutex::new(None))
    }

    /// Low-level keyboard hook procedure.
    /// Called by Windows for every keyboard event system-wide.
    unsafe extern "system" fn hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code >= 0 && wparam.0 as u32 == WM_KEYUP {
            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            if let Ok(state) = hook_state().lock() {
                if let Some(ref hs) = *state {
                    if kb.vkCode == hs.vkey {
                        (hs.callback)();
                    }
                }
            }
        }
        unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
    }

    /// Installs a Windows low-level keyboard hook that detects key-up events
    /// for the specified virtual key code and calls the callback.
    ///
    /// Only one hook can be active at a time. Installing a new hook removes the previous one.
    /// Runs a message loop in a background thread to keep the hook alive.
    pub fn install_key_up_hook<F>(vkey: u32, callback: F) -> Result<(), String>
    where
        F: Fn() + Send + Sync + 'static,
    {
        // Remove existing hook if any
        remove_hook()?;

        // Install hook on a background thread with its own message loop
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

        std::thread::spawn(move || {
            let hook_result = unsafe {
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), HINSTANCE::default(), 0)
            };

            match hook_result {
                Ok(hook) => {
                    {
                        let mut state = hook_state().lock().unwrap();
                        *state = Some(HookState {
                            vkey,
                            callback: Box::new(callback),
                            hook_handle: hook,
                        });
                    }
                    let _ = tx.send(Ok(()));

                    // Run message loop to keep the hook alive
                    unsafe {
                        let mut msg = MSG::default();
                        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                            let _ = TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to install keyboard hook: {e}")));
                }
            }
        });

        rx.recv()
            .map_err(|_| "Hook thread terminated unexpectedly".to_string())?
    }

    /// Removes the currently installed keyboard hook, if any.
    pub fn uninstall_key_up_hook() -> Result<(), String> {
        remove_hook()
    }

    fn remove_hook() -> Result<(), String> {
        let mut state = hook_state()
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;

        if let Some(hs) = state.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(hs.hook_handle);
            }
        }

        Ok(())
    }
}
