# AudioCaptor — Development Guide

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri v2](https://v2.tauri.app/) (Rust backend + WebView frontend) |
| Backend | Rust 2021 edition |
| Frontend | Svelte 5 + TypeScript + Vite |
| Audio capture | [wasapi](https://github.com/HEnquist/wasapi-rs) (WASAPI bindings) |
| Audio mixing | Real-time mixer with [rubato](https://github.com/HEnquist/rubato) resampling |
| WAV writing | [hound](https://github.com/ruuda/hound) |
| Sound notifications | [rodio](https://github.com/RustAudio/rodio) (OGG playback via `include_bytes!`) |
| Global hotkeys | `tauri-plugin-global-shortcut` |
| Single instance | `tauri-plugin-single-instance` |
| Logging | `tauri-plugin-log` (file + console) |
| Package manager | [pnpm](https://pnpm.io/) |
| Task runner | [just](https://github.com/casey/just) |

## Prerequisites

- **Windows 10+** (the app is Windows-only by design)
- [Rust](https://rustup.rs/) stable toolchain
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/) (`npm install -g pnpm`)
- [just](https://github.com/casey/just) (optional, for task shortcuts)
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11; available on most Windows 10 machines)

## Getting Started

```bash
# Install JavaScript dependencies
pnpm install

# Start development server (Rust + Vite hot-reload)
just dev       # or: pnpm tauri dev
```

The app will open automatically. Rust changes trigger a rebuild; frontend changes hot-reload instantly.

## Project Structure

```
AudioCaptor/
├── src/                        # Frontend (Svelte 5 + TypeScript)
│   ├── App.svelte              # Main application component
│   ├── main.ts                 # Entry point
│   ├── app.css                 # Global styles & CSS tokens
│   └── lib/
│       ├── i18n.ts             # Paraglide i18n initialization
│       ├── components/         # UI components
│       ├── stores/             # Svelte state stores
│       ├── types/              # TypeScript type definitions
│       └── utils/              # IPC invoke helpers
├── src-tauri/                  # Backend (Rust / Tauri)
│   ├── src/
│   │   ├── lib.rs              # Tauri commands & app setup
│   │   ├── main.rs             # Entry point
│   │   ├── audio/              # Audio capture, mixing, writing
│   │   │   ├── capture.rs      # WASAPI mic & loopback capture
│   │   │   ├── devices.rs      # Device enumeration
│   │   │   ├── mixer.rs        # Real-time audio mixer
│   │   │   ├── writer.rs       # WAV file writer
│   │   │   └── types.rs        # Audio types & enums
│   │   ├── device_monitor.rs   # IMMNotificationClient hot-plug detection
│   │   ├── hotkey.rs           # Global hotkey FSM (short/long press)
│   │   ├── profiles.rs         # Recording profiles CRUD
│   │   ├── settings.rs         # Settings persistence & migration
│   │   ├── portable.rs         # Portable mode (paths relative to exe)
│   │   ├── sounds.rs           # Notification sounds (start/pause/stop)
│   │   ├── state.rs            # Shared application state
│   │   └── tray.rs             # System tray icon & menu
│   ├── sounds/                 # OGG notification sound files
│   ├── icons/                  # App icons (all platforms)
│   ├── capabilities/           # Tauri permission capabilities
│   ├── tauri.conf.json         # Tauri configuration
│   └── Cargo.toml              # Rust dependencies
├── messages/                   # i18n message files (en, uk)
├── docs/                       # Design specs & research
├── public/                     # Static assets (favicon)
├── index.html                  # HTML entry point
├── package.json                # Node.js config & scripts
├── vite.config.ts              # Vite bundler configuration
├── justfile                    # Task runner recipes
└── tsconfig*.json              # TypeScript configuration
```

## Build Commands

| Command | Description |
|---------|-------------|
| `just dev` | Start dev server with hot-reload |
| `just build` | Production build (optimized, slow compile) |
| `just build-fast` | Fast build (less optimization, quick compile) |
| `just clean` | Clean Rust build artifacts |
| `just install` | Install JS dependencies |
| `just vite-dev` | Frontend-only dev server (no Rust) |

<details>
<summary>Equivalent pnpm commands</summary>

```bash
pnpm tauri dev                                       # dev
pnpm tauri build --no-bundle                         # build (release profile)
pnpm tauri build --no-bundle -- --profile release-fast  # build-fast
pnpm vite dev --port 1420                            # vite-dev
```

</details>

## Build Profiles

| Profile | Optimizations | Use Case |
|---------|--------------|----------|
| `release` | `lto=true`, `codegen-units=1`, `opt-level="s"`, `strip=true`, `panic="abort"` | Final distribution |
| `release-fast` | `lto=false`, `codegen-units=16`, `opt-level=1`, `panic="unwind"` | Quick iteration during development |

The output binary is located at:
- **release**: `src-tauri/target/release/AudioCaptor.exe`
- **release-fast**: `src-tauri/target/release-fast/AudioCaptor.exe`

## Portable Mode

AudioCaptor runs as a **single portable executable** — no installation required. All data is stored in `AudioCaptor-data/` next to the exe:

```
AudioCaptor.exe
AudioCaptor-data/
├── settings.json     # User preferences
├── logs/             # Application logs
└── Recordings/       # Default recordings folder
```

These directories are created automatically on first launch (see `portable.rs`).

## App Icons

Icons are generated from a single 1024×1024 source PNG using the Tauri CLI:

```bash
pnpm tauri icon src-tauri/icons/128x128.png
```

This generates all required variants in `src-tauri/icons/` (ICO for Windows, ICNS for macOS, PNGs for Linux, plus mobile icons). The Windows Store Square logos and the webview favicon are generated separately with ImageMagick.

Icon integration points:
- `src-tauri/icons/` — all platform icon variants
- `src-tauri/tauri.conf.json` → `bundle.icon` — references for the packaging step
- `public/favicon.png` — webview favicon (referenced in `index.html`)

## Architecture Overview

### Audio Pipeline

```
┌──────────┐     ┌───────────┐     ┌─────────┐     ┌───────────┐
│ WASAPI   │────▶│ Ring      │────▶│ Mixer   │────▶│ WAV       │
│ Capture  │     │ Buffers   │     │ Thread  │     │ Writer    │
│ Threads  │     └───────────┘     │         │     │ (hound)   │
│          │                       │ resample│     └───────────┘
│ • mic    │                       │ volume  │
│ • loopback│                      │ clip    │
└──────────┘                       └─────────┘
```

- **Capture threads**: One per audio source (mic, loopback) using WASAPI shared mode
- **Ring buffers**: Lock-free transfer from capture threads to mixer
- **Mixer thread**: Resamples if needed, applies volume, clips, and writes to WAV
- **Pause**: Mixer keeps reading buffers (prevents desync) but skips writing

### IPC Commands

Frontend communicates with backend via Tauri IPC commands:

| Command | Direction |
|---------|-----------|
| `get_audio_devices` | Frontend → Backend |
| `start_recording` | Frontend → Backend |
| `pause_recording` | Frontend → Backend |
| `resume_recording` | Frontend → Backend |
| `stop_recording` | Frontend → Backend |
| `load_settings` / `save_settings` | Frontend ↔ Backend |
| `recording-state-changed` | Backend → Frontend (event) |

### Global Hotkey FSM

The global hotkey (default: `Pause/Break`) uses a finite state machine with short/long press detection:

| Current State | Short press (<500ms) | Long press (≥500ms) |
|--------------|---------------------|---------------------|
| Idle | → Start recording | → Start recording |
| Recording | → Pause | → Stop |
| Paused | → Resume | → Stop |

## Accessibility

Accessibility (a11y) is a **highest-priority requirement**:

- All UI elements have ARIA labels and roles
- Full keyboard navigation (Tab, Enter, Escape)
- Mnemonics: Alt+S (Start), Alt+P (Pause), Alt+R (Resume), Alt+T (Stop), Alt+I (Status info)
- `aria-live` region announces state changes to screen readers
- `decorations: true` in Tauri config (required for screen reader compatibility)
- Svelte compile-time a11y checks are enabled
- Sound notifications duplicate visual state changes

## CI/CD

### GitHub Actions Workflows

| Workflow | Trigger | Description |
|----------|---------|-------------|
| **CI** (`ci.yml`) | push / PR → `develop` | Frontend checks (`pnpm check`) + Rust checks (`cargo check`, `cargo clippy`) |
| **Release** (`release.yml`) | manual (`workflow_dispatch`) | Builds `AudioCaptor.exe`, creates GitHub Release |

### Release Process

1. Запустити **Release** workflow з версією (наприклад `0.2.0`) та прапорцем pre-release
2. Дочекатись успішного створення GitHub Release
3. Оновити Scoop-маніфест — див. нижче

### Оновлення Scoop-маніфесту

Цей репозиторій **нічого не надсилає** до `scoop-bucket`. Оновлення робить сам bucket:
його workflow **Excavator** читає поля `checkver` і `autoupdate` у `bucket/audiocaptor.json`,
знаходить новий реліз, збирає URL і бере хеш — і комітить оновлений маніфест.

Запускається він **вручну** з вкладки Actions репозиторію
[scoop-bucket](https://github.com/ruslan-rv-ua/scoop-bucket/actions), плюс раз на добу
о 04:20 UTC як підстраховка. Натискати безпечно будь-коли: якщо оновлювати нема чого,
він нічого не змінює.

Раніше тут був workflow `update-scoop.yml`, який слав `repository_dispatch` до bucket
і для цього тримав секрет `SCOOP_BUCKET_TOKEN`. Обидва прибрано: PAT із правом запису
в чужий репозиторій треба ротувати, і він відмовляє мовчки — реліз проходить, а bucket
тихо лишається позаду.

### Required Secrets

Жодного. Workflow цього репозиторію користуються тільки тимчасовим `GITHUB_TOKEN`,
який GitHub видає кожному запуску й забирає після нього.
