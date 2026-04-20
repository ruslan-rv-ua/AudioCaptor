<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="AudioCaptor logo" width="128" height="128">
</p>

<h1 align="center">AudioCaptor</h1>

<p align="center">
  <strong>Portable system audio recorder for Windows</strong>
</p>

<p align="center">
  Record your microphone, system audio, or both — with flexible output modes.<br>
  No installation required. Just run the exe.
</p>

---

## Features

- **System audio capture (loopback)** — record what you hear through your speakers
- **Microphone recording** — capture from any connected mic
- **Real-time mixing** — combine mic and system audio with adjustable volume levels
- **Multiple output modes** — Microphone, System audio, Mix, Mix + Mic, Mix + Loopback, Separate Files
- **Recording profiles** — save per-profile output folder, mode, sample rate, volume, and filenames
- **Global hotkey** — start, pause, and stop recording from any window (default: `Pause/Break` key)
  - Short press: start / pause / resume
  - Long press (≥500ms): stop and save
- **System tray** — minimize to tray, control recording from tray menu
- **Sound notifications** — audible feedback for start, pause, and stop events
- **Themes** — Auto, Light, and Dark mode
- **Internationalization** — English and Ukrainian
- **Portable** — single `.exe` file, all data stored in `AudioCaptor-data/` folder
- **Accessible** — full screen reader support, keyboard navigation, ARIA labels
- **WAV output** — lossless recording at 8, 16, 44.1, or 48 kHz

## Quick Start

1. Download `AudioCaptor.exe`
2. Place it in any folder (e.g., `C:\Tools\AudioCaptor\`)
3. Run it

On first launch, AudioCaptor creates the following alongside the exe:

```
AudioCaptor.exe
AudioCaptor-data/
├── settings.json      # Your preferences (auto-saved)
├── logs/              # Diagnostic logs
└── Recordings/        # Default recordings folder
```

## Usage

1. **Select devices** — choose your microphone and/or system audio device from the dropdowns
2. **Configure a profile** — choose output mode, sample rate, volume levels, and output folder
3. **Hit Record** — or use the global hotkey (`Pause/Break` by default)
4. **Pause / Resume** — short press the hotkey or use the UI buttons
5. **Stop** — long press the hotkey (≥500ms) or click Stop

Recordings are saved as WAV files with automatic timestamps (e.g., `mix_2026-03-31_14-30-00.wav`).

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Pause/Break` | Global hotkey — start / pause / stop recording |
| `Alt+S` | Start recording |
| `Alt+P` | Pause recording |
| `Alt+R` | Resume recording |
| `Alt+T` | Stop recording |
| `Alt+I` | Announce current status (for screen readers) |
| `Tab` | Navigate between UI elements |
| `Escape` | Close dialogs / minimize to tray |

The global hotkey works even when AudioCaptor is not focused. You can change it in Settings.

## System Requirements

- **OS**: Windows 10 or later
- **Runtime**: WebView2 (pre-installed on Windows 11; available on most Windows 10 machines)
- **Audio**: At least one audio input or output device
- **Disk**: ~10 MB for the executable; recording space depends on duration and sample rate

## Scoop Installation

AudioCaptor is designed to work with [Scoop](https://scoop.sh/), a Windows package manager:

```powershell
scoop bucket add audiocaptor https://github.com/ruslan-rv-ua/AudioCaptor
scoop install audiocaptor
```

When installed via Scoop, the `AudioCaptor-data/` folder is automatically persisted across updates.

## Accessibility

AudioCaptor is built with accessibility as a top priority:

- Every UI element is labeled for screen readers (NVDA, JAWS, Narrator)
- Full keyboard navigation — no mouse required
- Sound notifications mirror visual state changes
- Global hotkey works from any application
- `Alt+I` announces current recording status
- About dialog with hotkeys reference and quick start guide

## License

[MIT](LICENSE)

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for build instructions and architecture overview.
