# Phase 1 MVP: Audio Recording — Design Spec

> Date: 2026-03-30
> Status: Approved
> Source: [phase-1.md](../../phases/phase-1.md), [PRD](../../requirements/PRD.md), [stack](../../stack-tauri-v2.md)

---

## 1. Overview

Minimum viable product: record audio from microphone and/or system loopback into a WAV file, with a Svelte 5 UI, portable exe, and screen reader accessibility.

**Scaffolding:** `npm create tauri-app@latest -- --template svelte-ts`, then customize.

---

## 2. Audio Pipeline Architecture

```
┌─────────────┐     ringbuf      ┌─────────────┐     hound      ┌──────────┐
│ Capture:Mic  │───────────────→ │             │──────────────→ │          │
│ (WASAPI thd) │                 │ Mixer Thread │               │ WAV File │
└─────────────┘                 │  - resample  │               └──────────┘
                                │  - scale_amp │
┌─────────────┐     ringbuf     │  - add_amp   │
│ Capture:Loop │───────────────→ │  - clip_amp  │
│ (WASAPI thd) │                 └──────┬──────┘
└─────────────┘                        │
                              command channel (crossbeam)
                              ┌────────┴────────┐
                              │   Tauri IPC      │
                              └────────┬────────┘
                              ┌────────▼────────┐
                              │   Svelte 5 UI    │
                              └─────────────────┘
```

### Components

1. **Capture threads** (2x) — each runs in its own `std::thread`, opens a WASAPI `AudioClient` in shared mode, reads audio packets into a `ringbuf` producer. Mic capture uses standard input device. Loopback capture uses render device with `AUDCLNT_STREAMFLAGS_LOOPBACK`.

2. **Mixer thread** — reads from both ring buffer consumers. If sample rates differ, resamples via `rubato` to the target rate. Applies `dasp::Signal::scale_amp()` per source, `add_amp()` to mix, `clip_amp()` to clamp. Writes to `hound::WavWriter`. Receives commands via `crossbeam::channel`.

3. **WAV writer** — owned by mixer thread. `hound::WavWriter` with configurable `WavSpec` (sample rate, 16-bit PCM, 1 or 2 channels). Finalized on Stop command.

### Ring Buffers

- Crate: `ringbuf` (lock-free SPSC)
- One ring buffer per capture source (mic, loopback)
- Buffer size: ~100ms of audio at source sample rate (enough to absorb jitter without adding latency)

---

## 3. State Management

### Tauri-managed state

```rust
struct AppState {
    recording_state: RecordingState,  // Idle | Recording | Paused
    selected_mic: Option<String>,      // device ID
    selected_loopback: Option<String>, // device ID
    mic_volume: f32,                   // 0.0–4.0, default 1.0
    loopback_volume: f32,              // 0.0–4.0, default 0.5
    output_mode: OutputMode,           // Mic | Loopback | Mix
    recording_duration_ms: u64,
}
```

Wrapped in `Mutex<AppState>`, registered via `app.manage()`. IPC commands read/write this state.

### Audio command channel

```rust
enum AudioCommand {
    Start {
        mic_device: Option<String>,
        loopback_device: Option<String>,
        output_mode: OutputMode,
        mic_volume: f32,
        loopback_volume: f32,
        sample_rate: u32,
        output_path: PathBuf,
    },
    Pause,
    Resume,
    Stop,
    SetMicVolume(f32),
    SetLoopbackVolume(f32),
}
```

Sent via `crossbeam::channel::bounded`. Audio threads own their internal running/paused flags — no shared mutex with UI.

### FSM

```
Idle ──Start──→ Recording ──Pause──→ Paused
                    ↑                    │
                    └───Resume───────────┘
                    │                    │
                    └───Stop────→ Idle ←─┘
```

On Pause: mixer thread continues draining ring buffers (prevents desync) but skips WAV writes.

---

## 4. Output Modes

| Mode       | Sources captured       | Files written |
|------------|------------------------|---------------|
| Microphone | Mic only               | 1 WAV (mic)   |
| Loopback   | Loopback only          | 1 WAV (loop)  |
| Mix        | Mic + Loopback         | 1 WAV (mixed) |

File naming: `recording_{YYYY-MM-DD_HH-MM-SS}.wav`
Default directory: `{exe_dir}/Recordings/`

---

## 5. IPC Interface

### Commands (frontend → backend)

| Command              | Parameters                        | Returns            |
|----------------------|-----------------------------------|--------------------|
| `get_audio_devices`  | —                                 | `Vec<AudioDevice>` |
| `start_recording`    | mic_id?, loopback_id?, mode, sample_rate | `Result<(), Error>` |

`start_recording` validates that required devices exist for the selected output mode before launching audio threads (FR3.13). Returns an error if a required device is missing or unavailable.

| `pause_recording`    | —                                 | `Result<(), Error>` |
| `resume_recording`   | —                                 | `Result<(), Error>` |
| `stop_recording`     | —                                 | `Result<(), Error>` |
| `set_mic_volume`     | volume: f32                       | `()`               |
| `set_loopback_volume`| volume: f32                       | `()`               |
| `load_settings`      | —                                 | `Settings`         |
| `save_settings`      | settings: Settings                | `Result<(), Error>` |

### Events (backend → frontend)

| Event                       | Payload                                    |
|-----------------------------|--------------------------------------------|
| `recording-state-changed`   | `{ state: string, duration_ms: u64 }`      |

Emitted periodically (~4 Hz) during recording for timer updates.

---

## 6. Rust Module Structure

```
src-tauri/
  .cargo/
    config.toml          # target-feature=+crt-static
  src/
    main.rs              # #![windows_subsystem = "windows"], calls lib::run()
    lib.rs               # Tauri builder setup, plugin init, IPC registration
    audio/
      mod.rs             # re-exports
      capture.rs         # WASAPI capture thread (mic + loopback)
      mixer.rs           # mixer thread: resample, mix, clip, write
      writer.rs          # hound WavWriter wrapper
      devices.rs         # device enumeration via wasapi::DeviceCollection
      types.rs           # AudioCommand, OutputMode, AudioDevice, ring buffer factory
    state.rs             # AppState, RecordingState, Tauri managed state
    settings.rs          # Settings struct, read/write settings.json
    portable.rs          # exe_dir(), ensure_dirs() for first-run setup
  Cargo.toml             # all dependencies, release profile
  tauri.conf.json        # decorations:true, capabilities, window config
  capabilities/
    default.json         # core + event permissions
  build.rs
```

---

## 7. Frontend Structure (Svelte 5 + TypeScript)

```
src/
  index.html
  main.ts                # mount App
  App.svelte             # layout, keyboard navigation
  app.css                # global styles, a11y utilities
  lib/
    components/
      DeviceSelect.svelte      # <select> with aria-label, role
      VolumeSlider.svelte      # <input type="range"> 0.0–4.0
      RecordControls.svelte    # Start/Pause/Stop buttons
      StatusIndicator.svelte   # Recording state + duration timer
    stores/
      recording.svelte.ts      # $state runes for recording state
      devices.svelte.ts        # $state runes for device lists
    types/
      index.ts                 # AudioDevice, RecordingState, OutputMode, Settings
    utils/
      invoke.ts                # typed wrappers around @tauri-apps/api invoke
```

### Key UI decisions

- Language: English only (hardcoded, no i18n infra in phase 1)
- All interactive elements have `role`, `aria-label`
- Tab navigation between all controls, Enter to activate, Escape for dialogs
- `aria-live="polite"` region for state change announcements
- Svelte compile-time a11y warnings enabled (default in Svelte 5)

---

## 8. Portable App Setup

### First-run initialization (`portable.rs`)

```
{exe_dir}/
  AudioCaptor.exe
  settings.json          # created if missing, defaults
  logs/                  # created if missing
  Recordings/            # created if missing
```

### Static CRT linking (`.cargo/config.toml`)

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

### Release profile (`Cargo.toml`)

```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

### Build command

```
pnpm tauri build --no-bundle
```

Produces standalone exe without NSIS/MSI installer.

---

## 9. Plugins

| Plugin                         | Purpose                                |
|--------------------------------|----------------------------------------|
| `tauri-plugin-log`             | File logging with rotation to {exe}/logs/ |
| `tauri-plugin-single-instance` | Prevent duplicate instances             |

---

## 10. Dependencies

### Rust (`Cargo.toml`)

| Crate          | Version  | Purpose                      |
|----------------|----------|------------------------------|
| `tauri`        | 2        | Framework core               |
| `serde`        | 1        | Serialization                |
| `serde_json`   | 1        | JSON settings                |
| `wasapi`       | 0.17     | WASAPI audio capture         |
| `hound`        | 3.5      | WAV file writing             |
| `dasp_sample`  | 0.11     | Sample type conversions      |
| `dasp_frame`   | 0.11     | Mono/stereo frame ops        |
| `dasp_signal`  | 0.11     | scale_amp, add_amp, clip_amp |
| `rubato`       | 0.16     | Sample rate conversion       |
| `ringbuf`      | 0.4      | Lock-free SPSC ring buffer   |
| `crossbeam-channel` | 0.5 | Command channel to audio     |
| `thiserror`    | 2        | Typed errors for IPC         |
| `anyhow`       | 1        | Internal error handling       |
| `chrono`       | 0.4      | Timestamp for file naming    |
| `log`          | 0.4      | Logging facade               |
| `tauri-plugin-log` | 2    | File logging                 |
| `tauri-plugin-single-instance` | 2 | Single instance       |

### Frontend (`package.json`)

| Package               | Purpose              |
|-----------------------|----------------------|
| `@tauri-apps/api`     | IPC invoke + events  |
| `@tauri-apps/cli`     | Build tooling (dev)  |
| `vite`                | Bundler              |
| `@sveltejs/vite-plugin-svelte` | Svelte integration |
| `svelte`              | UI framework         |
| `typescript`          | Type checking        |

---

## 11. Non-Functional Requirements

| Requirement | How addressed |
|-------------|---------------|
| NR3: CPU < 1% during recording | Lock-free ring buffers, WASAPI shared mode, no allocations in audio path |
| NR5: Latency < 20ms | WASAPI shared mode (~10ms) + lock-free mixer pipeline |
| NR6: File logging with rotation | tauri-plugin-log with Folder target |
| NR7: Simultaneous mic + loopback | Separate capture threads, independent ring buffers |
| NR8: Seamless pause/resume | Mixer drains buffers on pause, just skips WAV writes |

---

## 12. Definition of Done

See [phase-1.md](../../phases/phase-1.md#definition-of-done) — all 18 acceptance criteria apply.
