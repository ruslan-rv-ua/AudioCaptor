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
│   ├── app.css                 # Global styles
│   └── lib/
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
│   │   ├── hotkey.rs           # Global hotkey FSM (short/long press)
│   │   ├── settings.rs         # Settings persistence (settings.json)
│   │   ├── portable.rs         # Portable mode (paths relative to exe)
│   │   ├── sounds.rs           # Notification sounds (start/pause/stop)
│   │   └── state.rs            # Shared application state
│   ├── sounds/                 # OGG notification sound files
│   ├── icons/                  # App icons (all platforms)
│   ├── capabilities/           # Tauri permission capabilities
│   ├── tauri.conf.json         # Tauri configuration
│   └── Cargo.toml              # Rust dependencies
├── docs/                       # Project documentation
│   ├── requirements/           # PRD, glossary, user flows
│   └── phases/                 # Development phase specs
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
- **release**: `src-tauri/target/release/audio-captor.exe`
- **release-fast**: `src-tauri/target/release-fast/audio-captor.exe`

## Portable Mode

AudioCaptor runs as a **single portable executable** — no installation required. All data is stored relative to the exe location:

```
audio-captor.exe
├── settings.json     # User preferences
├── logs/             # Application logs
└── Recordings/       # Recorded audio files
```

These directories are created automatically on first launch (see `portable.rs`).

## App Icons

Icons are generated from a single 1024×1024 source PNG using the Tauri CLI:

```bash
pnpm tauri icon docs/requirements/resources/logo_audiocaptor.png
```

This generates all required variants in `src-tauri/icons/` (ICO for Windows, ICNS for macOS, PNGs for Linux, plus mobile icons). The Windows Store Square logos and the webview favicon are generated separately with ImageMagick — see the source file at `docs/requirements/resources/logo_audiocaptor.png`.

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

## Documentation

- [PRD (Product Requirements)](docs/requirements/PRD.md) — full functional requirements
- [Development Phases](docs/phases/index.md) — phased implementation plan
- [Tech Stack Research](docs/stack-tauri-v2.md) — technology decisions and rationale
