# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Recordings can now be saved as Opus, a compressed format that takes many times less space than WAV. Edit a profile and choose its new Format: "WAV (uncompressed)", which stays the default, "Opus — speech (mono)", about 14 MB per hour, or "Opus — music (stereo)", about 43 MB per hour; an hour of WAV at 48 kHz takes 691 MB. The recording is still written as WAV. When you stop it, AudioCaptor converts it in the background (an hour takes about half a minute on a recent PC), checks that the new `.opus` file can be decoded and lasts exactly as long as the recording, and only then deletes the WAV. Recordings are converted one at a time, in the order you stopped them, and you can start a new recording meanwhile.
- If a recording cannot be converted, it is not lost: the WAV stays, no unfinished `.opus` file is left, the warning sound plays (unless another recording is running, so that the sound does not get into it) and a message, which the screen reader reads, says where the WAV is.
- The screen reader says when a conversion starts and when all conversions are finished. `Alt+I` now also says that a conversion is running, about how long is left for all of them and how many files are waiting.
- If you exit while recordings are being converted or are waiting for it, AudioCaptor asks first. Nothing is lost if you exit: the WAV stays whole, no unfinished `.opus` file is left, and the conversion is finished the next time AudioCaptor starts, which the screen reader says. The setting that turns this question off is the one for recordings, now called "Confirm exit during recording or conversion".
- Opus works at every sample rate of a profile. A 44.1 kHz recording is converted to 48 kHz, because Opus has no 44.1 kHz. At 16 and 8 kHz the sample rate itself limits the quality, so the files are smaller there (speech 24 and 16 kbit/s, music 48 and 32 kbit/s instead of 32 and 96): a higher bitrate would not sound better.

### Changed

- If you go back to AudioCaptor 0.1.2 or older, your profiles stay, but the older version does not know the Format and records WAV; after you update again, choose the format once more.

## [0.1.2] — 2026-11-26

### Fixed

- NVDA now reads the title of the About dialog once instead of three times when it opens, and Enter closes About, as Escape and Close already did.
- If AudioCaptor cannot read `settings.json` (for example, it is damaged or empty), it no longer erases your settings without a word: it keeps the old file next to the new one as `settings.unreadable.json` (never overwriting an earlier copy), plays the warning sound and shows a message, which the screen reader reads, saying that the settings were reset to defaults and where the old file is. Settings are also saved through a temporary file now, so a power cut while saving cannot leave a damaged `settings.json`.

### Changed

- AudioCaptor now keeps the data of its built-in browser (Microsoft Edge WebView2, which draws the window) in a new `AudioCaptor-webview` folder next to `AudioCaptor.exe` instead of `%LOCALAPPDATA%\com.audiocaptor.app\EBWebView`, so copies in different folders no longer share it. The folder holds only the browser's cache and its own state, none of your settings or recordings; if you delete it while AudioCaptor is closed, the app creates it again. With Scoop it lies in the folder of each version, so after an update the browser starts with an empty cache. This version neither uses nor deletes the old `EBWebView` folder, so an older copy still works; you can delete it once you no longer need AudioCaptor 0.1.1 or older.

## [0.1.1] — 2026-11-25

### Fixed

- Starting AudioCaptor again while it is already running now brings its window to the front, even when it is hidden in the tray or minimized to the taskbar; before, nothing happened and the screen reader said nothing. Recording and pause are not affected.
- When you close a dialog (About, Settings, a profile, or the delete or exit confirmation), keyboard focus now goes back to where it was when the dialog opened, for example the button that opened it, and the screen reader reads it. After you save or delete a profile, focus goes to the profile list instead, and the screen reader names the active profile. Before, focus always jumped to the About button or the profile list, and after Cancel in the exit confirmation during a recording it was lost. When the place to go back to is no longer available, focus goes to the profile list, or to the recording buttons while recording.
- Keyboard focus is no longer lost when recording starts, pauses, resumes or stops. When the control with focus becomes unavailable during recording (the profile list and its buttons, the device lists) or a recording button goes away, focus moves to the recording button of the new state (Start, Pause or Resume), and the screen reader names it. Before, the screen reader said "unavailable" or only the app name, and you had to find your place again. Focus on Stop now stays there when you pause or resume.
- When you resume a paused recording, the screen reader now says "Recording resumed" instead of "Recording started", so you can tell it apart from a new recording. The announcement is the same however you resume: Alt+R, the Resume button, the global hotkey or the tray menu.
- When AudioCaptor starts, the screen reader now says its name once before it names the profile list. Before, NVDA said "AudioCaptor" six or seven times in a row.
- AudioCaptor now keeps its log only in the `AudioCaptor-data\logs` folder next to the exe, and the log no longer fills up with technical lines from the audio library. Before, it also wrote a second log to `%LOCALAPPDATA%\com.audiocaptor.app\logs`. You can delete that `logs` folder: earlier versions left it behind.
- About, opened with F1 or from the tray menu while Settings or the delete or exit confirmation is open, now appears on top of that dialog. Before, it opened behind it: the keys went to About and the screen reader read About, but the screen showed the other dialog. Any dialog that opens over another one now appears on top of it.

## [0.1.0] — 2026-04-20

Initial release.

### Added

- System audio capture (WASAPI loopback) and microphone recording
- Real-time mixer with per-source volume control and resampling
- Multiple output modes: Microphone, System, Mix, Mix + Mic, Mix + Loopback, Separate Files
- WAV output at 8, 16, 44.1, or 48 kHz sample rates
- Recording profiles with per-profile output folder, mode, sample rate, volume, and filename patterns
- Global hotkey (`Pause/Break`) with short press (start/pause/resume) and long press (stop)
- System tray with Show/Hide, recording controls, and Open Recordings Folder
- Sound notifications for start, pause, and stop events
- Hot-plug device detection (IMMNotificationClient)
- Auto, Light, and Dark themes
- Internationalization: English and Ukrainian
- Portable mode — single `.exe`, all data in `AudioCaptor-data/` folder
- Full accessibility: ARIA labels, keyboard navigation, screen reader support, `Alt+I` status announcement
- About dialog with hotkeys reference and quick start guide
- Settings migration (v1 → v2 → v3 → v4 → v5)
- Scoop package manifest in [ruslan-rv-ua/scoop-bucket](https://github.com/ruslan-rv-ua/scoop-bucket)

[0.1.2]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.2
[0.1.1]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.1
[0.1.0]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.0
