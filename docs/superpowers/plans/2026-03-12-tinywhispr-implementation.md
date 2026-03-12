# TinyWhispr Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimalistic Windows system tray speech-to-text app using cloud transcription APIs.

**Architecture:** Tauri v2 app with Rust backend handling audio capture, API calls, and system integration. Svelte 5 + Tailwind CSS v4 frontend with two windows: a main settings/history window and a floating recording indicator.

**Tech Stack:** Tauri v2, Rust, Svelte 5, Tailwind CSS v4, cpal, hound, rubato, reqwest, enigo, arboard, rusqlite

**Spec:** `docs/superpowers/specs/2026-03-12-tinywhispr-design.md`

---

## Chunk 1: Project Scaffolding

### Task 1: Initialize Tauri v2 + Svelte 5 project

**Files:**
- Create: `package.json`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `index.html`
- Create: `src/main.ts`, `src/App.svelte`, `src/app.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize npm project and install frontend dependencies**

```bash
cd F:/Projects/tinywhispr
npm init -y
npm install -D @sveltejs/vite-plugin-svelte svelte vite typescript @tauri-apps/cli@latest
npm install @tauri-apps/api@latest @tauri-apps/plugin-global-shortcut @tauri-apps/plugin-notification @tauri-apps/plugin-autostart
```

- [ ] **Step 2: Initialize Tauri in the project**

```bash
cd F:/Projects/tinywhispr
npx tauri init --app-name tinywhispr --window-title TinyWhispr --dev-url http://localhost:5173 --before-dev-command "npm run dev" --before-build-command "npm run build" --frontend-dist ../dist
```

- [ ] **Step 3: Create `vite.config.ts`**

```typescript
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "chrome105",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: "dist",
    rollupOptions: {
      input: {
        main: "index.html",
        indicator: "src/indicator/indicator.html",
      },
    },
  },
});
```

- [ ] **Step 4: Create `svelte.config.js`**

```javascript
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  preprocess: vitePreprocess(),
};
```

- [ ] **Step 5: Create `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "resolveJsonModule": true,
    "allowJs": true,
    "checkJs": true,
    "isolatedModules": true,
    "moduleResolution": "bundler",
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "types": ["svelte"]
  },
  "include": ["src/**/*.ts", "src/**/*.svelte"]
}
```

- [ ] **Step 6: Create `index.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>TinyWhispr</title>
    <link rel="stylesheet" href="/src/app.css" />
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 7: Create `src/main.ts`**

```typescript
import App from "./App.svelte";
import { mount } from "svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

- [ ] **Step 8: Create `src/App.svelte` (minimal placeholder)**

```svelte
<main class="bg-[#111111] text-[#e5e5e5] h-screen font-sans text-[13px]">
  <p class="p-4">TinyWhispr</p>
</main>
```

- [ ] **Step 9: Create `src/app.css`**

```css
@import "tailwindcss";

:root {
  font-family: system-ui, -apple-system, sans-serif;
}

body {
  margin: 0;
  padding: 0;
  overflow: hidden;
}
```

- [ ] **Step 10: Create indicator window entry point `src/indicator/indicator.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="/src/app.css" />
  </head>
  <body style="background: transparent;">
    <div id="indicator"></div>
    <script type="module" src="/src/indicator/indicator-main.ts"></script>
  </body>
</html>
```

- [ ] **Step 11: Create `src/indicator/indicator-main.ts` and `src/indicator/IndicatorApp.svelte`**

`src/indicator/indicator-main.ts`:
```typescript
import IndicatorApp from "./IndicatorApp.svelte";
import { mount } from "svelte";

const app = mount(IndicatorApp, { target: document.getElementById("indicator")! });

export default app;
```

`src/indicator/IndicatorApp.svelte`:
```svelte
<div class="indicator hidden">
  <span class="dot"></span>
  <span class="label">REC</span>
</div>
```

- [ ] **Step 12: Configure Tailwind CSS v4 theme in `src/app.css`**

Tailwind v4 uses CSS-based configuration, NOT `tailwind.config.ts`. Update `src/app.css` to define the custom theme tokens:

```css
@import "tailwindcss";

@theme {
  --color-background: #111111;
  --color-surface: #1a1a1a;
  --color-border: #262626;
  --color-text-primary: #e5e5e5;
  --color-text-secondary: #a3a3a3;
  --color-text-muted: #737373;
  --color-accent: #3b82f6;
  --color-recording: #ef4444;
  --color-processing: #3b82f6;
}

:root {
  font-family: system-ui, -apple-system, sans-serif;
}

body {
  margin: 0;
  padding: 0;
  overflow: hidden;
}
```

Do NOT create a `tailwind.config.ts` — Tailwind v4 auto-detects content from the project and uses `@theme` for custom values.

- [ ] **Step 12b: Remove the separate `src/app.css` created in Step 9**

Step 9's `app.css` is now superseded by Step 12. The theme tokens and base styles are all in one file. Delete the Step 9 version and use only Step 12's version.

- [ ] **Step 12c: Configure Tauri v2 capabilities/permissions**

Tauri v2 requires explicit capability grants for plugins and IPC commands. Create `src-tauri/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "Default capabilities for TinyWhispr",
  "windows": ["main", "indicator"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-close",
    "core:window:allow-set-focus",
    "core:window:allow-set-ignore-cursor-events",
    "global-shortcut:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "notification:default",
    "notification:allow-notify",
    "autostart:default",
    "autostart:allow-enable",
    "autostart:allow-disable"
  ]
}

- [ ] **Step 13: Add npm scripts to `package.json`**

Add to `package.json`:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri"
  }
}
```

- [ ] **Step 14: Configure Tauri for two windows in `src-tauri/tauri.conf.json`**

Update the `windows` array and app config:
```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "TinyWhispr",
        "url": "/",
        "width": 420,
        "height": 520,
        "resizable": false,
        "visible": false,
        "decorations": false,
        "skipTaskbar": true,
        "center": true
      },
      {
        "label": "indicator",
        "url": "/src/indicator/indicator.html",
        "width": 80,
        "height": 28,
        "resizable": false,
        "visible": false,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "x": 680,
        "y": 20
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  },
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "bundle": {
    "active": true,
    "targets": "nsis",
    "icon": ["icons/icon.png", "icons/icon.ico"]
  },
  "plugins": {}
}
```

- [ ] **Step 15: Update Rust dependencies in `src-tauri/Cargo.toml`**

Add all required dependencies:
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-notification = "2"
tauri-plugin-autostart = { version = "2", features = ["default"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cpal = "0.15"
hound = "3.5"
rubato = "0.16"
reqwest = { version = "0.12", features = ["multipart", "json"] }
enigo = "0.2"
arboard = "3"
rusqlite = { version = "0.32", features = ["bundled"] }
tokio = { version = "1", features = ["full"] }
windows = { version = "0.58", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }
dirs = "5"
chrono = "0.4"
```

- [ ] **Step 16: Create minimal `src-tauri/src/lib.rs`**

```rust
pub mod commands;
pub mod settings;
pub mod db;
pub mod audio;
pub mod transcribe;
pub mod hotkey;
pub mod output;
pub mod tray;
```

- [ ] **Step 17: Create minimal `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 18: Create stub files for all Rust modules**

Create empty stubs so `lib.rs` compiles. Each file gets a placeholder:

`src-tauri/src/settings.rs`:
```rust
// Settings management — will be implemented in Task 2
```

`src-tauri/src/db.rs`:
```rust
// Database management — will be implemented in Task 3
```

`src-tauri/src/audio.rs`:
```rust
// Audio capture — will be implemented in Task 6
```

`src-tauri/src/transcribe.rs`:
```rust
// Transcription API client — will be implemented in Task 7
```

`src-tauri/src/hotkey.rs`:
```rust
// Hotkey management — will be implemented in Task 9
```

`src-tauri/src/output.rs`:
```rust
// Output (paste/clipboard) — will be implemented in Task 8
```

`src-tauri/src/tray.rs`:
```rust
// System tray — will be implemented in Task 10
```

`src-tauri/src/commands.rs`:
```rust
// Tauri IPC commands — will be implemented incrementally
```

- [ ] **Step 19: Create `.gitignore`**

```
node_modules/
dist/
src-tauri/target/
.superpowers/
*.log
```

- [ ] **Step 20: Create placeholder tray icons**

Create `src-tauri/icons/` directory. Generate simple placeholder `.ico` and `.png` files (can be replaced with real icons later). The build needs at least `icon.ico` and `icon.png` to succeed.

- [ ] **Step 21: Verify the project builds**

```bash
cd F:/Projects/tinywhispr
npm run tauri build -- --debug
```

Expected: Build succeeds. The app launches showing a dark window with "TinyWhispr" text.

- [ ] **Step 22: Initialize git and commit**

```bash
cd F:/Projects/tinywhispr
git init
git add .
git commit -m "feat: scaffold Tauri v2 + Svelte 5 + Tailwind project"
```

---

## Chunk 2: Settings & Database Backend

### Task 2: Settings module

**Files:**
- Create: `src-tauri/src/settings.rs`

- [ ] **Step 1: Write settings struct and defaults**

```rust
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
            provider: "openai".into(),
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".into(),
            model: "whisper-1".into(),
            language: "auto".into(),
            hotkey: "Ctrl+Shift+Space".into(),
            activation_mode: "toggle".into(),
            output_mode: "auto_paste".into(),
            launch_at_startup: false,
        }
    }
}
```

- [ ] **Step 2: Implement settings file path resolution**

```rust
fn settings_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tinywhispr");
    fs::create_dir_all(&dir).ok();
    dir.join("settings.json")
}
```

- [ ] **Step 3: Implement load/save functions**

```rust
pub fn load_settings() -> Settings {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => {
            let settings = Settings::default();
            save_settings_to_file(&settings).ok();
            settings
        }
    }
}

fn save_settings_to_file(settings: &Settings) -> Result<(), String> {
    let path = settings_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
```

- [ ] **Step 4: Implement validation**

```rust
pub fn validate_settings(settings: &Settings) -> Result<(), String> {
    // base_url must look like a URL
    if !settings.base_url.starts_with("http://") && !settings.base_url.starts_with("https://") {
        return Err("base_url must start with http:// or https://".into());
    }
    // activation_mode must be valid
    if settings.activation_mode != "toggle" && settings.activation_mode != "push_to_talk" {
        return Err("activation_mode must be 'toggle' or 'push_to_talk'".into());
    }
    // output_mode must be valid
    if settings.output_mode != "auto_paste" && settings.output_mode != "clipboard" {
        return Err("output_mode must be 'auto_paste' or 'clipboard'".into());
    }
    // hotkey must have modifier + key
    let parts: Vec<&str> = settings.hotkey.split('+').collect();
    if parts.len() < 2 {
        return Err("hotkey must include at least one modifier and a key".into());
    }
    let has_modifier = parts.iter().any(|p| {
        let p = p.trim();
        p == "Ctrl" || p == "Alt" || p == "Shift"
    });
    if !has_modifier {
        return Err("hotkey must include Ctrl, Alt, or Shift".into());
    }
    Ok(())
}
```

- [ ] **Step 5: Implement masked API key helper and save-with-merge**

```rust
const API_KEY_MASK: &str = "sk-••••";

pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        String::new()
    } else {
        API_KEY_MASK.to_string()
    }
}

pub fn save_settings_merged(incoming: Settings) -> Result<Settings, String> {
    validate_settings(&incoming)?;
    let mut current = load_settings();
    // Preserve existing API key if incoming is empty or masked
    let new_key = if incoming.api_key.is_empty() || incoming.api_key == API_KEY_MASK {
        current.api_key.clone()
    } else {
        incoming.api_key.clone()
    };
    current = Settings {
        api_key: new_key,
        ..incoming
    };
    save_settings_to_file(&current)?;
    Ok(current)
}
```

- [ ] **Step 6: Write unit tests for settings**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = Settings::default();
        assert_eq!(s.provider, "openai");
        assert_eq!(s.hotkey, "Ctrl+Shift+Space");
        assert_eq!(s.activation_mode, "toggle");
    }

    #[test]
    fn test_validate_valid() {
        let s = Settings::default();
        assert!(validate_settings(&s).is_ok());
    }

    #[test]
    fn test_validate_bad_url() {
        let mut s = Settings::default();
        s.base_url = "not-a-url".into();
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn test_validate_bad_activation_mode() {
        let mut s = Settings::default();
        s.activation_mode = "invalid".into();
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn test_validate_hotkey_no_modifier() {
        let mut s = Settings::default();
        s.hotkey = "Space".into();
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk-abc123"), "sk-••••");
        assert_eq!(mask_api_key(""), "");
    }
}
```

- [ ] **Step 7: Run tests**

```bash
cd F:/Projects/tinywhispr/src-tauri
cargo test settings -- --nocapture
```

Expected: All tests pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat: implement settings module with validation and API key masking"
```

### Task 3: Database module

**Files:**
- Create: `src-tauri/src/db.rs`

- [ ] **Step 1: Implement database initialization and schema**

```rust
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: i64,
    pub text: String,
    pub provider: String,
    pub model: String,
    pub language: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, String> {
        let path = Self::db_path();
        let conn = Connection::open(&path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                language TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_transcriptions_created_at
                ON transcriptions(created_at DESC);"
        ).map_err(|e| e.to_string())?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn db_path() -> PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tinywhispr");
        std::fs::create_dir_all(&dir).ok();
        dir.join("history.db")
    }
}
```

- [ ] **Step 2: Implement CRUD operations**

```rust
impl Database {
    pub fn insert(&self, text: &str, provider: &str, model: &str, language: Option<&str>, duration_ms: Option<i64>) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO transcriptions (text, provider, model, language, duration_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![text, provider, model, language, duration_ms],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all(&self) -> Result<Vec<Transcription>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, text, provider, model, language, duration_ms, created_at FROM transcriptions ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(Transcription {
                id: row.get(0)?,
                text: row.get(1)?,
                provider: row.get(2)?,
                model: row.get(3)?,
                language: row.get(4)?,
                duration_ms: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn delete(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM transcriptions WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM transcriptions", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<Transcription>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, text, provider, model, language, duration_ms, created_at FROM transcriptions WHERE text LIKE ?1 ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(Transcription {
                id: row.get(0)?,
                text: row.get(1)?,
                provider: row.get(2)?,
                model: row.get(3)?,
                language: row.get(4)?,
                duration_ms: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }
}
```

- [ ] **Step 3: Write unit tests for database**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE transcriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                language TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        ).unwrap();
        Database { conn: Mutex::new(conn) }
    }

    #[test]
    fn test_insert_and_get() {
        let db = test_db();
        db.insert("hello world", "openai", "whisper-1", Some("en"), Some(1500)).unwrap();
        let all = db.get_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].text, "hello world");
    }

    #[test]
    fn test_delete() {
        let db = test_db();
        let id = db.insert("test", "openai", "whisper-1", None, None).unwrap();
        db.delete(id).unwrap();
        assert_eq!(db.get_all().unwrap().len(), 0);
    }

    #[test]
    fn test_clear_all() {
        let db = test_db();
        db.insert("one", "openai", "whisper-1", None, None).unwrap();
        db.insert("two", "openai", "whisper-1", None, None).unwrap();
        db.clear_all().unwrap();
        assert_eq!(db.get_all().unwrap().len(), 0);
    }

    #[test]
    fn test_search() {
        let db = test_db();
        db.insert("hello world", "openai", "whisper-1", None, None).unwrap();
        db.insert("goodbye world", "openai", "whisper-1", None, None).unwrap();
        db.insert("foo bar", "openai", "whisper-1", None, None).unwrap();
        let results = db.search("world").unwrap();
        assert_eq!(results.len(), 2);
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cd F:/Projects/tinywhispr/src-tauri
cargo test db -- --nocapture
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: implement SQLite database module for transcription history"
```

### Task 4: Tauri IPC commands (settings + history)

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Implement settings and history commands**

`src-tauri/src/commands.rs`:
```rust
use tauri::State;
use crate::settings::{self, Settings, mask_api_key};
use crate::db::Database;

#[derive(Clone, serde::Serialize)]
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
    let s = settings::load_settings();
    Ok(MaskedSettings::from(&s))
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> Result<MaskedSettings, String> {
    let saved = settings::save_settings_merged(settings)?;
    Ok(MaskedSettings::from(&saved))
}

#[tauri::command]
pub fn get_history(db: State<'_, Database>) -> Result<Vec<crate::db::Transcription>, String> {
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
pub fn search_history(db: State<'_, Database>, query: String) -> Result<Vec<crate::db::Transcription>, String> {
    db.search(&query)
}
```

- [ ] **Step 2: Update `main.rs` to register commands and managed state**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tinywhispr::*;

fn main() {
    let db = tinywhispr::db::Database::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_history,
            delete_transcription,
            clear_history,
            search_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd F:/Projects/tinywhispr/src-tauri
cargo check
```

Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "feat: add Tauri IPC commands for settings and history"
```

---

## Chunk 3: Audio, Transcription, and Output

### Task 5: Audio recording module

**Files:**
- Create: `src-tauri/src/audio.rs`

- [ ] **Step 1: Implement audio recorder struct with cpal**

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};

const MAX_DURATION: Duration = Duration::from_secs(300); // 5 minutes
const TARGET_SAMPLE_RATE: u32 = 16000;

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(TARGET_SAMPLE_RATE)),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }
}
```

- [ ] **Step 2: Implement start_recording**

```rust
impl AudioRecorder {
    pub fn start_recording(&self) -> Result<(), String> {
        if self.is_recording() {
            return Err("Already recording".into());
        }

        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        let config = device.default_input_config()
            .map_err(|e| format!("Failed to get input config: {}", e))?;

        let native_rate = config.sample_rate().0;
        *self.sample_rate.lock().unwrap() = native_rate;

        let samples = self.samples.clone();
        samples.lock().unwrap().clear();

        let is_recording = self.is_recording.clone();
        is_recording.store(true, Ordering::Relaxed);

        let start_time = Instant::now();
        let is_rec_clone = is_recording.clone();

        let stream_config: cpal::StreamConfig = config.into();
        let channels = stream_config.channels as usize;

        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !is_rec_clone.load(Ordering::Relaxed) { return; }
                if start_time.elapsed() >= MAX_DURATION {
                    is_rec_clone.store(false, Ordering::Relaxed);
                    return;
                }
                // Mix to mono
                let mono: Vec<f32> = data.chunks(channels)
                    .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                    .collect();
                samples.lock().unwrap().extend_from_slice(&mono);
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        ).map_err(|e| format!("Failed to build stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to start stream: {}", e))?;

        // Keep stream alive in a background thread
        let is_rec_bg = self.is_recording.clone();
        std::thread::spawn(move || {
            while is_rec_bg.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(50));
            }
            drop(stream);
        });

        Ok(())
    }
}
```

- [ ] **Step 3: Implement stop_recording with resampling and WAV encoding**

```rust
impl AudioRecorder {
    pub fn stop_recording(&self) -> Result<Vec<u8>, String> {
        self.is_recording.store(false, Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(100)); // let stream flush

        let samples = self.samples.lock().unwrap().clone();
        let native_rate = *self.sample_rate.lock().unwrap();

        if samples.is_empty() {
            return Err("No audio captured".into());
        }

        // Resample to 16kHz if needed
        let resampled = if native_rate != TARGET_SAMPLE_RATE {
            resample(&samples, native_rate, TARGET_SAMPLE_RATE)?
        } else {
            samples
        };

        // Encode to WAV
        encode_wav(&resampled, TARGET_SAMPLE_RATE)
    }

    pub fn duration_ms(&self) -> u64 {
        let samples = self.samples.lock().unwrap();
        let rate = *self.sample_rate.lock().unwrap();
        if rate == 0 { return 0; }
        (samples.len() as u64 * 1000) / rate as u64
    }
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>, String> {
    use rubato::{SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction, Resampler};

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let chunk_size = 1024;
    let mut resampler = SincFixedIn::<f32>::new(
        to_rate as f64 / from_rate as f64,
        2.0,
        params,
        chunk_size,
        1,
    ).map_err(|e| format!("Resampler init failed: {}", e))?;

    let mut output_samples = Vec::new();
    for chunk in samples.chunks(chunk_size) {
        // Pad last chunk if needed
        let mut input_chunk = chunk.to_vec();
        if input_chunk.len() < chunk_size {
            input_chunk.resize(chunk_size, 0.0);
        }
        let input = vec![input_chunk];
        let result = resampler.process(&input, None)
            .map_err(|e| format!("Resampling failed: {}", e))?;
        if let Some(channel) = result.into_iter().next() {
            output_samples.extend_from_slice(&channel);
        }
    }

    Ok(output_samples)
}

fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(&mut cursor, spec)
        .map_err(|e| format!("WAV writer init failed: {}", e))?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        writer.write_sample(int_sample).map_err(|e| format!("WAV write failed: {}", e))?;
    }
    writer.finalize().map_err(|e| format!("WAV finalize failed: {}", e))?;

    Ok(cursor.into_inner())
}
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio.rs
git commit -m "feat: implement audio recording with cpal, resampling, and WAV encoding"
```

### Task 6: Transcription API client

**Files:**
- Create: `src-tauri/src/transcribe.rs`

- [ ] **Step 1: Implement the transcription function**

```rust
use reqwest::multipart;

pub struct TranscriptionResult {
    pub text: String,
}

pub async fn transcribe(
    base_url: &str,
    api_key: &str,
    model: &str,
    language: &str,
    audio_data: Vec<u8>,
) -> Result<TranscriptionResult, TranscriptionError> {
    let url = format!("{}/audio/transcriptions", base_url.trim_end_matches('/'));

    let file_part = multipart::Part::bytes(audio_data)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| TranscriptionError::Request(e.to_string()))?;

    let mut form = multipart::Form::new()
        .part("file", file_part)
        .text("model", model.to_string())
        .text("response_format", "json".to_string());

    if language != "auto" && !language.is_empty() {
        form = form.text("language", language.to_string());
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| TranscriptionError::Network(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(match status.as_u16() {
            401 | 403 => TranscriptionError::InvalidApiKey,
            429 => TranscriptionError::RateLimited,
            _ => TranscriptionError::ApiError(format!("{}: {}", status, body)),
        });
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| TranscriptionError::Request(e.to_string()))?;

    let text = json["text"].as_str().unwrap_or("").to_string();
    if text.is_empty() {
        return Err(TranscriptionError::EmptyResult);
    }

    Ok(TranscriptionResult { text })
}

#[derive(Debug)]
pub enum TranscriptionError {
    Network(String),
    InvalidApiKey,
    RateLimited,
    ApiError(String),
    EmptyResult,
    Request(String),
}

impl std::fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Transcription failed: network error ({})", e),
            Self::InvalidApiKey => write!(f, "Invalid API key"),
            Self::RateLimited => write!(f, "Rate limit exceeded. Try again."),
            Self::ApiError(e) => write!(f, "{}", e),
            Self::EmptyResult => write!(f, "No speech detected"),
            Self::Request(e) => write!(f, "Request error: {}", e),
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/transcribe.rs
git commit -m "feat: implement transcription API client with error handling"
```

### Task 7: Output module (clipboard + auto-paste)

**Files:**
- Create: `src-tauri/src/output.rs`

- [ ] **Step 1: Implement clipboard and auto-paste output**

```rust
use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Settings as EnigoSettings, Key, Direction};
use std::thread;
use std::time::Duration;

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())
}

pub fn auto_paste(text: &str) -> Result<(), String> {
    copy_to_clipboard(text)?;

    // Brief delay to ensure clipboard is ready
    thread::sleep(Duration::from_millis(50));

    let mut enigo = Enigo::new(&EnigoSettings::default())
        .map_err(|e| e.to_string())?;

    // Simulate Ctrl+V
    enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn output_text(text: &str, mode: &str) -> Result<(), String> {
    match mode {
        "auto_paste" => auto_paste(text),
        "clipboard" => copy_to_clipboard(text),
        _ => copy_to_clipboard(text),
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/output.rs
git commit -m "feat: implement output module with clipboard and auto-paste"
```

---

## Chunk 4: Hotkey, Tray, and Indicator

### Task 8: Global hotkey management

**Files:**
- Create: `src-tauri/src/hotkey.rs`

- [ ] **Step 1: Implement hotkey registration for toggle mode**

```rust
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct HotkeyManager {
    current_shortcut: std::sync::Mutex<Option<String>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            current_shortcut: std::sync::Mutex::new(None),
        }
    }

    pub fn register<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        hotkey: &str,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> Result<(), String> {
        // Unregister previous shortcut if any
        self.unregister(app)?;

        let shortcut: Shortcut = hotkey.parse()
            .map_err(|e| format!("Invalid hotkey '{}': {}", hotkey, e))?;

        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
            callback();
        }).map_err(|e| format!("Failed to register hotkey: {}", e))?;

        *self.current_shortcut.lock().unwrap() = Some(hotkey.to_string());
        Ok(())
    }

    pub fn unregister<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        if let Some(ref key) = *self.current_shortcut.lock().unwrap() {
            if let Ok(shortcut) = key.parse::<Shortcut>() {
                app.global_shortcut().unregister(shortcut).ok();
            }
        }
        *self.current_shortcut.lock().unwrap() = None;
        Ok(())
    }
}
```

- [ ] **Step 2: Add push-to-talk key-up detection via Windows keyboard hook**

```rust
#[cfg(target_os = "windows")]
pub mod ptt {
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Foundation::*;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex, OnceLock};

    static KEY_UP_CALLBACK: OnceLock<Mutex<Option<Box<dyn Fn() + Send + Sync>>>> = OnceLock::new();
    static TARGET_VKEY: OnceLock<Mutex<u32>> = OnceLock::new();

    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 && wparam.0 as u32 == WM_KEYUP {
            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let target = TARGET_VKEY.get().and_then(|m| m.lock().ok()).map(|v| *v).unwrap_or(0);
            if kb.vkCode == target {
                if let Some(cb_mutex) = KEY_UP_CALLBACK.get() {
                    if let Ok(cb_lock) = cb_mutex.lock() {
                        if let Some(ref cb) = *cb_lock {
                            cb();
                        }
                    }
                }
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    pub fn install_key_up_hook(vkey: u32, callback: impl Fn() + Send + Sync + 'static) -> Result<(), String> {
        KEY_UP_CALLBACK.get_or_init(|| Mutex::new(None));
        TARGET_VKEY.get_or_init(|| Mutex::new(0));

        *KEY_UP_CALLBACK.get().unwrap().lock().unwrap() = Some(Box::new(callback));
        *TARGET_VKEY.get().unwrap().lock().unwrap() = vkey;

        std::thread::spawn(|| {
            unsafe {
                let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0)
                    .expect("Failed to set keyboard hook");

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).into() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                UnhookWindowsHookEx(hook).ok();
            }
        });

        Ok(())
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/hotkey.rs
git commit -m "feat: implement hotkey manager with toggle and push-to-talk support"
```

### Task 9: System tray

**Files:**
- Create: `src-tauri/src/tray.rs`

- [ ] **Step 1: Implement tray icon with context menu**

```rust
use tauri::{
    AppHandle, Runtime,
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Manager, Emitter,
};
use std::sync::{Arc, Mutex};

pub enum TrayState {
    Idle,
    Recording,
    Processing,
}

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let toggle_item = MenuItem::with_id(app, "toggle", "Start Recording", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings_item = MenuItem::with_id(app, "settings", "Open Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let history_item = MenuItem::with_id(app, "history", "Open History", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(app, &[
        &toggle_item,
        &separator,
        &settings_item,
        &history_item,
        &PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?,
        &quit_item,
    ]).map_err(|e| e.to_string())?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("TinyWhispr")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "toggle" => { app.emit("tray-toggle-recording", ()).ok(); }
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                        app.emit("navigate", "settings").ok();
                    }
                }
                "history" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                        app.emit("navigate", "history").ok();
                    }
                }
                "quit" => { app.exit(0); }
                _ => {}
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat: implement system tray with context menu"
```

### Task 10: Floating indicator window

**Files:**
- Modify: `src/indicator/IndicatorApp.svelte`

- [ ] **Step 1: Implement the indicator UI with recording/processing states**

`src/indicator/IndicatorApp.svelte`:
```svelte
<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let state = $state<"hidden" | "recording" | "processing">("hidden");

  const appWindow = getCurrentWindow();

  listen("recording-started", () => {
    state = "recording";
    appWindow.show();
  });

  listen("recording-stopped", () => {
    state = "processing";
  });

  listen("transcription-complete", () => {
    state = "hidden";
    appWindow.hide();
  });

  listen("transcription-error", () => {
    state = "hidden";
    appWindow.hide();
  });
</script>

{#if state !== "hidden"}
<div class="indicator" class:recording={state === "recording"} class:processing={state === "processing"}>
  {#if state === "recording"}
    <span class="dot recording-dot"></span>
    <span class="label">REC</span>
  {:else}
    <span class="spinner"></span>
    <span class="label">...</span>
  {/if}
</div>
{/if}

<style>
  .indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border-radius: 20px;
    font-family: system-ui, sans-serif;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.5px;
    backdrop-filter: blur(8px);
    user-select: none;
  }

  .recording {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #fca5a5;
  }

  .processing {
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid rgba(59, 130, 246, 0.4);
    color: #93c5fd;
  }

  .recording-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ef4444;
    box-shadow: 0 0 8px rgba(239, 68, 68, 0.6);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 2px solid transparent;
    border-top-color: #3b82f6;
    animation: spin 0.8s linear infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/indicator/IndicatorApp.svelte
git commit -m "feat: implement floating indicator with recording/processing states"
```

---

## Chunk 5: Frontend UI

### Task 11: TypeScript types and stores

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/stores/settings.ts`
- Create: `src/lib/stores/history.ts`
- Create: `src/lib/stores/recording.ts`

- [ ] **Step 1: Create TypeScript types**

`src/lib/types.ts`:
```typescript
export interface Settings {
  provider: string;
  api_key: string;
  base_url: string;
  model: string;
  language: string;
  hotkey: string;
  activation_mode: string;
  output_mode: string;
  launch_at_startup: boolean;
}

export interface Transcription {
  id: number;
  text: string;
  provider: string;
  model: string;
  language: string | null;
  duration_ms: number | null;
  created_at: string;
}

export interface ProviderPreset {
  name: string;
  base_url: string;
  models: string[];
}

export const PROVIDER_PRESETS: Record<string, ProviderPreset> = {
  openai: {
    name: "OpenAI",
    base_url: "https://api.openai.com/v1",
    models: ["whisper-1", "gpt-4o-mini-transcribe", "gpt-4o-transcribe"],
  },
  groq: {
    name: "Groq",
    base_url: "https://api.groq.com/openai/v1",
    models: ["whisper-large-v3", "whisper-large-v3-turbo"],
  },
  custom: {
    name: "Custom",
    base_url: "",
    models: [],
  },
};

export type RecordingState = "idle" | "recording" | "processing";
```

- [ ] **Step 2: Create settings store**

`src/lib/stores/settings.ts`:
```typescript
import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";

export const settings = writable<Settings | null>(null);
export const settingsError = writable<string | null>(null);

export async function loadSettings() {
  try {
    const s = await invoke<Settings>("get_settings");
    settings.set(s);
    settingsError.set(null);
  } catch (e) {
    settingsError.set(String(e));
  }
}

export async function saveSettings(s: Settings) {
  try {
    const saved = await invoke<Settings>("save_settings", { settings: s });
    settings.set(saved);
    settingsError.set(null);
  } catch (e) {
    settingsError.set(String(e));
    throw e;
  }
}
```

- [ ] **Step 3: Create history store**

`src/lib/stores/history.ts`:
```typescript
import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Transcription } from "../types";

export const history = writable<Transcription[]>([]);

export async function loadHistory() {
  const items = await invoke<Transcription[]>("get_history");
  history.set(items);
}

export async function deleteTranscription(id: number) {
  await invoke("delete_transcription", { id });
  await loadHistory();
}

export async function clearHistory() {
  await invoke("clear_history");
  history.set([]);
}

export async function searchHistory(query: string) {
  if (!query.trim()) {
    await loadHistory();
    return;
  }
  const items = await invoke<Transcription[]>("search_history", { query });
  history.set(items);
}
```

- [ ] **Step 4: Create recording store**

`src/lib/stores/recording.ts`:
```typescript
import { writable } from "svelte/store";
import type { RecordingState } from "../types";

export const recordingState = writable<RecordingState>("idle");
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/
git commit -m "feat: add TypeScript types, provider presets, and Svelte stores"
```

### Task 12: TabBar component

**Files:**
- Create: `src/lib/components/TabBar.svelte`

- [ ] **Step 1: Implement tab bar**

```svelte
<script lang="ts">
  let { activeTab = $bindable("settings") }: { activeTab: string } = $props();
</script>

<div class="flex border-b border-border" data-tauri-drag-region>
  <button
    class="tab"
    class:active={activeTab === "settings"}
    onclick={() => activeTab = "settings"}
  >
    Settings
  </button>
  <button
    class="tab"
    class:active={activeTab === "history"}
    onclick={() => activeTab = "history"}
  >
    History
  </button>
  <!-- Drag region fills remaining space -->
  <div class="flex-1" data-tauri-drag-region></div>
  <!-- Window close button -->
  <button
    class="px-3 py-2 text-text-muted hover:text-text-primary hover:bg-surface transition-colors"
    onclick={async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      getCurrentWindow().hide();
    }}
  >
    ✕
  </button>
</div>

<style>
  .tab {
    padding: 10px 20px;
    font-size: 13px;
    color: #737373;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }
  .tab:hover {
    color: #a3a3a3;
  }
  .tab.active {
    color: #3b82f6;
    font-weight: 600;
    border-bottom-color: #3b82f6;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/TabBar.svelte
git commit -m "feat: add TabBar component with draggable title region"
```

### Task 13: Settings component

**Files:**
- Create: `src/lib/components/Settings.svelte`

- [ ] **Step 1: Implement settings form**

```svelte
<script lang="ts">
  import { settings, saveSettings, loadSettings } from "../stores/settings";
  import { PROVIDER_PRESETS, type Settings } from "../types";
  import { onMount } from "svelte";

  let form = $state<Settings | null>(null);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let hotkeyCapture = $state(false);

  onMount(() => {
    loadSettings();
  });

  $effect(() => {
    if ($settings) {
      form = { ...$settings };
    }
  });

  function onProviderChange() {
    if (!form) return;
    const preset = PROVIDER_PRESETS[form.provider];
    if (preset && form.provider !== "custom") {
      form.base_url = preset.base_url;
      form.model = preset.models[0] || "";
    }
  }

  async function handleSave() {
    if (!form) return;
    saving = true;
    error = null;
    try {
      await saveSettings(form);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function handleHotkeyCapture(e: KeyboardEvent) {
    e.preventDefault();
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.key !== "Control" && e.key !== "Alt" && e.key !== "Shift") {
      parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
      if (form && parts.length >= 2) {
        form.hotkey = parts.join("+");
      }
      hotkeyCapture = false;
    }
  }
</script>

{#if form}
<div class="p-4 flex flex-col gap-4 overflow-y-auto h-full">
  {#if !form.api_key || form.api_key === ""}
    <div class="bg-recording/10 border border-recording/30 rounded-md p-3 text-recording text-xs">
      Set an API key to enable transcription.
    </div>
  {/if}

  {#if error}
    <div class="bg-recording/10 border border-recording/30 rounded-md p-3 text-recording text-xs">
      {error}
    </div>
  {/if}

  <!-- Provider -->
  <div>
    <label class="label">Transcription Provider</label>
    <select bind:value={form.provider} onchange={onProviderChange} class="input">
      <option value="openai">OpenAI</option>
      <option value="groq">Groq</option>
      <option value="custom">Custom</option>
    </select>
  </div>

  <!-- API Key -->
  <div>
    <label class="label">API Key</label>
    <input
      type="password"
      bind:value={form.api_key}
      class="input"
      placeholder="Enter API key..."
    />
  </div>

  <!-- Base URL -->
  <div>
    <label class="label">Base URL</label>
    <input
      type="text"
      bind:value={form.base_url}
      class="input"
      disabled={form.provider !== "custom"}
    />
  </div>

  <!-- Model -->
  <div>
    <label class="label">Model</label>
    {#if form.provider !== "custom" && PROVIDER_PRESETS[form.provider]}
      <select bind:value={form.model} class="input">
        {#each PROVIDER_PRESETS[form.provider].models as model}
          <option value={model}>{model}</option>
        {/each}
      </select>
    {:else}
      <input type="text" bind:value={form.model} class="input" placeholder="Model name" />
    {/if}
  </div>

  <!-- Hotkey + Activation Mode -->
  <div class="flex gap-3">
    <div class="flex-1">
      <label class="label">Hotkey</label>
      <button
        class="input text-center w-full"
        class:border-accent={hotkeyCapture}
        onclick={() => hotkeyCapture = true}
        onkeydown={hotkeyCapture ? handleHotkeyCapture : undefined}
      >
        {hotkeyCapture ? "Press a key combo..." : form.hotkey}
      </button>
    </div>
    <div class="flex-1">
      <label class="label">Mode</label>
      <select bind:value={form.activation_mode} class="input">
        <option value="toggle">Toggle</option>
        <option value="push_to_talk">Push to Talk</option>
      </select>
    </div>
  </div>

  <!-- Output Mode -->
  <div>
    <label class="label">After Transcription</label>
    <select bind:value={form.output_mode} class="input">
      <option value="auto_paste">Auto-paste at cursor</option>
      <option value="clipboard">Copy to clipboard</option>
    </select>
  </div>

  <!-- Language -->
  <div>
    <label class="label">Language</label>
    <select bind:value={form.language} class="input">
      <option value="auto">Auto-detect</option>
      <option value="en">English</option>
      <option value="es">Spanish</option>
      <option value="fr">French</option>
      <option value="de">German</option>
      <option value="pt">Portuguese</option>
      <option value="it">Italian</option>
      <option value="ja">Japanese</option>
      <option value="ru">Russian</option>
      <option value="zh">Chinese</option>
    </select>
  </div>

  <!-- Launch at startup -->
  <div class="flex items-center justify-between">
    <label class="label mb-0">Launch at startup</label>
    <input type="checkbox" bind:checked={form.launch_at_startup} class="accent-accent" />
  </div>

  <!-- Save button -->
  <button
    class="mt-2 bg-accent hover:bg-accent/80 text-white rounded-md py-2 text-sm font-medium transition-colors disabled:opacity-50"
    onclick={handleSave}
    disabled={saving}
  >
    {saving ? "Saving..." : "Save Settings"}
  </button>
</div>
{/if}

<style>
  .label {
    display: block;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #737373;
    margin-bottom: 6px;
  }
  .input {
    width: 100%;
    background: #1a1a1a;
    border: 1px solid #262626;
    border-radius: 6px;
    padding: 8px 12px;
    color: #e5e5e5;
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s;
    appearance: none;
    box-sizing: border-box;
  }
  .input:focus {
    border-color: #3b82f6;
  }
  select.input {
    cursor: pointer;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/Settings.svelte
git commit -m "feat: implement Settings component with provider presets and hotkey capture"
```

### Task 14: History components

**Files:**
- Create: `src/lib/components/HistoryItem.svelte`
- Create: `src/lib/components/History.svelte`

- [ ] **Step 1: Create HistoryItem component**

`src/lib/components/HistoryItem.svelte`:
```svelte
<script lang="ts">
  import type { Transcription } from "../types";

  let { item, onDelete, onCopy }: {
    item: Transcription;
    onDelete: (id: number) => void;
    onCopy: (text: string) => void;
  } = $props();

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr + "Z");
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today.getTime() - 86400000);
    const itemDate = new Date(date.getFullYear(), date.getMonth(), date.getDate());

    const time = date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });

    if (itemDate.getTime() === today.getTime()) return `Today, ${time}`;
    if (itemDate.getTime() === yesterday.getTime()) return `Yesterday, ${time}`;
    return `${date.toLocaleDateString()}, ${time}`;
  }
</script>

<div class="border-b border-border/50 py-3">
  <div class="flex justify-between items-start mb-1">
    <span class="text-[11px] text-text-secondary">{formatDate(item.created_at)}</span>
    <div class="flex gap-3">
      <button class="text-[11px] text-text-muted hover:text-text-primary transition-colors" onclick={() => onCopy(item.text)}>
        Copy
      </button>
      <button class="text-[11px] text-text-muted hover:text-recording transition-colors" onclick={() => onDelete(item.id)}>
        Delete
      </button>
    </div>
  </div>
  <p class="text-[13px] text-text-primary leading-relaxed line-clamp-2">{item.text}</p>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
```

- [ ] **Step 2: Create History component**

`src/lib/components/History.svelte`:
```svelte
<script lang="ts">
  import { history, loadHistory, deleteTranscription, clearHistory, searchHistory } from "../stores/history";
  import HistoryItem from "./HistoryItem.svelte";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  let searchQuery = $state("");
  let confirmClear = $state(false);

  onMount(() => {
    loadHistory();
    listen("transcription-complete", () => loadHistory());
  });

  async function handleSearch() {
    await searchHistory(searchQuery);
  }

  async function handleCopy(text: string) {
    await navigator.clipboard.writeText(text);
  }

  async function handleClearAll() {
    if (confirmClear) {
      await clearHistory();
      confirmClear = false;
    } else {
      confirmClear = true;
      setTimeout(() => confirmClear = false, 3000);
    }
  }
</script>

<div class="flex flex-col h-full">
  <!-- Search bar -->
  <div class="p-3 pb-2">
    <input
      type="text"
      bind:value={searchQuery}
      oninput={handleSearch}
      placeholder="Search transcriptions..."
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-xs text-text-primary placeholder:text-text-muted outline-none focus:border-accent transition-colors"
    />
  </div>

  <!-- History list -->
  <div class="flex-1 overflow-y-auto px-4">
    {#if $history.length === 0}
      <div class="flex items-center justify-center h-32 text-text-muted text-xs">
        {searchQuery ? "No matching transcriptions." : "No transcriptions yet."}
      </div>
    {:else}
      {#each $history as item (item.id)}
        <HistoryItem
          {item}
          onDelete={deleteTranscription}
          onCopy={handleCopy}
        />
      {/each}
    {/if}
  </div>

  <!-- Clear all button -->
  {#if $history.length > 0 && !searchQuery}
    <div class="p-3 border-t border-border">
      <button
        class="w-full py-2 text-xs rounded-md transition-colors"
        class:bg-recording/10={confirmClear}
        class:text-recording={confirmClear}
        class:text-text-muted={!confirmClear}
        class:hover:text-text-secondary={!confirmClear}
        onclick={handleClearAll}
      >
        {confirmClear ? "Click again to confirm" : "Clear All History"}
      </button>
    </div>
  {/if}
</div>
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/HistoryItem.svelte src/lib/components/History.svelte
git commit -m "feat: implement History and HistoryItem components"
```

### Task 15: Main App.svelte

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Wire up App.svelte with tabs**

```svelte
<script lang="ts">
  import TabBar from "./lib/components/TabBar.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import History from "./lib/components/History.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let activeTab = $state("settings");

  onMount(() => {
    listen<string>("navigate", (event) => {
      activeTab = event.payload;
    });

    // Hide on close instead of quit
    getCurrentWindow().onCloseRequested(async (e) => {
      e.preventDefault();
      await getCurrentWindow().hide();
    });
  });
</script>

<main class="bg-background text-text-primary h-screen flex flex-col font-sans text-[13px]">
  <TabBar bind:activeTab />
  <div class="flex-1 overflow-hidden">
    {#if activeTab === "settings"}
      <Settings />
    {:else}
      <History />
    {/if}
  </div>
</main>
```

- [ ] **Step 2: Commit**

```bash
git add src/App.svelte
git commit -m "feat: wire up main App with tab navigation and window hide-on-close"
```

---

## Chunk 6: Integration and End-to-End Wiring

### Task 16: Wire the full recording flow in main.rs

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: Add recording state and commands**

Add to `src-tauri/src/commands.rs`:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use crate::audio::AudioRecorder;
use crate::transcribe;
use crate::output;

#[tauri::command]
pub async fn start_recording(
    recorder: State<'_, AudioRecorder>,
    app: AppHandle,
) -> Result<(), String> {
    recorder.start_recording()?;
    app.emit("recording-started", ()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(
    recorder: State<'_, AudioRecorder>,
    db: State<'_, Database>,
    app: AppHandle,
) -> Result<String, String> {
    let duration_ms = recorder.duration_ms();
    let wav_data = recorder.stop_recording()?;
    app.emit("recording-stopped", ()).map_err(|e| e.to_string())?;

    let settings = crate::settings::load_settings();

    if settings.api_key.is_empty() {
        app.emit("transcription-error", "No API key configured").ok();
        return Err("No API key configured".into());
    }

    let result = transcribe::transcribe(
        &settings.base_url,
        &settings.api_key,
        &settings.model,
        &settings.language,
        wav_data,
    ).await;

    match result {
        Ok(transcription) => {
            // Save to history
            db.insert(
                &transcription.text,
                &settings.provider,
                &settings.model,
                if settings.language == "auto" { None } else { Some(&settings.language) },
                Some(duration_ms as i64),
            ).ok();

            // Output the text
            output::output_text(&transcription.text, &settings.output_mode)?;

            app.emit("transcription-complete", &transcription.text).map_err(|e| e.to_string())?;
            Ok(transcription.text)
        }
        Err(e) => {
            let msg = e.to_string();
            app.emit("transcription-error", &msg).ok();

            // Send notification
            use tauri_plugin_notification::NotificationExt;
            app.notification()
                .builder()
                .title("TinyWhispr")
                .body(&msg)
                .show()
                .ok();

            Err(msg)
        }
    }
}

#[tauri::command]
pub fn get_recording_state(recorder: State<'_, AudioRecorder>) -> String {
    if recorder.is_recording() {
        "recording".into()
    } else {
        "idle".into()
    }
}
```

- [ ] **Step 2: Update main.rs with full initialization**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let db = tinywhispr::db::Database::new().expect("Failed to initialize database");
    let recorder = tinywhispr::audio::AudioRecorder::new();
    let hotkey_manager = tinywhispr::hotkey::HotkeyManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(db)
        .manage(recorder)
        .manage(hotkey_manager)
        .invoke_handler(tauri::generate_handler![
            tinywhispr::get_settings,
            tinywhispr::save_settings,
            tinywhispr::get_history,
            tinywhispr::delete_transcription,
            tinywhispr::clear_history,
            tinywhispr::search_history,
            tinywhispr::start_recording,
            tinywhispr::stop_recording,
            tinywhispr::get_recording_state,
        ])
        .setup(|app| {
            // Create tray
            tinywhispr::tray::create_tray(&app.handle()).map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>)?;

            // Register hotkey
            let settings = tinywhispr::settings::load_settings();
            let handle = app.handle().clone();
            let hotkey_mgr: tauri::State<tinywhispr::hotkey::HotkeyManager> = app.state();

            hotkey_mgr.register(&app.handle(), &settings.hotkey, move || {
                let h = handle.clone();
                h.emit("tray-toggle-recording", ()).ok();
            }).ok();

            // Set up indicator window click-through
            if let Some(indicator) = app.get_webview_window("indicator") {
                indicator.set_ignore_cursor_events(true).ok();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide main window on close instead of quitting
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    window.hide().ok();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Wire the tray toggle-recording event to actual recording logic**

The frontend listens for `tray-toggle-recording` events and calls `start_recording`/`stop_recording` commands. Add a listener in `src/App.svelte`:

Add to `App.svelte` `onMount`:
```typescript
import { invoke } from "@tauri-apps/api/core";
import { recordingState } from "./lib/stores/recording";

listen("tray-toggle-recording", async () => {
  const state = await invoke<string>("get_recording_state");
  if (state === "recording") {
    recordingState.set("processing");
    try {
      await invoke("stop_recording");
    } catch (e) {
      console.error("Stop recording failed:", e);
    }
    recordingState.set("idle");
  } else {
    try {
      await invoke("start_recording");
      recordingState.set("recording");
    } catch (e) {
      console.error("Start recording failed:", e);
    }
  }
});
```

- [ ] **Step 4: Verify it compiles**

```bash
cd F:/Projects/tinywhispr/src-tauri
cargo check
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: wire full recording flow end-to-end"
```

### Task 16b: Fix feature gaps

This task addresses reviewer-identified gaps between the plan and the spec.

- [ ] **Step 1: Add tray icon state changes**

Update `src-tauri/src/tray.rs` to add a function that swaps the tray icon based on state:

```rust
pub fn update_tray_icon<R: Runtime>(app: &AppHandle<R>, state: TrayState) {
    let icon_bytes = match state {
        TrayState::Idle => include_bytes!("../icons/icon.ico"),
        TrayState::Recording => include_bytes!("../icons/icon-recording.ico"),
        TrayState::Processing => include_bytes!("../icons/icon-processing.ico"),
    };
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(icon) = tauri::image::Image::from_bytes(icon_bytes) {
            tray.set_icon(Some(icon)).ok();
        }
    }
}

pub fn update_tray_menu_text<R: Runtime>(app: &AppHandle<R>, recording: bool) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Some(menu) = tray.menu() {
            if let Some(item) = menu.get("toggle") {
                if let Some(menu_item) = item.as_menuitem() {
                    let text = if recording { "Stop Recording" } else { "Start Recording" };
                    menu_item.set_text(text).ok();
                }
            }
        }
    }
}
```

Call `update_tray_icon` and `update_tray_menu_text` from the recording flow in `stop_recording` and `start_recording` commands.

- [ ] **Step 2: Add notification for clipboard mode**

Update `src-tauri/src/output.rs` to accept an `AppHandle` and send a notification when in clipboard mode:

```rust
pub fn output_text<R: Runtime>(app: &AppHandle<R>, text: &str, mode: &str) -> Result<(), String> {
    match mode {
        "auto_paste" => auto_paste(text),
        "clipboard" | _ => {
            copy_to_clipboard(text)?;
            use tauri_plugin_notification::NotificationExt;
            app.notification()
                .builder()
                .title("TinyWhispr")
                .body("Transcription copied to clipboard")
                .show()
                .ok();
            Ok(())
        }
    }
}
```

- [ ] **Step 3: Handle max recording duration auto-stop**

In the audio recorder, when the 5-minute timeout fires, emit an event to trigger the stop flow. Update `start_recording` to accept a callback that gets called on timeout:

```rust
// In the recording thread, after is_recording goes false due to timeout:
// Call the on_timeout callback which should trigger stop_recording
let timeout_flag = is_recording.clone();
let on_timeout = Arc::new(Mutex::new(Some(Box::new(move || {
    // This will be set by the caller to trigger stop_recording
}) as Box<dyn Fn() + Send>)));
```

The `main.rs` setup should pass a closure that emits `"tray-toggle-recording"` when the timeout fires, which triggers the normal stop flow.

- [ ] **Step 4: Wire push-to-talk mode**

In `main.rs` setup, when registering the hotkey callback, check `activation_mode`. If `push_to_talk`, after `start_recording`, install the key-up hook that calls `stop_recording`:

```rust
// In the hotkey callback:
let settings = tinywhispr::settings::load_settings();
if settings.activation_mode == "push_to_talk" {
    // Start recording
    app.emit("tray-toggle-recording", ()).ok();
    // Install key-up hook to stop
    let handle_clone = handle.clone();
    tinywhispr::hotkey::ptt::install_key_up_hook(vkey, move || {
        handle_clone.emit("tray-toggle-recording", ()).ok();
    }).ok();
}
```

- [ ] **Step 5: Re-register hotkey and autostart on settings save**

Update the `save_settings` command in `commands.rs` to accept `HotkeyManager` state and `AppHandle`, and re-register the hotkey if it changed, and toggle autostart:

```rust
#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    hotkey_mgr: State<'_, HotkeyManager>,
    app: AppHandle,
) -> Result<MaskedSettings, String> {
    let old = crate::settings::load_settings();
    let saved = crate::settings::save_settings_merged(settings)?;

    if saved.hotkey != old.hotkey {
        let handle = app.clone();
        hotkey_mgr.register(&app, &saved.hotkey, move || {
            handle.emit("tray-toggle-recording", ()).ok();
        })?;
    }

    if saved.launch_at_startup != old.launch_at_startup {
        use tauri_plugin_autostart::ManagerExt;
        let autostart = app.autolaunch();
        if saved.launch_at_startup {
            autostart.enable().map_err(|e| e.to_string())?;
        } else {
            autostart.disable().map_err(|e| e.to_string())?;
        }
    }

    Ok(MaskedSettings::from(&saved))
}
```

- [ ] **Step 6: Disable recording when no API key is set**

In `start_recording`, check for API key before starting:

```rust
#[tauri::command]
pub async fn start_recording(
    recorder: State<'_, AudioRecorder>,
    app: AppHandle,
) -> Result<(), String> {
    let settings = crate::settings::load_settings();
    if settings.api_key.is_empty() {
        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title("TinyWhispr")
            .body("Set an API key in Settings first")
            .show()
            .ok();
        return Err("No API key configured".into());
    }
    recorder.start_recording()?;
    app.emit("recording-started", ()).map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 7: Add fade animation to indicator**

Update `IndicatorApp.svelte` to use CSS transitions instead of `{#if}`:

```svelte
<div
  class="indicator"
  class:recording={state === "recording"}
  class:processing={state === "processing"}
  class:visible={state !== "hidden"}
>
  ...
</div>

<style>
  .indicator {
    opacity: 0;
    transform: translateY(-4px);
    transition: opacity 0.2s ease, transform 0.2s ease;
    pointer-events: none;
  }
  .indicator.visible {
    opacity: 1;
    transform: translateY(0);
  }
</style>
```

- [ ] **Step 8: Use arboard for copy in History component instead of navigator.clipboard**

In `History.svelte`, replace `navigator.clipboard.writeText` with a Tauri command:

```typescript
async function handleCopy(text: string) {
  await invoke("copy_to_clipboard", { text });
}
```

Add a corresponding command in `commands.rs`:
```rust
#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    crate::output::copy_to_clipboard(&text)
}
```

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "fix: address review feedback — tray states, PTT wiring, notifications, fade animation"
```

### Task 17: Final build and manual verification

- [ ] **Step 1: Build the app**

```bash
cd F:/Projects/tinywhispr
npm run tauri build -- --debug
```

Expected: Build succeeds.

- [ ] **Step 2: Manual verification checklist**

Run the debug build and verify:
1. App starts in system tray with no visible window
2. Right-clicking tray icon shows context menu (Start Recording, Open Settings, Open History, Quit)
3. "Open Settings" shows the main window with Settings tab
4. "Open History" shows the main window with History tab
5. Closing the window hides it (app stays in tray)
6. Pressing the configured hotkey starts recording (indicator appears)
7. Pressing the hotkey again stops recording and sends to API
8. Transcription result is pasted/copied based on output mode setting
9. Transcription appears in history
10. Search filters history entries
11. Delete removes individual entries
12. Clear All removes all entries with confirmation
13. "Quit" from tray exits the app

- [ ] **Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: address issues found during manual verification"
```

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: TinyWhispr v1 complete"
```
