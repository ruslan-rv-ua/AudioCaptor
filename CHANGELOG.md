# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Starting AudioCaptor again while it is already running now brings its window to the front, even when it is
  hidden in the tray or minimized to the taskbar; before, nothing happened and the screen reader said nothing.
  Recording and pause are not affected.
- When a dialog closes (About, Settings, a profile, or the delete or exit confirmation), keyboard focus now goes
  back to where it was when the dialog opened, for example the button that opened it, and the screen reader reads
  it. Before, focus always jumped to the About button or the profile list, and after Cancel in the exit
  confirmation during a recording it was lost. When that place is no longer available, focus goes to the profile
  list, or to the recording buttons while recording.

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
- CI/CD: GitHub Actions workflows for CI, Release, and Scoop manifest update
- Scoop package manifest in [ruslan-rv-ua/scoop-bucket](https://github.com/ruslan-rv-ua/scoop-bucket)

[0.1.0]: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.0
