<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="AudioCaptor" width="128" height="128">
</p>

<h1 align="center">AudioCaptor</h1>

<p align="center">
  <strong>Portable system audio recorder for Windows</strong>
</p>

<p align="center">
  Record your microphone, system audio, or both — mixed into a single file.<br>
  No installation required. Just run the exe.
</p>

---

## Features

- **System audio capture (loopback)** — record what you hear through your speakers
- **Microphone recording** — capture from any connected mic
- **Real-time mixing** — combine mic and system audio into one file with adjustable volume levels
- **Multiple output modes** — Microphone only, System audio only, or Mix
- **Global hotkey** — start, pause, and stop recording from any window (default: `Pause/Break` key)
  - Short press: start / pause / resume
  - Long press (≥500ms): stop and save
- **Sound notifications** — audible feedback for start, pause, and stop events
- **Portable** — single `.exe` file, all data stored next to it
- **Accessible** — full screen reader support, keyboard navigation, and ARIA labels
- **WAV output** — lossless recording at 8, 16, 44.1, or 48 kHz

## Quick Start

1. Download `audio-captor.exe`
2. Place it in any folder (e.g., `C:\Tools\AudioCaptor\`)
3. Run it

On first launch, AudioCaptor creates the following alongside the exe:

```
audio-captor.exe
├── settings.json      # Your preferences (auto-saved)
├── logs/              # Diagnostic logs
└── Recordings/        # Your recorded audio files
```

## Usage

1. **Select devices** — choose your microphone and/or system audio device from the dropdowns
2. **Choose output mode** — Microphone only, System audio only, or Mix
3. **Adjust volume** — set mic and system audio levels (0–400%)
4. **Hit Record** — or use the global hotkey (`Pause/Break` by default)
5. **Pause / Resume** — short press the hotkey or use the UI buttons
6. **Stop** — long press the hotkey (≥500ms) or click Stop

Recordings are saved as WAV files in the `Recordings/` folder with automatic timestamps:
`recording_2026-03-31_14-30-00.wav`

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
| `Escape` | Close dialogs |

The global hotkey works even when AudioCaptor is not focused. You can change it in the settings.

## Settings

All settings are saved automatically to `settings.json` next to the executable:

| Setting | Default |
|---------|---------|
| Microphone device | (none) |
| System audio device | (none) |
| Mic volume | 100% |
| System audio volume | 50% |
| Output mode | Mix |
| Sample rate | 48000 Hz |
| Global hotkey | Pause/Break |
| Sound notifications | Enabled |

## System Requirements

- **OS**: Windows 10 or later
- **Runtime**: WebView2 (pre-installed on Windows 11; available on most Windows 10 machines)
- **Audio**: At least one audio input or output device
- **Disk**: ~10 MB for the executable; recording space depends on duration and sample rate

## Scoop Installation

AudioCaptor is designed to work with [Scoop](https://scoop.sh/), a Windows package manager. When installed via Scoop, your `settings.json`, `Recordings/`, and `logs/` folders are automatically persisted across updates.

## Accessibility

AudioCaptor is built with accessibility as a top priority:

- Every UI element is labeled for screen readers (NVDA, JAWS, Narrator)
- Full keyboard navigation — no mouse required
- Sound notifications mirror visual state changes
- Global hotkey works from any application
- `Alt+I` announces current recording status

## License

[MIT](LICENSE)

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for build instructions, architecture overview, and contribution guidelines.
