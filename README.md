# TinyWhispr

Minimalistic Windows tray app for speech-to-text. Click the tray icon or press a hotkey to record, and the transcription is pasted at your cursor.

## Features

- **System tray** — lives in your tray, no window clutter
- **One-click recording** — left-click the tray icon or use a global hotkey
- **Auto-paste** — transcribed text is pasted where your cursor is (or copied to clipboard)
- **Multiple providers** — OpenAI, Groq, or any OpenAI-compatible API endpoint
- **Transcription history** — searchable list of all past transcriptions
- **Push-to-talk** — hold the hotkey to record, release to stop
- **Audio feedback** — Windows Speech Recognition sounds on start/stop
- **Pulsating tray icon** — bright red mic while recording so you always know

## Installation

Download the latest release from [Releases](https://github.com/Ashesh3/tiny-whispr/releases):

- **`TinyWhispr_x.x.x_x64-setup.exe`** — Windows installer (recommended)
- **`tinywhispr.exe`** — Portable executable, no install needed

## Setup

1. Launch TinyWhispr — it appears in your system tray
2. Right-click the tray icon → **Open Settings**
3. Select a provider (OpenAI, Groq, or Custom)
4. Enter your API key
5. Click **Save Settings**
6. Left-click the tray icon to start recording

## Usage

| Action | How |
|--------|-----|
| Start/stop recording | Left-click tray icon, or press hotkey (default: `Ctrl+Shift+Space`) |
| Open settings | Right-click tray → Open Settings |
| View history | Right-click tray → Open History |
| Quit | Right-click tray → Quit |

## Supported Providers

| Provider | Models |
|----------|--------|
| OpenAI | `whisper-1`, `gpt-4o-mini-transcribe`, `gpt-4o-transcribe` |
| Groq | `whisper-large-v3`, `whisper-large-v3-turbo` |
| Custom | Any OpenAI-compatible `/v1/audio/transcriptions` endpoint |

## Tech Stack

- [Tauri v2](https://v2.tauri.app/) + Rust backend
- [Svelte 5](https://svelte.dev/) + [Tailwind CSS v4](https://tailwindcss.com/) frontend
- Audio capture via [cpal](https://crates.io/crates/cpal), resampling via [rubato](https://crates.io/crates/rubato)
- SQLite history via [rusqlite](https://crates.io/crates/rusqlite)

## Building from Source

**Prerequisites:** [Node.js](https://nodejs.org/), [Rust](https://rustup.rs/), [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
git clone https://github.com/Ashesh3/tiny-whispr.git
cd tiny-whispr
npm install
npm run tauri build
```

Output: `src-tauri/target/release/tinywhispr.exe`

## License

MIT
