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

<p align="center">
  English · <a href="README_UK.md">Українська</a>
</p>

---

## Features

- **System audio capture (loopback)** — record what you hear through your speakers
- **Microphone recording** — capture from any connected mic
- **Real-time mixing** — combine mic and system audio with adjustable volume levels
- **Multiple output modes** — Microphone, System audio, Mix, Mix + Mic, Mix + Loopback, Separate Files
- **Recording profiles** — save per-profile output folder, mode, sample rate, format, volume, and filenames
- **Global hotkey** — start, pause, and stop recording from any window (default: `Pause/Break` key)
  - Short press: start / pause / resume
  - Long press (≥500ms): stop and save
- **System tray** — minimize to tray, control recording from tray menu
- **Sound notifications** — audible feedback for start, pause, and stop events
- **Themes** — Auto, Light, and Dark mode
- **Internationalization** — English and Ukrainian
- **Portable** — single `.exe` file, all data stored in `AudioCaptor-data/` folder
- **Accessible** — full screen reader support, keyboard navigation, ARIA labels
- **WAV, FLAC, Opus or MP3** — lossless WAV, lossless FLAC in a smaller file, or Opus and MP3 files many times smaller, made automatically when a recording stops; 8, 16, 44.1, or 48 kHz

## Quick Start

1. Download `AudioCaptor.exe`
2. Place it in any folder (e.g., `C:\Tools\AudioCaptor\`)
3. Run it

On first launch, AudioCaptor creates the following alongside the exe:

```
AudioCaptor.exe
AudioCaptor-data/
├── settings.json      # Your preferences (auto-saved)
├── settings.unreadable.json  # Only if settings.json could not be read: a copy of the old file
├── conversion-queue.json     # Only while recordings wait to be converted to Opus, MP3 or FLAC
├── logs/              # Diagnostic logs
└── Recordings/        # Default recordings folder
AudioCaptor-webview/   # Cache of the built-in browser
```

The built-in browser that draws the window (Microsoft Edge WebView2) keeps its cache and its own state in the `AudioCaptor-webview` folder. None of your settings or recordings are in it, so it needs no backup; if you delete it while AudioCaptor is closed, the app creates it again. Versions 0.1.1 and older kept this data in `%LOCALAPPDATA%\com.audiocaptor.app`: once you no longer run them, you can delete that folder.

## Usage

1. **Select devices** — choose your microphone and/or system audio device from the dropdowns
2. **Configure a profile** — choose output mode, sample rate, format, volume levels, and output folder
3. **Hit Record** — or use the global hotkey (`Pause/Break` by default)
4. **Pause / Resume** — short press the hotkey or use the UI buttons
5. **Stop** — long press the hotkey (≥500ms) or click Stop

Recordings are saved with automatic timestamps (e.g., `mix_2026-03-31_14-30-00.wav`): as WAV files, or as `.opus`, `.mp3` or `.flac` files when the profile's format is Opus, MP3 or FLAC.

## Recording Formats

The **Format** list of a profile sets the kind of file you get:

| Format | What you get | An hour of recording |
|--------|--------------|----------------------|
| WAV (uncompressed) | The sound exactly as recorded. The default. | 691 MB at 48 kHz |
| Opus — speech (mono) | Small files for meetings and lectures. Both channels are mixed into one. | about 14 MB |
| Opus — music (stereo) | Music and other system audio, in stereo. | about 43 MB |
| MP3 — speech (mono, 64 kbps) | Meetings and lectures in the format that every player, phone and website opens. Both channels are mixed into one. | about 29 MB |
| MP3 — standard (128 kbps) | Any sound in stereo, in the most common kind of MP3. | about 58 MB |
| MP3 — high quality (320 kbps) | Music in stereo, as close to the WAV as MP3 gets. | about 144 MB |
| FLAC (lossless) | The sound exactly as recorded, sample for sample, in a smaller file. For an archive, or to edit the recording later. | from about 130 MB (speech) to about 380 MB (music) at 48 kHz |

Opus and MP3 are lossy formats: they leave out what the ear hardly notices, so the sound is very close to the WAV but not identical. Opus gives the same quality in smaller files; MP3 opens everywhere. Choose FLAC or WAV when you need the recording exactly as it was, for example to edit it later. FLAC is lossless: it keeps every sample of the WAV and only packs them tighter, so how much smaller the file gets depends on the sound. Quiet speech packs down to about a fifth of the WAV, music to a little over a half.

How a recording becomes an Opus, MP3 or FLAC file:

- The recording itself is written as WAV, as before. When you stop it, AudioCaptor converts it in the background. On a recent PC an hour of recording takes about half a minute in Opus and about a quarter of a minute in MP3 or FLAC.
- The new file gets the same name with `.opus`, `.mp3` or `.flac` at the end. AudioCaptor checks it before it deletes the WAV, so one file per track is left: an Opus file must decode and last exactly as long as the recording; an MP3 file must consist of whole MP3 frames and last as long as the recording plus the silence of a few hundredths of a second that every MP3 encoder adds at the start and the end; a FLAC file is decoded in full, and every sample of it must be the same as in the WAV.
- Recordings are converted one at a time, in the order you stopped them. You can start a new recording meanwhile.
- The screen reader says when a conversion starts and when all conversions are finished. `Alt+I` says about how long is left and how many files are waiting.
- If a recording cannot be converted, it is not lost: the WAV stays, the warning sound plays and a message says where the WAV is.
- If you exit while recordings are being converted, AudioCaptor asks first. If you do exit, the WAV stays whole, and the conversion is finished the next time AudioCaptor starts.

Opus and the sample rate of a profile:

- **48 kHz** gives the best quality.
- **44.1 kHz** is converted to 48 kHz, because Opus has no 44.1 kHz.
- At **16 and 8 kHz** the sample rate itself limits the quality: a 16 kHz recording holds no sound above 8 kHz, an 8 kHz recording none above 4 kHz, whatever the format. More bits would not sound better there, so AudioCaptor makes smaller files: speech at 24 and 16 kbit/s instead of 32, music at 48 and 32 kbit/s instead of 96.

MP3 and the sample rate of a profile:

- **44.1 and 48 kHz** give the bitrates in the table above.
- At **16 kHz** MP3 allows at most 80 kbit/s in stereo and 48 kbit/s in mono, so "standard" and "high quality" both give 80 kbit/s (about 36 MB per hour) and "speech" gives 48 kbit/s (about 22 MB).
- At **8 kHz** MP3 allows at most 32 kbit/s in stereo and 16 kbit/s in mono: "standard" and "high quality" give 32 kbit/s (about 14 MB per hour), "speech" 16 kbit/s (about 7 MB).

FLAC and the sample rate of a profile: FLAC keeps the sample rate and the channels of the recording as they are, at 8, 16, 44.1 and 48 kHz. Nothing is converted and nothing is mixed.

WAV recordings over 4 GB: a WAV file can hold at most 4 GB of sound, and a recording reaches that after about 6 h 12 min at 48 kHz (6 h 45 min at 44.1 kHz, more than 18 hours at 16 kHz). A longer recording still stays one file: once it passes 4 GB, AudioCaptor goes on writing it as RF64, the form of WAV made for files over 4 GB, and the file keeps its `.wav` name. Not every program opens RF64. Windows itself reads it: Explorer shows its length, and the media components of Windows that its players use play it to the end. Programs built on FFmpeg, such as mpv, open it too. Some programs, mostly older ones, do not open it. If a program you need does not open such a file, record long sessions in the FLAC format: FLAC keeps every sample as WAV does, in a far smaller file without this limit. A recording under 4 GB is an ordinary WAV that opens wherever it opened before.

MP3 on Windows N: AudioCaptor makes MP3 files with the MP3 encoder that comes with Windows, so it needs no extra library. The N and KN editions of Windows have no such encoder until the Media Feature Pack is installed. There a recording in an MP3 profile stays WAV: the warning sound plays and a message says how to get the encoder. Install the Media Feature Pack from Optional features in Windows Settings, restart the computer, and the next recordings are saved as MP3.

AudioCaptor 0.1.2 and older do not know the format of a profile. If you go back to one of them, your profiles stay, but they record WAV, and you choose the format again after you update. AudioCaptor 0.2.0 knows WAV and Opus but not MP3, and AudioCaptor 0.3.0 knows MP3 but not FLAC: there a profile with a format that the version does not know records WAV, and the format works again after you update, unless you saved that profile in the older version.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Pause/Break` | Global hotkey — start / pause / stop recording |
| `Alt+S` | Start recording |
| `Alt+P` | Pause recording |
| `Alt+R` | Resume recording |
| `Alt+T` | Stop recording |
| `Alt+I` | Announce current status, including a running conversion (for screen readers) |
| `Tab` | Navigate between UI elements |
| `Escape` | Close dialogs / minimize to tray |

The global hotkey works even when AudioCaptor is not focused. You can change it in Settings: press Capture, then the key. Capture takes the key itself, not the character it types, so it works the same under any keyboard layout: Ctrl with the Й/Q key is set as Ctrl+Q under the Ukrainian layout too.

## System Requirements

- **OS**: Windows 10 or later
- **Runtime**: WebView2 (pre-installed on Windows 11; available on most Windows 10 machines)
- **Audio**: At least one audio input or output device
- **Disk**: ~10 MB for the executable; recording space depends on duration and sample rate

## Scoop Installation

AudioCaptor is available via [Scoop](https://scoop.sh/), a Windows package manager:

```powershell
scoop bucket add ruslan-rv-ua https://github.com/ruslan-rv-ua/scoop-bucket
scoop install audiocaptor
```

When installed via Scoop, the `AudioCaptor-data/` folder is automatically persisted across updates. The `AudioCaptor-webview/` folder is not: it lies in the folder of each version, so after an update the browser starts with an empty cache.

## Accessibility

AudioCaptor is built with accessibility as a top priority:

- Every UI element is labeled for screen readers (NVDA, JAWS, Narrator)
- Full keyboard navigation — no mouse required
- Sound notifications mirror visual state changes
- Global hotkey works from any application
- `Alt+I` announces current recording status
- The start and the end of a conversion to Opus, MP3 or FLAC are announced; `Alt+I` says about how long is left
- About dialog with hotkeys reference and quick start guide

## License

[MIT](LICENSE)

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for build instructions and architecture overview.
