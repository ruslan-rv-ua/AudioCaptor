# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- After a recording stops in an Opus, MP3 or FLAC profile, the screen reader now always says first that the recording stopped and then how its conversion ended, even when the conversion ends at once. Before, when the conversion failed at once (for example, on the N editions of Windows without the Media Feature Pack) or a very short recording was converted at once, the screen reader skipped "Recording stopped" and read only the end of the conversion. To leave time for the stop announcement, the end of a conversion and the message about a recording that could not be converted now come no sooner than three seconds after the stop.
- Alt+S now says why a recording does not start when the profile needs a device that is not selected, for example "Select a system audio device": the same hint as the disabled Start recording button. Before, Alt+S did nothing and the screen reader said nothing. Focus stays where it was. The global recording hotkey in this state plays the warning sound and does not start a recording, as before.
- The keys and the menu of the built-in browser no longer work in the AudioCaptor window. Before, F5 or Ctrl+R reloaded the window: it went back to its starting state, an unsaved profile was lost, and during a recording the screen reader wrongly said "Recording started". Ctrl+P opened printing, Ctrl+F, F3 and Ctrl+G opened a search bar, and the Applications key, Shift+F10 or a right click opened a browser menu with items such as "Reload" and "Print". Copy, paste, cut, select all and undo still work in the text fields.
- The time in the app's log (`AudioCaptor-data/logs/audiocaptor.log`) now matches the Windows clock. Before, every line was stamped in UTC: on a PC two hours ahead of UTC, a recording started at 23:17 was logged at 21:17, and near midnight even the date was a day off. The rest of the line is unchanged.
- An unreadable `settings.json` that AudioCaptor could not copy at startup (for example, because another program briefly locked it or the disk was full) is no longer overwritten by the first change of a setting. Before your first change is saved, AudioCaptor tries again to copy the file to `settings.unreadable.json`, and saves your settings as usual once the copy is made. If the copy fails again, `settings.json` stays as it is until AudioCaptor restarts: your changes work until you close the app, but are not saved.

## [0.4.0] — 2026-11-27

### Added

- Recordings can now be saved as FLAC, a lossless format: the file holds exactly the sound of the WAV, sample for sample, and takes less space. The Format list of a profile has a new entry, "FLAC (lossless)". How much smaller the file gets depends on the sound: an hour of quiet speech at 48 kHz takes about 130 MB instead of 691 MB, an hour of music about 380 MB. As with Opus and MP3, the recording is written as WAV and converted in the background when you stop it (an hour takes about a quarter of a minute on a recent PC). The WAV is deleted only after AudioCaptor has decoded the whole `.flac` file and found every sample the same as in the WAV.
- FLAC works at every sample rate of a profile and keeps the sample rate and the channels of the recording as they are. It needs no encoder from Windows, so it also works on the N and KN editions without the Media Feature Pack.

### Changed

- If you go back to AudioCaptor 0.3.0, a profile with the FLAC format records WAV there. The format works again after you update, unless you saved that profile in 0.3.0.

### Fixed

- The profile dialog now closes with the first press of Escape. Before, after a profile had been saved with Enter, the built-in browser of AudioCaptor offered the saved name as an autofill suggestion in the empty Name field: the screen reader announced the suggestions, and the first Escape only hid them. The text fields of the profile dialog no longer show autofill suggestions.

## [0.3.0] — 2026-11-27

### Added

- Recordings can now be saved as MP3, the format that every player, phone and website opens. The Format list of a profile has three new entries: "MP3 — speech (mono, 64 kbps)", about 29 MB per hour; "MP3 — standard (128 kbps)", about 58 MB per hour; and "MP3 — high quality (320 kbps)", about 144 MB per hour, for music. As with Opus, the recording is written as WAV and converted in the background when you stop it (an hour takes about a quarter of a minute on a recent PC), and the WAV is deleted only after AudioCaptor has checked that the `.mp3` file is whole and lasts as long as the recording.
- MP3 files are made by the MP3 encoder that comes with Windows, so AudioCaptor stays a single small file. The N and KN editions of Windows have no such encoder until the Media Feature Pack is installed: there a recording in an MP3 profile stays WAV, the warning sound plays and a message, which the screen reader reads, says how to install the Media Feature Pack. AudioCaptor itself starts and works as before on these editions.
- MP3 works at every sample rate of a profile. At 16 and 8 kHz MP3 allows only lower bitrates, so "standard" and "high quality" give 80 kbit/s at 16 kHz and 32 kbit/s at 8 kHz, and "speech" 48 and 16 kbit/s.

### Changed

- Conversions to Opus and MP3 now run at a lower priority, so a recording that you make while an earlier one is being converted always comes first.
- If you go back to AudioCaptor 0.2.0, a profile with an MP3 format records WAV there. The format works again after you update, unless you saved that profile in 0.2.0.

## [0.2.0] — 2026-11-26

### Added

- Recordings can now be saved as Opus, a compressed format that takes many times less space than WAV. Edit a profile and choose its new Format: "WAV (uncompressed)", which stays the default, "Opus — speech (mono)", about 14 MB per hour, or "Opus — music (stereo)", about 43 MB per hour; an hour of WAV at 48 kHz takes 691 MB. The recording is still written as WAV. When you stop it, AudioCaptor converts it in the background (an hour takes about half a minute on a recent PC), checks that the new `.opus` file can be decoded and lasts exactly as long as the recording, and only then deletes the WAV. Recordings are converted one at a time, in the order you stopped them, and you can start a new recording meanwhile.
- If a recording cannot be converted, it is not lost: the WAV stays, no unfinished `.opus` file is left, the warning sound plays (unless another recording is running, so that the sound does not get into it) and a message, which the screen reader reads, says where the WAV is.
- The screen reader says when a conversion starts and when all conversions are finished. `Alt+I` now also says that a conversion is running, about how long is left for all of them and how many files are waiting.
- If you exit while recordings are being converted or are waiting for it, AudioCaptor asks first. Nothing is lost if you exit: the WAV stays whole, no unfinished `.opus` file is left, and the conversion is finished the next time AudioCaptor starts, which the screen reader says. The setting that turns this question off is the one for recordings, now called "Confirm exit during recording or conversion".
- Opus works at every sample rate of a profile. A 44.1 kHz recording is converted to 48 kHz, because Opus has no 44.1 kHz. At 16 and 8 kHz the sample rate itself limits the quality, so the files are smaller there (speech 24 and 16 kbit/s, music 48 and 32 kbit/s instead of 32 and 96): a higher bitrate would not sound better.

### Changed

- When you update, the first start of AudioCaptor 0.2.0 moves `settings.json` to a new version (from 5 to 6). Your profiles and settings stay as they were, and every profile gets the Format "WAV (uncompressed)", so recording works as before until you choose Opus.
- If you go back to AudioCaptor 0.1.x, it opens the new `settings.json` with your profiles and settings in place, but it does not know the Format and records WAV. Once it saves its settings (for example, after you change a setting or a profile), the profiles lose their Format; after you update again, choose the format once more.

### Fixed

- Pressing `Alt+I` again in the same state now announces the status again. Before, the screen reader stayed silent the second time: "Ready" twice in a row, or the conversion status pressed twice within a few seconds.

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

[0.4.0]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.4.0
[0.3.0]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.3.0
[0.2.0]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.2.0
[0.1.2]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.2
[0.1.1]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.1
[0.1.0]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.0
