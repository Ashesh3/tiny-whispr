use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Manages a single global hotkey registration, replacing the previous one
/// whenever a new shortcut is registered.
pub struct HotkeyManager {
    current_shortcut: Mutex<Option<String>>,
}

impl HotkeyManager {
    /// Creates a new HotkeyManager with no shortcut registered.
    pub fn new() -> Self {
        Self {
            current_shortcut: Mutex::new(None),
        }
    }

    /// Registers a global hotkey. Unregisters any previously registered shortcut first.
    /// The callback is invoked on key-down events only.
    pub fn register<F>(&self, app: &AppHandle, hotkey_str: &str, callback: F) -> Result<(), String>
    where
        F: Fn(&AppHandle) + Send + Sync + 'static,
    {
        // Unregister previous shortcut if any
        self.unregister(app)?;

        let gs = app.global_shortcut();

        gs.on_shortcut(hotkey_str, move |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                callback(app);
            }
        })
        .map_err(|e| format!("Failed to register hotkey '{}': {}", hotkey_str, e))?;

        let mut current = self
            .current_shortcut
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;
        *current = Some(hotkey_str.to_string());

        Ok(())
    }

    /// Unregisters the currently registered shortcut, if any.
    pub fn unregister(&self, app: &AppHandle) -> Result<(), String> {
        let mut current = self
            .current_shortcut
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;

        if let Some(shortcut_str) = current.take() {
            let gs = app.global_shortcut();
            gs.unregister(shortcut_str.as_str())
                .map_err(|e| format!("Failed to unregister hotkey '{}': {}", shortcut_str, e))?;
        }

        Ok(())
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
