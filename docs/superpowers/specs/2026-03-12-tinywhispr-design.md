# TinyWhispr — Design Specification

## Overview

TinyWhispr is a minimalistic speech-to-text desktop app for Windows that lives in the system tray. Users press a global hotkey or click the tray icon to record audio, which is sent to a cloud transcription API and the resulting text is either auto-pasted at the cursor or copied to the clipboard. A searchable history of all transcriptions is available.

## Goals

- **Tiny** — small binary (~5-15MB), minimal resource usage, fast startup
- **Minimalistic** — no feature creep; dictation, settings, and history only
- **Polished** — dark UI, smooth animations, clear visual feedback during recording
- **Configurable** — supports multiple OpenAI-compatible transcription endpoints (OpenAI, Groq, or any custom endpoint)

## Non-Goals (v1)

- Local/offline transcription (Whisper.cpp, NVIDIA Parakeet)
- Streaming/real-time transcription (WebSocket providers)
- AI post-processing or reasoning
- Meeting transcription
- Notes system
- Custom dictionary
- Cross-platform (Windows only for v1)
- OpenWhispr Cloud integration

## Tech Stack

| Layer | Technology |
|-------|-----------|
| App framework | Tauri v2 |
| Backend language | Rust |
| Frontend framework | Svelte 5 |
| Styling | Tailwind CSS v4 |
| Audio capture | `cpal` crate |
| Audio encoding | `hound` crate (WAV) |
| Audio resampling | `rubato` crate (for downsampling to 16kHz when device doesn't support it natively) |
| HTTP client | `reqwest` crate |
| Keyboard simulation | `enigo` crate |
| Clipboard | `arboard` crate |
| Database | `rusqlite` (SQLite) |
| Global hotkeys | Tauri's `global-shortcut` plugin |
| System tray | Tauri's `tray-icon` plugin |
| Notifications | Tauri's `notification` plugin |
| Autostart | Tauri's `autostart` plugin |
| Settings storage | JSON file via Tauri's `app_data_dir` |
| Build/bundle | Tauri CLI + NSIS installer |

## Architecture

### Components

```
┌─────────────────────────────────────────────┐
│              Tauri (Rust backend)            │
│                                             │
│  ┌───────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Tray      │  │ Audio    │  │ API      │  │
│  │ Manager   │  │ Recorder │  │ Client   │  │
│  └─────┬─────┘  └────┬─────┘  └────┬─────┘  │
│        │             │             │        │
│  ┌─────┴─────────────┴─────────────┴─────┐  │
│  │         Global Hotkey Manager         │  │
│  │         SQLite Store (history)        │  │
│  │         Settings Manager (JSON)       │  │
│  └───────────────────────────────────────┘  │
│                    ▲ IPC (Tauri commands)   │
├────────────────────┼────────────────────────┤
│              Webview (Frontend)             │
│  ┌─────────────────┴─────────────────────┐  │
│  │  Svelte + Tailwind                    │  │
│  │  ├── Settings View                    │  │
│  │  └── History View                     │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  ┌───────────────────────────────────────┐  │
│  │  Floating Indicator (2nd window)      │  │
│  │  Frameless, transparent, always-on-top│  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### Rust Backend Modules

1. **`tray.rs`** — System tray icon and context menu. Menu items: Start/Stop Recording, Open Settings, Open History, separator, Quit. The tray icon changes state: idle (default icon), recording (red-tinted icon), processing (blue-tinted icon). Icons are `.ico` format stored in `src-tauri/icons/`.

2. **`audio.rs`** — Audio capture via `cpal`. Records from the default input device at the device's native sample rate. If the native sample rate is not 16kHz, uses `rubato` to downsample to 16kHz before encoding. Outputs WAV data (16-bit PCM, mono, 16kHz) suitable for the Whisper API. Enforces a maximum recording duration of 5 minutes — if reached, automatically stops recording and proceeds to transcription. Handles device enumeration if needed.

3. **`transcribe.rs`** — HTTP client that sends audio to the configured OpenAI-compatible endpoint. The request URL is constructed as `{base_url}/audio/transcriptions` where `base_url` includes the version path (e.g., `https://api.openai.com/v1`). Sends as `multipart/form-data` with fields: `file` (audio blob), `model` (string), `language` (optional), `response_format` ("json"). Returns transcribed text.

4. **`hotkey.rs`** — Registers and manages global hotkeys via Tauri's global-shortcut plugin. In **toggle mode**: the global shortcut fires on key-down — first press starts recording, second press stops. In **push-to-talk mode**: the global shortcut starts recording on key-down; a low-level Windows keyboard hook (via the `windows` crate, `SetWindowsHookExW` with `WH_KEYBOARD_LL`) detects key-up to stop recording. This platform-specific code is necessary because Tauri's global-shortcut plugin only provides key-down events.

5. **`output.rs`** — Handles transcription output. Two modes: auto-paste (copies text to clipboard via `arboard`, then simulates Ctrl+V via `enigo`) or clipboard-only (copies text to clipboard, sends a system notification via Tauri's notification plugin).

6. **`db.rs`** — SQLite database for transcription history. Single table: `transcriptions(id INTEGER PRIMARY KEY, text TEXT, provider TEXT, model TEXT, language TEXT, duration_ms INTEGER, created_at TEXT)`. Provides CRUD operations (insert, get all, delete by id, clear all) and text search.

7. **`settings.rs`** — Settings read/write from a JSON file stored in Tauri's app data directory. On first launch or if the file is missing/corrupt, uses default values (see Settings Schema below — the JSON block represents defaults). Validation rules: `base_url` must be a valid URL, `hotkey` must contain at least one modifier (Ctrl/Alt/Shift) plus a non-modifier key, `activation_mode` must be `"toggle"` or `"push_to_talk"`, `output_mode` must be `"auto_paste"` or `"clipboard"`.

8. **`commands.rs`** — Tauri IPC command handlers exposed to the frontend. Commands: `get_settings` (returns settings with API key masked as `sk-••••`), `save_settings` (accepts settings object; if `api_key` is empty or matches the masked pattern, the existing key is preserved — only updates the key if a new non-masked value is provided), `get_history`, `delete_transcription`, `clear_history`, `search_history`, `start_recording`, `stop_recording`, `get_recording_state`.

### Frontend Structure

```
src/
  lib/
    components/
      Settings.svelte       — Settings form
      History.svelte         — Transcription history list
      TabBar.svelte          — Settings/History tab switcher
      HistoryItem.svelte     — Single transcription entry
    stores/
      settings.ts            — Settings state (synced with backend)
      history.ts             — History state (loaded from backend)
      recording.ts           — Recording state (idle/recording/processing)
    types.ts                 — TypeScript interfaces
  App.svelte                 — Main window (tabs: Settings | History)
  main.ts                    — Main window entry point
  indicator/
    indicator.html           — Entry point for floating indicator window
    IndicatorApp.svelte      — Indicator app root (mounts the indicator UI)
    indicator-main.ts        — Indicator window entry point
  app.css                    — Tailwind imports and global styles
```

## Windows

### 1. Main Window

- **Size**: ~420x520px, not resizable
- **Visibility**: Hidden by default. Shown via tray menu (Open Settings or Open History).
- **Close behavior**: Hides the window (does not quit the app)
- **Taskbar**: No taskbar presence when hidden
- **Content**: Two tabs — Settings and History

### 2. Floating Indicator

- **Size**: ~80x28px pill shape
- **Position**: Top-center of the screen, 20px from top edge
- **Properties**: Frameless, transparent background, always-on-top, no taskbar presence. Click-through is achieved via Tauri v2's `ignore_cursor_events(true)` window method. If this proves insufficient, fall back to setting `WS_EX_TRANSPARENT` via the `windows` crate.
- **States**:
  - **Hidden**: Not visible when idle
  - **Recording**: Red pill with pulsing dot and "REC" label
  - **Processing**: Blue pill with spinning indicator and "..." label
- **Interaction**: Not interactive (click-through). Recording is controlled via hotkey or tray.

## Settings Schema

The following JSON represents the **default values** used on first launch or when the settings file is missing/corrupt:

```json
{
  "provider": "openai",
  "api_key": "",
  "base_url": "https://api.openai.com/v1",
  "model": "whisper-1",
  "language": "auto",
  "hotkey": "Ctrl+Shift+Space",
  "activation_mode": "toggle",
  "output_mode": "auto_paste",
  "launch_at_startup": false
}
```

### Provider Presets

When the user selects a provider, the base URL and available models update:

| Provider | Base URL | Models |
|----------|----------|--------|
| OpenAI | `https://api.openai.com/v1` | `whisper-1`, `gpt-4o-mini-transcribe`, `gpt-4o-transcribe` |
| Groq | `https://api.groq.com/openai/v1` | `whisper-large-v3`, `whisper-large-v3-turbo` |
| Custom | User-provided URL | User-provided model name |

All providers use the same OpenAI-compatible API. The request URL is `{base_url}/audio/transcriptions` (base_url already includes the version path like `/v1`).

### Setting Descriptions

- **provider**: Which transcription service to use. Determines default base URL and available models.
- **api_key**: API key for the selected provider. Stored in the settings JSON file in the app data directory. All API calls happen in Rust — the webview receives a masked version (`sk-••••`) via `get_settings`. When saving, the backend preserves the existing key if the value is empty or matches the masked pattern.
- **base_url**: API base URL including version path. Auto-filled from provider preset, editable for custom endpoints.
- **model**: Transcription model. Dropdown populated from provider preset, or free-text for custom.
- **language**: Language hint for transcription. `"auto"` for auto-detection, or an ISO 639-1 code (e.g., `"en"`, `"es"`, `"fr"`). Sent as the `language` parameter in the API request. Omitted from the request when set to `"auto"`.
- **hotkey**: Global keyboard shortcut. Default `Ctrl+Shift+Space`. User rebinds by clicking the hotkey field (it enters capture mode showing "Press a key combo..."), then pressing their desired combination. Must include at least one modifier key (Ctrl, Alt, or Shift) plus a non-modifier key. If the combo cannot be registered (conflict with another app), the previous hotkey is restored and a notification is shown.
- **activation_mode**: `"toggle"` (press to start, press again to stop) or `"push_to_talk"` (hold to record, release to stop).
- **output_mode**: `"auto_paste"` (paste at cursor via simulated Ctrl+V) or `"clipboard"` (copy to clipboard only, show a system notification).
- **launch_at_startup**: Register/unregister TinyWhispr to start with Windows via Tauri's `autostart` plugin.

## Transcription API Integration

All providers use the same HTTP request format:

```
POST {base_url}/audio/transcriptions
Authorization: Bearer {api_key}
Content-Type: multipart/form-data

file: <audio.wav>
model: <model_name>
language: <language_code>  (omitted if "auto")
response_format: json
```

Response:
```json
{
  "text": "Transcribed text here."
}
```

### Audio Format

Audio is recorded at the device's native sample rate. If the native rate is not 16kHz, `rubato` downsamples to 16kHz. The final output is 16-bit PCM WAV, mono, 16kHz — the most widely compatible format across all OpenAI-compatible endpoints. The file is sent as `audio.wav` in the multipart request.

### Maximum Recording Duration

Recording is capped at **5 minutes**. If the user is still recording when the limit is reached, recording automatically stops and the captured audio is sent for transcription. A notification is shown: "Maximum recording duration reached (5 min)."

### Error Handling

- **No API key**: Settings view shows a warning banner. Recording is disabled until a key is set.
- **Network error**: Indicator flashes red briefly then hides. System notification: "Transcription failed: network error."
- **API error (401/403)**: System notification: "Invalid API key."
- **API error (429)**: System notification: "Rate limit exceeded. Try again."
- **API error (other)**: System notification with the error message from the API response.
- **Empty transcription**: If API returns empty text, show notification "No speech detected." Do not save to history.

All notifications use Tauri's `notification` plugin (Windows toast notifications).

## History

### Database Schema

```sql
CREATE TABLE transcriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    language TEXT,
    duration_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_transcriptions_created_at ON transcriptions(created_at DESC);
```

### History View Features

- Chronological list, newest first
- Each entry shows: timestamp (relative, e.g., "Today, 2:34 PM"), transcribed text (truncated to 2 lines with ellipsis), Copy and Delete buttons
- Search bar at top — filters by text content using SQLite `LIKE` (simple substring match)
- "Clear All" button at the bottom to delete all history (with confirmation dialog)
- No pagination for v1 — load all entries (reasonable for typical dictation usage volume)

## UI Design

### Theme

Dark theme only for v1. Colors:
- Background: `#111111`
- Surface: `#1a1a1a`
- Border: `#262626`
- Text primary: `#e5e5e5`
- Text secondary: `#a3a3a3`
- Text muted: `#737373`
- Accent (active tab, links): `#3b82f6`
- Recording: `#ef4444`
- Processing: `#3b82f6`

### Typography

System font stack (`system-ui, -apple-system, sans-serif`). Small, clean type:
- Labels: 11px, uppercase, letter-spacing 1px, muted color
- Body: 13px
- Headings: 13px, semibold

### Main Window

- No native title bar — use Tauri's decorations or a custom drag region
- Tab bar at top: "Settings" | "History"
- Active tab has a 2px bottom border in accent blue
- Compact padding — the window should feel dense but not cramped

### Floating Indicator

- Rounded pill shape (border-radius: 20px)
- Semi-transparent background with backdrop blur
- Recording: red tint background, pulsing red dot, "REC" label
- Processing: blue tint background, spinning circle, "..." label
- Appears/disappears with a subtle fade animation

### Tray Icon

Three icon states (`.ico` format):
- **Idle**: Default app icon (small microphone or sound wave, monochrome/white to fit Windows tray style)
- **Recording**: Red-tinted version of the icon
- **Processing**: Blue-tinted version of the icon

## Data Flow

### Recording Flow

```
1. User triggers recording (hotkey or tray click)
2. Rust: if push-to-talk mode, install low-level keyboard hook for key-up detection
3. Rust: start audio capture via cpal
4. Rust: start 5-minute timeout timer
5. Rust: update tray icon to recording state
6. Rust: emit "recording-started" event to both windows
7. Frontend: indicator window shows recording state
8. User triggers stop (hotkey again, key release, tray click, or 5-min timeout)
9. Rust: stop audio capture, downsample if needed, encode WAV
10. Rust: update tray icon to processing state
11. Rust: emit "recording-stopped" event
12. Frontend: indicator shows processing state
13. Rust: send audio to transcription API
14. Rust: receive text response
15. Rust: save to SQLite history
16. Rust: output text (auto-paste or clipboard)
17. Rust: update tray icon to idle state
18. Rust: emit "transcription-complete" event with text
19. Frontend: indicator hides, history refreshes if visible
```

### Settings Flow

```
1. Frontend: user modifies a setting
2. Frontend: calls save_settings Tauri command with full settings object
   (api_key field is empty or masked if user didn't change it)
3. Rust: validates settings, merges api_key appropriately, writes to JSON file
4. Rust: if hotkey changed, re-register global shortcut
5. Rust: if launch_at_startup changed, update autostart registration
6. Rust: returns success/error
```

## File Structure

```
tinywhispr/
  src-tauri/
    src/
      main.rs              — Tauri app setup, window creation, plugin registration
      tray.rs              — System tray icon and menu
      audio.rs             — Audio capture (cpal) + resampling (rubato)
      transcribe.rs        — API client for transcription
      hotkey.rs            — Global hotkey management + Windows keyboard hook for PTT
      output.rs            — Auto-paste / clipboard output
      db.rs                — SQLite history store
      settings.rs          — Settings read/write + validation
      commands.rs          — Tauri IPC command handlers
      lib.rs               — Module declarations
    Cargo.toml
    tauri.conf.json
    icons/
      icon.ico             — Default tray icon (idle)
      icon-recording.ico   — Recording tray icon
      icon-processing.ico  — Processing tray icon
      icon.png             — App icon (for installer/taskbar)
  src/
    lib/
      components/
        Settings.svelte
        History.svelte
        TabBar.svelte
        HistoryItem.svelte
      stores/
        settings.ts
        history.ts
        recording.ts
      types.ts
    App.svelte
    main.ts
    indicator/
      indicator.html
      IndicatorApp.svelte
      indicator-main.ts
    app.css
  index.html
  package.json
  svelte.config.js
  vite.config.ts
  tailwind.config.ts
  tsconfig.json
```

## Security Considerations

- API keys are stored in a JSON file in the OS app data directory, not in the webview or localStorage
- All API calls happen in Rust — the webview never sees the raw API key (receives masked `sk-••••` only)
- When saving settings, the backend only updates the API key if a new non-masked, non-empty value is provided
- Audio data is transient — only kept in memory during recording, written to a temp file for the API call, then deleted
- No telemetry, no analytics, no external connections except the configured transcription API

## Future Considerations (Post-v1)

These are explicitly out of scope for v1 but inform the architecture:
- Local transcription (Whisper.cpp) — would add a new backend module, no frontend changes needed
- Streaming providers — would require WebSocket support in Rust
- AI post-processing — would add a second API call after transcription
- Custom dictionary — would add a keywords parameter to the API request
- Opus encoding — would reduce upload size; `opus` crate requires C build toolchain, deferred to avoid complexity
- Cross-platform — Tauri supports macOS/Linux, but platform-specific code (keyboard hook for PTT, auto-paste, startup registration) would need attention
- Light theme — add a theme toggle in settings
