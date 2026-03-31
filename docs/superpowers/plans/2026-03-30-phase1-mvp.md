# Phase 1 MVP: Audio Recording — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a portable Windows app that records audio from microphone and/or system loopback into WAV files, with a Svelte 5 UI and screen reader accessibility.

**Architecture:** Tauri v2 with Rust backend handling WASAPI audio capture in dedicated threads, lock-free ring buffers feeding a mixer thread, and crossbeam command channels for control. Svelte 5 frontend communicates via Tauri IPC commands and events.

**Tech Stack:** Tauri v2, Svelte 5, TypeScript, Vite, Rust (wasapi, hound, dasp, rubato, ringbuf, crossbeam-channel)

**Spec:** [2026-03-30-phase1-mvp-design.md](../specs/2026-03-30-phase1-mvp-design.md)

---

## Task 1: Scaffold Project & Configure Build

**Files:**
- Create: entire project structure via `create tauri-app`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/.cargo/config.toml`
- Modify: `package.json`
- Create: `justfile`
- Create: `.gitignore`

- [ ] **Step 1: Scaffold Tauri + Svelte 5 project**

Run from `c:\dev\AudioCaptor`:

```bash
pnpm dlx create-tauri-app@latest . --template svelte-ts --manager pnpm
```

If the directory is not empty (due to existing docs), accept overwrite prompts. The template creates `src/`, `src-tauri/`, `package.json`, `vite.config.ts`, etc.

- [ ] **Step 2: Install dependencies**

```bash
pnpm install
```

- [ ] **Step 3: Verify scaffold compiles**

```bash
pnpm tauri dev
```

The app window should appear. Close it. This confirms the base scaffold works.

- [ ] **Step 4: Configure `src-tauri/Cargo.toml` — add all dependencies**

Replace the `[dependencies]` section with:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-log = "2"
tauri-plugin-single-instance = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasapi = "0.17"
hound = "3.5"
dasp_sample = "0.11"
dasp_frame = "0.11"
dasp_signal = "0.11"
rubato = "0.16"
ringbuf = "0.4"
crossbeam-channel = "0.5"
thiserror = "2"
anyhow = "1"
chrono = "0.4"
log = "0.4"
```

Add release profiles at the bottom:

```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true

[profile.release-fast]
inherits = "release"
opt-level = 1
lto = false
codegen-units = 16
strip = false
panic = "unwind"
```

- [ ] **Step 5: Create `src-tauri/.cargo/config.toml` for static CRT**

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

- [ ] **Step 6: Configure `src-tauri/tauri.conf.json`**

Key changes to the generated config:
- Set `"productName": "AudioCaptor"`
- Set `"identifier": "com.audiocaptor.app"`
- Ensure `"decorations": true` in the window config (critical for a11y — Tauri Issue #12901)
- Set window `"title": "AudioCaptor"`, `"width": 480`, `"height": 600`
- Set CSP: `"default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';"`
- In `bundle`, set `"targets": "all"` (the `--no-bundle` flag at build time overrides this)

- [ ] **Step 7: Configure `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default permissions for AudioCaptor",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-close",
    "core:window:allow-set-title",
    "core:event:default",
    "core:event:allow-emit",
    "core:event:allow-listen",
    "log:default",
    "single-instance:default"
  ]
}
```

- [ ] **Step 8: Update `package.json` scripts**

```json
{
  "scripts": {
    "tauri": "tauri",
    "dev": "tauri dev",
    "build": "tauri build --no-bundle",
    "build:fast": "tauri build --no-bundle -- --profile release-fast",
    "vite:dev": "vite dev --port 1420",
    "vite:build": "vite build"
  }
}
```

- [ ] **Step 9: Create `justfile`**

```just
# AudioCaptor — Tauri v2 app

default:
    @just --list

# Start Tauri dev server
dev:
    pnpm tauri dev

# Production build — minimal exe, slow compile
build:
    pnpm tauri build --no-bundle

# Fast build — larger exe, quick compile
build-fast:
    pnpm tauri build --no-bundle -- --profile release-fast

# Frontend-only dev server
vite-dev:
    pnpm vite dev --port 1420

# Clean Rust build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml

# Install JS dependencies
install:
    pnpm install
```

- [ ] **Step 10: Update `.gitignore`**

Ensure it includes:

```
node_modules/
dist/
src-tauri/target/
.env
*.pem
*.key
```

- [ ] **Step 11: Verify the project compiles with all dependencies**

```bash
cd src-tauri && cargo check
```

Expected: compiles with no errors (warnings from unused imports are fine at this stage).

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 + Svelte 5 project with all dependencies"
```

---

## Task 2: Portable Utilities & Settings

**Files:**
- Create: `src-tauri/src/portable.rs`
- Create: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/lib.rs` — register modules

- [ ] **Step 1: Create `src-tauri/src/portable.rs`**

```rust
use std::fs;
use std::path::PathBuf;

/// Returns the directory containing the running executable.
pub fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Cannot determine exe directory"))
}

/// Creates required directories and default files on first run.
/// Called during Tauri setup.
pub fn ensure_dirs() -> anyhow::Result<()> {
    let base = exe_dir()?;

    let dirs = ["logs", "Recordings"];
    for dir in &dirs {
        let path = base.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)?;
            log::info!("Created directory: {}", path.display());
        }
    }

    let settings_path = base.join("settings.json");
    if !settings_path.exists() {
        let defaults = crate::settings::Settings::default();
        let json = serde_json::to_string_pretty(&defaults)?;
        fs::write(&settings_path, json)?;
        log::info!("Created default settings.json");
    }

    Ok(())
}
```

- [ ] **Step 2: Create `src-tauri/src/settings.rs`**

```rust
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub selected_mic: Option<String>,
    pub selected_loopback: Option<String>,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub output_mode: String,
    pub sample_rate: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_mic: None,
            selected_loopback: None,
            mic_volume: 1.0,
            loopback_volume: 0.5,
            output_mode: "mix".to_string(),
            sample_rate: 48000,
        }
    }
}

fn settings_path() -> anyhow::Result<PathBuf> {
    Ok(crate::portable::exe_dir()?.join("settings.json"))
}

pub fn read_settings() -> Settings {
    let path = match settings_path() {
        Ok(p) => p,
        Err(_) => return Settings::default(),
    };
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn write_settings(settings: &Settings) -> anyhow::Result<()> {
    let path = settings_path()?;
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json)?;
    Ok(())
}
```

- [ ] **Step 3: Wire modules into `src-tauri/src/lib.rs`**

Replace the content of `lib.rs` with a minimal skeleton:

```rust
pub mod portable;
pub mod settings;

use tauri::Manager;

#[tauri::command]
fn load_settings() -> settings::Settings {
    settings::read_settings()
}

#[tauri::command]
fn save_settings(new_settings: settings::Settings) -> Result<(), String> {
    settings::write_settings(&new_settings).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: portable::exe_dir().unwrap_or_default().join("logs"),
                        file_name: Some("audiocaptor".into()),
                    },
                ))
                .max_file_size(1_000_000) // 1 MB rotation
                .build(),
        )
        .setup(|_app| {
            portable::ensure_dirs().map_err(|e| e.to_string())?;
            log::info!("AudioCaptor started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Update `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    audio_captor_lib::run()
}
```

Note: the lib name must match what's in `Cargo.toml` `[lib]` section. Update `Cargo.toml` `[lib]`:

```toml
[lib]
name = "audio_captor_lib"
crate-type = ["lib", "cdylib", "staticlib"]
```

- [ ] **Step 5: Verify it compiles and runs**

```bash
pnpm tauri dev
```

Expected: app window opens. Close it. Check that `logs/` directory was created next to the exe (in `src-tauri/target/debug/`). Check `settings.json` was created.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/portable.rs src-tauri/src/settings.rs src-tauri/src/lib.rs src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "feat: add portable path utilities, settings, logging, single-instance"
```

---

## Task 3: Audio Types & Device Enumeration

**Files:**
- Create: `src-tauri/src/audio/mod.rs`
- Create: `src-tauri/src/audio/types.rs`
- Create: `src-tauri/src/audio/devices.rs`
- Modify: `src-tauri/src/lib.rs` — add module, IPC command

- [ ] **Step 1: Create `src-tauri/src/audio/types.rs`**

```rust
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum OutputMode {
    Microphone,
    Loopback,
    Mix,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool, // true = mic, false = output (for loopback)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
}

#[derive(Debug)]
pub enum AudioCommand {
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

- [ ] **Step 2: Create `src-tauri/src/audio/devices.rs`**

```rust
use crate::audio::types::AudioDevice;
use wasapi::*;

/// Enumerates all active audio input devices (microphones).
pub fn list_input_devices() -> anyhow::Result<Vec<AudioDevice>> {
    let collection = DeviceCollection::new(&Direction::Capture)?;
    let count = collection.get_nbr_devices()?;
    let mut devices = Vec::new();

    for i in 0..count {
        let device = collection.get_device_at_index(i)?;
        let id = device.get_id()?;
        let name = device.get_friendlyname()?;
        devices.push(AudioDevice {
            id,
            name,
            is_input: true,
        });
    }

    Ok(devices)
}

/// Enumerates all active audio output devices (for loopback capture).
pub fn list_output_devices() -> anyhow::Result<Vec<AudioDevice>> {
    let collection = DeviceCollection::new(&Direction::Render)?;
    let count = collection.get_nbr_devices()?;
    let mut devices = Vec::new();

    for i in 0..count {
        let device = collection.get_device_at_index(i)?;
        let id = device.get_id()?;
        let name = device.get_friendlyname()?;
        devices.push(AudioDevice {
            id,
            name,
            is_input: false,
        });
    }

    Ok(devices)
}

/// Returns all audio devices (inputs + outputs).
pub fn list_all_devices() -> anyhow::Result<Vec<AudioDevice>> {
    let mut all = list_input_devices()?;
    all.extend(list_output_devices()?);
    Ok(all)
}

/// Find a device by its ID in the given direction.
pub fn get_device_by_id(id: &str, direction: &Direction) -> anyhow::Result<Device> {
    let collection = DeviceCollection::new(direction)?;
    let count = collection.get_nbr_devices()?;

    for i in 0..count {
        let device = collection.get_device_at_index(i)?;
        if device.get_id()? == id {
            return Ok(device);
        }
    }

    anyhow::bail!("Device not found: {}", id)
}
```

- [ ] **Step 3: Create `src-tauri/src/audio/mod.rs`**

```rust
pub mod types;
pub mod devices;
```

- [ ] **Step 4: Add `get_audio_devices` IPC command to `lib.rs`**

Add to `lib.rs`:

```rust
pub mod audio;
```

Add the command function:

```rust
#[tauri::command]
fn get_audio_devices() -> Result<Vec<audio::types::AudioDevice>, String> {
    audio::devices::list_all_devices().map_err(|e| e.to_string())
}
```

Register it in the invoke handler:

```rust
.invoke_handler(tauri::generate_handler![
    load_settings,
    save_settings,
    get_audio_devices,
])
```

- [ ] **Step 5: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/audio/
git commit -m "feat: add audio types, device enumeration via WASAPI"
```

---

## Task 4: WAV Writer

**Files:**
- Create: `src-tauri/src/audio/writer.rs`
- Modify: `src-tauri/src/audio/mod.rs`

- [ ] **Step 1: Create `src-tauri/src/audio/writer.rs`**

```rust
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

pub struct AudioWriter {
    writer: WavWriter<BufWriter<File>>,
    path: PathBuf,
}

impl AudioWriter {
    /// Create a new WAV writer at the given path.
    /// Channels: 1 (mono) or 2 (stereo). Sample rate in Hz.
    pub fn new(path: PathBuf, sample_rate: u32, channels: u16) -> anyhow::Result<Self> {
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let writer = WavWriter::create(&path, spec)?;
        log::info!("WAV writer created: {}", path.display());
        Ok(Self { writer, path })
    }

    /// Write a slice of f32 samples (interleaved if stereo).
    /// Converts f32 [-1.0, 1.0] to i16.
    pub fn write_samples(&mut self, samples: &[f32]) -> anyhow::Result<()> {
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            self.writer.write_sample(int_sample)?;
        }
        Ok(())
    }

    /// Finalize the WAV file (writes header with correct sizes).
    pub fn finalize(self) -> anyhow::Result<PathBuf> {
        self.writer.finalize()?;
        log::info!("WAV file finalized: {}", self.path.display());
        Ok(self.path)
    }
}
```

- [ ] **Step 2: Add to `src-tauri/src/audio/mod.rs`**

```rust
pub mod types;
pub mod devices;
pub mod writer;
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio/writer.rs src-tauri/src/audio/mod.rs
git commit -m "feat: add WAV writer with f32-to-i16 conversion"
```

---

## Task 5: WASAPI Capture Threads

**Files:**
- Create: `src-tauri/src/audio/capture.rs`
- Modify: `src-tauri/src/audio/mod.rs`

This is the most complex audio component. Each capture thread opens a WASAPI AudioClient, reads audio packets, converts to f32, and pushes into a ring buffer.

- [ ] **Step 1: Create `src-tauri/src/audio/capture.rs`**

```rust
use ringbuf::traits::{Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wasapi::*;

/// Handle returned from starting a capture thread.
/// Drop this or set `running` to false to stop the thread.
pub struct CaptureHandle {
    pub running: Arc<AtomicBool>,
    pub thread: Option<std::thread::JoinHandle<()>>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl CaptureHandle {
    /// Signal the capture thread to stop and wait for it.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start capturing from a microphone (input device) in a background thread.
/// Returns a CaptureHandle and the consumer end of the ring buffer.
pub fn start_mic_capture(
    device_id: &str,
) -> anyhow::Result<(CaptureHandle, ringbuf::HeapCons<f32>)> {
    let device = crate::audio::devices::get_device_by_id(device_id, &Direction::Capture)?;
    start_capture(device, false)
}

/// Start capturing loopback (system audio) from a render device.
/// Returns a CaptureHandle and the consumer end of the ring buffer.
pub fn start_loopback_capture(
    device_id: &str,
) -> anyhow::Result<(CaptureHandle, ringbuf::HeapCons<f32>)> {
    let device = crate::audio::devices::get_device_by_id(device_id, &Direction::Render)?;
    start_capture(device, true)
}

fn start_capture(
    device: Device,
    loopback: bool,
) -> anyhow::Result<(CaptureHandle, ringbuf::HeapCons<f32>)> {
    // Get the device's mix format
    let mut audio_client = device.get_iaudioclient()?;
    let format = audio_client.get_mixformat()?;

    let sample_rate = format.get_samplespersec();
    let channels = format.get_nchannels();
    let bits_per_sample = format.get_bitspersample();
    let block_align = format.get_blockalign();

    log::info!(
        "Capture format: {}Hz, {}ch, {}bit, loopback={}",
        sample_rate, channels, bits_per_sample, loopback
    );

    // Shared mode, 100ms buffer
    let sharemode = ShareMode::Shared;
    let desired_period = audio_client.get_periods()?.1; // default period

    if loopback {
        audio_client.initialize_client(
            &format,
            desired_period,
            &Direction::Capture,
            &sharemode,
            true, // loopback
        )?;
    } else {
        audio_client.initialize_client(
            &format,
            desired_period,
            &Direction::Capture,
            &sharemode,
            false,
        )?;
    }

    let capture_client = audio_client.get_audiocaptureclient()?;
    let event = audio_client.set_get_eventhandle()?;

    // Ring buffer: ~200ms of f32 samples
    let rb_size = (sample_rate as usize) * (channels as usize) / 5; // 200ms
    let rb = HeapRb::<f32>::new(rb_size);
    let (producer, consumer) = rb.split();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let thread = std::thread::Builder::new()
        .name(if loopback {
            "capture-loopback".into()
        } else {
            "capture-mic".into()
        })
        .spawn(move || {
            capture_loop(
                audio_client,
                capture_client,
                event,
                producer,
                running_clone,
                channels,
                bits_per_sample,
                block_align,
            );
        })?;

    let handle = CaptureHandle {
        running,
        thread: Some(thread),
        sample_rate,
        channels,
    };

    Ok((handle, consumer))
}

fn capture_loop(
    audio_client: AudioClient,
    capture_client: AudioCaptureClient,
    event: Handle,
    mut producer: ringbuf::HeapProd<f32>,
    running: Arc<AtomicBool>,
    channels: u16,
    bits_per_sample: u16,
    block_align: u16,
) {
    if let Err(e) = audio_client.start_stream() {
        log::error!("Failed to start capture stream: {}", e);
        return;
    }

    while running.load(Ordering::Relaxed) {
        // Wait for audio data (timeout 100ms)
        if event.wait_for_event(100).is_err() {
            continue;
        }

        match capture_client.get_next_nbr_frames() {
            Ok(Some(nbr_frames)) if nbr_frames > 0 => {
                match capture_client.read_from_device(
                    nbr_frames as usize,
                    block_align as usize,
                ) {
                    Ok((data, _flags)) => {
                        let samples = bytes_to_f32(
                            &data,
                            bits_per_sample,
                            channels,
                        );
                        let pushed = producer.push_slice(&samples);
                        if pushed < samples.len() {
                            log::warn!(
                                "Ring buffer overflow: dropped {} samples",
                                samples.len() - pushed
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Capture read error: {}", e);
                    }
                }
            }
            Ok(_) => {} // no frames ready
            Err(e) => {
                log::error!("Capture frame count error: {}", e);
            }
        }
    }

    let _ = audio_client.stop_stream();
    log::info!("Capture thread stopped");
}

/// Convert raw bytes from WASAPI to f32 samples in [-1.0, 1.0].
fn bytes_to_f32(data: &[u8], bits_per_sample: u16, channels: u16) -> Vec<f32> {
    match bits_per_sample {
        16 => {
            data.chunks_exact(2)
                .map(|chunk| {
                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                    sample as f32 / i16::MAX as f32
                })
                .collect()
        }
        32 => {
            // Could be f32 (IEEE float) or i32 — WASAPI shared mode typically uses f32
            data.chunks_exact(4)
                .map(|chunk| {
                    f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                })
                .collect()
        }
        24 => {
            data.chunks_exact(3)
                .map(|chunk| {
                    // Sign-extend 24-bit to 32-bit
                    let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]) >> 8;
                    sample as f32 / (1 << 23) as f32
                })
                .collect()
        }
        _ => {
            log::warn!("Unsupported bit depth: {}", bits_per_sample);
            vec![]
        }
    }
}
```

- [ ] **Step 2: Add to `src-tauri/src/audio/mod.rs`**

```rust
pub mod types;
pub mod devices;
pub mod writer;
pub mod capture;
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Fix any API mismatches with the actual `wasapi` crate version. The `wasapi` crate API may differ slightly — check docs at [docs.rs/wasapi](https://docs.rs/wasapi) and adjust method names/signatures as needed. Key areas to verify:
- `Device::get_iaudioclient()` may return `IAudioClient` directly
- `AudioClient::initialize_client()` signature — check if loopback is a separate flag or part of stream flags
- `AudioCaptureClient::read_from_device()` return type
- `Handle::wait_for_event()` API

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio/capture.rs src-tauri/src/audio/mod.rs
git commit -m "feat: add WASAPI capture threads for mic and loopback"
```

---

## Task 6: Mixer Thread

**Files:**
- Create: `src-tauri/src/audio/mixer.rs`
- Modify: `src-tauri/src/audio/mod.rs`

The mixer thread reads from ring buffer consumers, resamples if needed, applies volume/mixing/clipping, and writes to the WAV file.

- [ ] **Step 1: Create `src-tauri/src/audio/mixer.rs`**

```rust
use crate::audio::types::{AudioCommand, OutputMode};
use crate::audio::writer::AudioWriter;
use crossbeam_channel::Receiver;
use ringbuf::traits::Consumer;
use rubato::{FftFixedIn, Resampler};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct MixerConfig {
    pub mic_consumer: Option<ringbuf::HeapCons<f32>>,
    pub loopback_consumer: Option<ringbuf::HeapCons<f32>>,
    pub mic_sample_rate: u32,
    pub loopback_sample_rate: u32,
    pub mic_channels: u16,
    pub loopback_channels: u16,
    pub target_sample_rate: u32,
    pub target_channels: u16,
    pub output_mode: OutputMode,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub writer: AudioWriter,
    pub command_rx: Receiver<AudioCommand>,
}

pub struct MixerHandle {
    pub running: Arc<AtomicBool>,
    pub thread: Option<std::thread::JoinHandle<()>>,
}

impl MixerHandle {
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for MixerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start the mixer thread. Returns a handle to control it.
pub fn start_mixer(config: MixerConfig) -> anyhow::Result<MixerHandle> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let thread = std::thread::Builder::new()
        .name("mixer".into())
        .spawn(move || {
            mixer_loop(config, running_clone);
        })?;

    Ok(MixerHandle {
        running,
        thread: Some(thread),
    })
}

/// Create a rubato resampler if source rate differs from target rate.
fn maybe_resampler(
    source_rate: u32,
    target_rate: u32,
    channels: usize,
    chunk_size: usize,
) -> Option<FftFixedIn<f32>> {
    if source_rate == target_rate {
        return None;
    }
    FftFixedIn::new(source_rate as usize, target_rate as usize, chunk_size, 1, channels).ok()
}

/// Convert mono samples to stereo by duplicating each sample.
fn mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(mono.len() * 2);
    for &s in mono {
        stereo.push(s);
        stereo.push(s);
    }
    stereo
}

/// Deinterleave interleaved samples into per-channel vectors (for rubato).
fn deinterleave(interleaved: &[f32], channels: usize) -> Vec<Vec<f32>> {
    let mut result: Vec<Vec<f32>> = (0..channels).map(|_| Vec::new()).collect();
    for (i, &sample) in interleaved.iter().enumerate() {
        result[i % channels].push(sample);
    }
    result
}

/// Interleave per-channel vectors back into a single buffer.
fn interleave(channels: &[Vec<f32>]) -> Vec<f32> {
    if channels.is_empty() {
        return vec![];
    }
    let frames = channels[0].len();
    let num_ch = channels.len();
    let mut result = Vec::with_capacity(frames * num_ch);
    for frame in 0..frames {
        for ch in channels {
            result.push(ch[frame]);
        }
    }
    result
}

/// Resample a buffer using the given resampler, if present.
/// Input/output are interleaved. Returns resampled interleaved buffer,
/// or the original if no resampling needed.
fn resample_if_needed(
    buf: &[f32],
    channels: usize,
    resampler: &mut Option<FftFixedIn<f32>>,
) -> Vec<f32> {
    match resampler {
        Some(ref mut r) => {
            let deint = deinterleave(buf, channels);
            match r.process(&deint, None) {
                Ok(resampled) => interleave(&resampled),
                Err(e) => {
                    log::error!("Resample error: {}", e);
                    buf.to_vec()
                }
            }
        }
        None => buf.to_vec(),
    }
}

fn mixer_loop(mut config: MixerConfig, running: Arc<AtomicBool>) {
    let mut paused = false;
    let mut mic_volume = config.mic_volume;
    let mut loopback_volume = config.loopback_volume;
    let target_channels = config.target_channels as usize;

    // Read buffer — sized for ~10ms at max source sample rate
    let max_sr = config.mic_sample_rate.max(config.loopback_sample_rate);
    let chunk_frames = (max_sr as usize) / 100; // 10ms worth of frames
    let chunk_size = chunk_frames * 2; // max stereo interleaved
    let mut mic_buf = vec![0.0f32; chunk_size];
    let mut loopback_buf = vec![0.0f32; chunk_size];

    // Create resamplers if sample rates differ from target
    let mut mic_resampler = maybe_resampler(
        config.mic_sample_rate,
        config.target_sample_rate,
        config.mic_channels as usize,
        chunk_frames,
    );
    let mut loopback_resampler = maybe_resampler(
        config.loopback_sample_rate,
        config.target_sample_rate,
        config.loopback_channels as usize,
        chunk_frames,
    );

    log::info!(
        "Mixer thread started, mode={:?}, target={}Hz/{}ch, mic={}Hz/{}ch, loop={}Hz/{}ch",
        config.output_mode,
        config.target_sample_rate, target_channels,
        config.mic_sample_rate, config.mic_channels,
        config.loopback_sample_rate, config.loopback_channels,
    );

    while running.load(Ordering::Relaxed) {
        // Check for commands (non-blocking)
        while let Ok(cmd) = config.command_rx.try_recv() {
            match cmd {
                AudioCommand::Pause => {
                    paused = true;
                    log::info!("Mixer paused");
                }
                AudioCommand::Resume => {
                    paused = false;
                    log::info!("Mixer resumed");
                }
                AudioCommand::Stop => {
                    log::info!("Mixer received Stop");
                    running.store(false, Ordering::Relaxed);
                    break;
                }
                AudioCommand::SetMicVolume(v) => mic_volume = v,
                AudioCommand::SetLoopbackVolume(v) => loopback_volume = v,
                _ => {}
            }
        }

        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Read available samples from ring buffers
        let mic_read = if let Some(ref mut consumer) = config.mic_consumer {
            consumer.pop_slice(&mut mic_buf)
        } else {
            0
        };

        let loopback_read = if let Some(ref mut consumer) = config.loopback_consumer {
            consumer.pop_slice(&mut loopback_buf)
        } else {
            0
        };

        // If no data available from either source, sleep briefly to avoid busy-waiting
        if mic_read == 0 && loopback_read == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        if paused {
            // Drain buffers but don't write — prevents desync
            continue;
        }

        // Process mic samples: resample + channel convert
        let mic_processed = if mic_read > 0 {
            let resampled = resample_if_needed(
                &mic_buf[..mic_read],
                config.mic_channels as usize,
                &mut mic_resampler,
            );
            // Convert to target channels if needed
            if config.mic_channels == 1 && target_channels == 2 {
                mono_to_stereo(&resampled)
            } else {
                resampled
            }
        } else {
            vec![]
        };

        // Process loopback samples: resample + channel convert
        let loop_processed = if loopback_read > 0 {
            let resampled = resample_if_needed(
                &loopback_buf[..loopback_read],
                config.loopback_channels as usize,
                &mut loopback_resampler,
            );
            if config.loopback_channels == 1 && target_channels == 2 {
                mono_to_stereo(&resampled)
            } else {
                resampled
            }
        } else {
            vec![]
        };

        // Apply volume, mix, clip, and write based on mode
        let output = match config.output_mode {
            OutputMode::Microphone => {
                mic_processed.iter().map(|&s| (s * mic_volume).clamp(-1.0, 1.0)).collect::<Vec<_>>()
            }
            OutputMode::Loopback => {
                loop_processed.iter().map(|&s| (s * loopback_volume).clamp(-1.0, 1.0)).collect::<Vec<_>>()
            }
            OutputMode::Mix => {
                let len = mic_processed.len().max(loop_processed.len());
                let mut out = Vec::with_capacity(len);
                for i in 0..len {
                    let mic_s = mic_processed.get(i).copied().unwrap_or(0.0) * mic_volume;
                    let loop_s = loop_processed.get(i).copied().unwrap_or(0.0) * loopback_volume;
                    out.push((mic_s + loop_s).clamp(-1.0, 1.0));
                }
                out
            }
        };

        if !output.is_empty() {
            if let Err(e) = config.writer.write_samples(&output) {
                log::error!("WAV write error: {}", e);
                break;
            }
        }
    }

    // Finalize WAV file
    match config.writer.finalize() {
        Ok(path) => log::info!("Recording saved: {}", path.display()),
        Err(e) => log::error!("Failed to finalize WAV: {}", e),
    }

    log::info!("Mixer thread stopped");
}
```

- [ ] **Step 2: Add to `src-tauri/src/audio/mod.rs`**

```rust
pub mod types;
pub mod devices;
pub mod writer;
pub mod capture;
pub mod mixer;
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio/mixer.rs src-tauri/src/audio/mod.rs
git commit -m "feat: add mixer thread with volume control, clipping, and WAV output"
```

---

## Task 7: App State & IPC Recording Commands

**Files:**
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs` — full IPC wiring

This task wires everything together: the app state manages the recording lifecycle, IPC commands control it from the frontend.

- [ ] **Step 1: Create `src-tauri/src/state.rs`**

```rust
use crate::audio::capture::CaptureHandle;
use crate::audio::mixer::MixerHandle;
use crate::audio::types::{AudioCommand, OutputMode, RecordingState};
use crossbeam_channel::Sender;
use std::sync::Mutex;

pub struct AppState {
    pub recording_state: RecordingState,
    pub selected_mic: Option<String>,
    pub selected_loopback: Option<String>,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub output_mode: OutputMode,
    pub recording_start_time: Option<std::time::Instant>,
    pub paused_duration: std::time::Duration,
    pub pause_start_time: Option<std::time::Instant>,
    // Active recording handles (None when idle)
    pub mic_capture: Option<CaptureHandle>,
    pub loopback_capture: Option<CaptureHandle>,
    pub mixer: Option<MixerHandle>,
    pub command_tx: Option<Sender<AudioCommand>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            recording_state: RecordingState::Idle,
            selected_mic: None,
            selected_loopback: None,
            mic_volume: 1.0,
            loopback_volume: 0.5,
            output_mode: OutputMode::Mix,
            recording_start_time: None,
            paused_duration: std::time::Duration::ZERO,
            pause_start_time: None,
            mic_capture: None,
            loopback_capture: None,
            mixer: None,
            command_tx: None,
        }
    }
}

impl AppState {
    /// Get the current recording duration in milliseconds, accounting for pauses.
    pub fn duration_ms(&self) -> u64 {
        match self.recording_start_time {
            Some(start) => {
                let elapsed = start.elapsed();
                let paused = self.paused_duration
                    + self
                        .pause_start_time
                        .map(|t| t.elapsed())
                        .unwrap_or_default();
                elapsed.saturating_sub(paused).as_millis() as u64
            }
            None => 0,
        }
    }
}

pub type SharedState = Mutex<AppState>;
```

- [ ] **Step 2: Rewrite `src-tauri/src/lib.rs` with full IPC commands**

```rust
pub mod audio;
pub mod portable;
pub mod settings;
pub mod state;

use audio::capture;
use audio::mixer::{self, MixerConfig};
use audio::types::{AudioCommand, AudioDevice, OutputMode, RecordingState};
use audio::writer::AudioWriter;
use state::SharedState;
use tauri::Manager;

#[tauri::command]
fn load_settings() -> settings::Settings {
    settings::read_settings()
}

#[tauri::command]
fn save_settings(new_settings: settings::Settings) -> Result<(), String> {
    settings::write_settings(&new_settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    audio::devices::list_all_devices().map_err(|e| e.to_string())
}

#[tauri::command]
fn start_recording(
    state: tauri::State<'_, SharedState>,
    app: tauri::AppHandle,
    mic_id: Option<String>,
    loopback_id: Option<String>,
    mode: OutputMode,
    sample_rate: u32,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if s.recording_state != RecordingState::Idle {
        return Err("Already recording".into());
    }

    // Validate devices for the selected mode (FR3.13)
    match mode {
        OutputMode::Microphone | OutputMode::Mix => {
            if mic_id.is_none() {
                return Err("Microphone device required for this mode".into());
            }
        }
        _ => {}
    }
    match mode {
        OutputMode::Loopback | OutputMode::Mix => {
            if loopback_id.is_none() {
                return Err("Loopback device required for this mode".into());
            }
        }
        _ => {}
    }

    // Generate output path
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("recording_{}.wav", timestamp);
    let output_path = portable::exe_dir()
        .map_err(|e| e.to_string())?
        .join("Recordings")
        .join(&filename);

    // Start capture threads as needed
    let (mic_handle, mic_consumer) = if let Some(ref id) = mic_id {
        match mode {
            OutputMode::Microphone | OutputMode::Mix => {
                let (handle, consumer) =
                    capture::start_mic_capture(id).map_err(|e| e.to_string())?;
                (Some(handle), Some(consumer))
            }
            _ => (None, None),
        }
    } else {
        (None, None)
    };

    let (loopback_handle, loopback_consumer) = if let Some(ref id) = loopback_id {
        match mode {
            OutputMode::Loopback | OutputMode::Mix => {
                let (handle, consumer) =
                    capture::start_loopback_capture(id).map_err(|e| e.to_string())?;
                (Some(handle), Some(consumer))
            }
            _ => (None, None),
        }
    } else {
        (None, None)
    };

    // Determine output channels (stereo)
    let channels = 2u16;

    // Create WAV writer
    let writer =
        AudioWriter::new(output_path, sample_rate, channels).map_err(|e| e.to_string())?;

    // Create command channel
    let (tx, rx) = crossbeam_channel::bounded::<AudioCommand>(32);

    // Determine sample rates from capture handles
    let mic_sr = mic_handle.as_ref().map(|h| h.sample_rate).unwrap_or(sample_rate);
    let loop_sr = loopback_handle
        .as_ref()
        .map(|h| h.sample_rate)
        .unwrap_or(sample_rate);

    let mic_ch = mic_handle.as_ref().map(|h| h.channels).unwrap_or(2);
    let loop_ch = loopback_handle.as_ref().map(|h| h.channels).unwrap_or(2);

    // Start mixer thread
    let mixer_config = MixerConfig {
        mic_consumer,
        loopback_consumer,
        mic_sample_rate: mic_sr,
        loopback_sample_rate: loop_sr,
        mic_channels: mic_ch,
        loopback_channels: loop_ch,
        target_sample_rate: sample_rate,
        target_channels: channels,
        output_mode: mode.clone(),
        mic_volume: s.mic_volume,
        loopback_volume: s.loopback_volume,
        writer,
        command_rx: rx,
    };

    let mixer_handle = mixer::start_mixer(mixer_config).map_err(|e| e.to_string())?;

    // Update state
    s.recording_state = RecordingState::Recording;
    s.recording_start_time = Some(std::time::Instant::now());
    s.paused_duration = std::time::Duration::ZERO;
    s.pause_start_time = None;
    s.output_mode = mode;
    s.mic_capture = mic_handle;
    s.loopback_capture = loopback_handle;
    s.mixer = Some(mixer_handle);
    s.command_tx = Some(tx);

    // Start emitting state events
    let app_clone = app.clone();
    let state_inner = app.state::<SharedState>().inner().clone();
    std::thread::Builder::new()
        .name("state-emitter".into())
        .spawn(move || {
            state_event_loop(app_clone, state_inner);
        })
        .map_err(|e| e.to_string())?;

    log::info!("Recording started");
    Ok(())
}

#[tauri::command]
fn pause_recording(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if s.recording_state != RecordingState::Recording {
        return Err("Not recording".into());
    }
    if let Some(ref tx) = s.command_tx {
        tx.send(AudioCommand::Pause).map_err(|e| e.to_string())?;
    }
    s.recording_state = RecordingState::Paused;
    s.pause_start_time = Some(std::time::Instant::now());
    log::info!("Recording paused");
    Ok(())
}

#[tauri::command]
fn resume_recording(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if s.recording_state != RecordingState::Paused {
        return Err("Not paused".into());
    }
    if let Some(ref tx) = s.command_tx {
        tx.send(AudioCommand::Resume).map_err(|e| e.to_string())?;
    }
    // Accumulate paused time
    if let Some(pause_start) = s.pause_start_time.take() {
        s.paused_duration += pause_start.elapsed();
    }
    s.recording_state = RecordingState::Recording;
    log::info!("Recording resumed");
    Ok(())
}

#[tauri::command]
fn stop_recording(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if s.recording_state == RecordingState::Idle {
        return Err("Not recording".into());
    }

    // Send stop command to mixer
    if let Some(ref tx) = s.command_tx {
        let _ = tx.send(AudioCommand::Stop);
    }

    // Stop capture threads
    if let Some(ref mut h) = s.mic_capture {
        h.stop();
    }
    if let Some(ref mut h) = s.loopback_capture {
        h.stop();
    }
    // Stop mixer
    if let Some(ref mut h) = s.mixer {
        h.stop();
    }

    // Clean up
    s.mic_capture = None;
    s.loopback_capture = None;
    s.mixer = None;
    s.command_tx = None;
    s.recording_state = RecordingState::Idle;
    s.recording_start_time = None;
    s.paused_duration = std::time::Duration::ZERO;
    s.pause_start_time = None;

    log::info!("Recording stopped");
    Ok(())
}

#[tauri::command]
fn set_mic_volume(state: tauri::State<'_, SharedState>, volume: f32) -> Result<(), String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    if let Some(ref tx) = s.command_tx {
        let _ = tx.send(AudioCommand::SetMicVolume(volume));
    }
    Ok(())
}

#[tauri::command]
fn set_loopback_volume(state: tauri::State<'_, SharedState>, volume: f32) -> Result<(), String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    if let Some(ref tx) = s.command_tx {
        let _ = tx.send(AudioCommand::SetLoopbackVolume(volume));
    }
    Ok(())
}

/// Background thread that emits recording state events at ~4 Hz.
fn state_event_loop(app: tauri::AppHandle, state: SharedState) {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(250));

        let s = match state.lock() {
            Ok(s) => s,
            Err(_) => break,
        };

        if s.recording_state == RecordingState::Idle && s.recording_start_time.is_none() {
            break;
        }

        let payload = serde_json::json!({
            "state": format!("{:?}", s.recording_state),
            "durationMs": s.duration_ms(),
        });

        drop(s); // release lock before emit

        let _ = app.emit("recording-state-changed", payload);
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: portable::exe_dir().unwrap_or_default().join("logs"),
                        file_name: Some("audiocaptor".into()),
                    },
                ))
                .max_file_size(1_000_000)
                .build(),
        )
        .manage(state::SharedState::default())
        .setup(|_app| {
            portable::ensure_dirs().map_err(|e| e.to_string())?;
            log::info!("AudioCaptor started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            get_audio_devices,
            start_recording,
            pause_recording,
            resume_recording,
            stop_recording,
            set_mic_volume,
            set_loopback_volume,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/lib.rs
git commit -m "feat: add app state, recording FSM, and all IPC commands"
```

---

## Task 8: Frontend Types & Stores

**Files:**
- Create: `src/lib/types/index.ts`
- Create: `src/lib/utils/invoke.ts`
- Create: `src/lib/stores/devices.svelte.ts`
- Create: `src/lib/stores/recording.svelte.ts`

- [ ] **Step 1: Create directory structure**

```bash
mkdir -p src/lib/types src/lib/utils src/lib/stores src/lib/components
```

- [ ] **Step 2: Create `src/lib/types/index.ts`**

```typescript
export interface AudioDevice {
  id: string;
  name: string;
  isInput: boolean;
}

export type RecordingState = "Idle" | "Recording" | "Paused";

export type OutputMode = "Microphone" | "Loopback" | "Mix";

export interface Settings {
  selectedMic: string | null;
  selectedLoopback: string | null;
  micVolume: number;
  loopbackVolume: number;
  outputMode: string;
  sampleRate: number;
}

export interface RecordingStateEvent {
  state: RecordingState;
  durationMs: number;
}
```

- [ ] **Step 3: Create `src/lib/utils/invoke.ts`**

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { AudioDevice, OutputMode, Settings } from "../types";

export async function getAudioDevices(): Promise<AudioDevice[]> {
  return invoke<AudioDevice[]>("get_audio_devices");
}

export async function startRecording(
  micId: string | null,
  loopbackId: string | null,
  mode: OutputMode,
  sampleRate: number
): Promise<void> {
  return invoke("start_recording", {
    micId,
    loopbackId,
    mode,
    sampleRate,
  });
}

export async function pauseRecording(): Promise<void> {
  return invoke("pause_recording");
}

export async function resumeRecording(): Promise<void> {
  return invoke("resume_recording");
}

export async function stopRecording(): Promise<void> {
  return invoke("stop_recording");
}

export async function setMicVolume(volume: number): Promise<void> {
  return invoke("set_mic_volume", { volume });
}

export async function setLoopbackVolume(volume: number): Promise<void> {
  return invoke("set_loopback_volume", { volume });
}

export async function loadSettings(): Promise<Settings> {
  return invoke<Settings>("load_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { newSettings: settings });
}
```

- [ ] **Step 4: Create `src/lib/stores/devices.svelte.ts`**

```typescript
import { getAudioDevices } from "../utils/invoke";
import type { AudioDevice } from "../types";

let devices = $state<AudioDevice[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

export function getDevices() {
  return {
    get all() { return devices; },
    get microphones() { return devices.filter((d) => d.isInput); },
    get loopbacks() { return devices.filter((d) => !d.isInput); },
    get loading() { return loading; },
    get error() { return error; },
  };
}

export async function refreshDevices() {
  loading = true;
  error = null;
  try {
    devices = await getAudioDevices();
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  } finally {
    loading = false;
  }
}
```

- [ ] **Step 5: Create `src/lib/stores/recording.svelte.ts`**

```typescript
import { listen } from "@tauri-apps/api/event";
import type { RecordingState, RecordingStateEvent, OutputMode } from "../types";
import * as api from "../utils/invoke";

let recordingState = $state<RecordingState>("Idle");
let durationMs = $state(0);
let selectedMic = $state<string | null>(null);
let selectedLoopback = $state<string | null>(null);
let micVolume = $state(1.0);
let loopbackVolume = $state(0.5);
let outputMode = $state<OutputMode>("Mix");
let sampleRate = $state(48000);
let recordingError = $state<string | null>(null);

export function getRecording() {
  return {
    get state() { return recordingState; },
    get durationMs() { return durationMs; },
    get selectedMic() { return selectedMic; },
    set selectedMic(v: string | null) { selectedMic = v; },
    get selectedLoopback() { return selectedLoopback; },
    set selectedLoopback(v: string | null) { selectedLoopback = v; },
    get micVolume() { return micVolume; },
    set micVolume(v: number) { micVolume = v; },
    get loopbackVolume() { return loopbackVolume; },
    set loopbackVolume(v: number) { loopbackVolume = v; },
    get outputMode() { return outputMode; },
    set outputMode(v: OutputMode) { outputMode = v; },
    get sampleRate() { return sampleRate; },
    set sampleRate(v: number) { sampleRate = v; },
    get error() { return recordingError; },
  };
}

export async function initRecordingListener() {
  await listen<RecordingStateEvent>("recording-state-changed", (event) => {
    recordingState = event.payload.state;
    durationMs = event.payload.durationMs;
  });
}

export async function startRecording() {
  recordingError = null;
  try {
    await api.startRecording(selectedMic, selectedLoopback, outputMode, sampleRate);
    recordingState = "Recording";
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function pauseRecording() {
  try {
    await api.pauseRecording();
    recordingState = "Paused";
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function resumeRecording() {
  try {
    await api.resumeRecording();
    recordingState = "Recording";
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function stopRecording() {
  try {
    await api.stopRecording();
    recordingState = "Idle";
    durationMs = 0;
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function updateMicVolume(volume: number) {
  micVolume = volume;
  if (recordingState !== "Idle") {
    await api.setMicVolume(volume);
  }
}

export async function updateLoopbackVolume(volume: number) {
  loopbackVolume = volume;
  if (recordingState !== "Idle") {
    await api.setLoopbackVolume(volume);
  }
}
```

- [ ] **Step 6: Verify frontend compiles**

```bash
pnpm vite:build
```

- [ ] **Step 7: Commit**

```bash
git add src/lib/
git commit -m "feat: add frontend types, IPC wrappers, and state stores"
```

---

## Task 9: UI Components

**Files:**
- Create: `src/lib/components/DeviceSelect.svelte`
- Create: `src/lib/components/VolumeSlider.svelte`
- Create: `src/lib/components/RecordControls.svelte`
- Create: `src/lib/components/StatusIndicator.svelte`

- [ ] **Step 1: Create `src/lib/components/DeviceSelect.svelte`**

```svelte
<script lang="ts">
  import type { AudioDevice } from "../types";

  interface Props {
    label: string;
    devices: AudioDevice[];
    value: string | null;
    onchange: (id: string | null) => void;
    disabled?: boolean;
  }

  let { label, devices, value, onchange, disabled = false }: Props = $props();

  function handleChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    onchange(target.value || null);
  }
</script>

<div class="device-select">
  <label for={label.toLowerCase().replace(/\s/g, "-")}>
    {label}
  </label>
  <select
    id={label.toLowerCase().replace(/\s/g, "-")}
    aria-label={label}
    {disabled}
    onchange={handleChange}
  >
    <option value="">-- Select {label} --</option>
    {#each devices as device}
      <option value={device.id} selected={device.id === value}>
        {device.name}
      </option>
    {/each}
  </select>
</div>

<style>
  .device-select {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label {
    font-weight: 600;
    font-size: 0.875rem;
  }
  select {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.875rem;
    background: #fff;
  }
  select:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }
  select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
```

- [ ] **Step 2: Create `src/lib/components/VolumeSlider.svelte`**

```svelte
<script lang="ts">
  interface Props {
    label: string;
    value: number;
    onchange: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
  }

  let {
    label,
    value,
    onchange,
    min = 0,
    max = 4.0,
    step = 0.1,
    disabled = false,
  }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onchange(parseFloat(target.value));
  }
</script>

<div class="volume-slider">
  <label for={label.toLowerCase().replace(/\s/g, "-")}>
    {label}: {value.toFixed(1)}
  </label>
  <input
    id={label.toLowerCase().replace(/\s/g, "-")}
    type="range"
    aria-label="{label} volume"
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value}
    {min}
    {max}
    {step}
    {value}
    {disabled}
    oninput={handleInput}
  />
</div>

<style>
  .volume-slider {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label {
    font-weight: 600;
    font-size: 0.875rem;
  }
  input[type="range"] {
    width: 100%;
    cursor: pointer;
  }
  input[type="range"]:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }
  input[type="range"]:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
```

- [ ] **Step 3: Create `src/lib/components/RecordControls.svelte`**

```svelte
<script lang="ts">
  import type { RecordingState } from "../types";

  interface Props {
    state: RecordingState;
    onstart: () => void;
    onpause: () => void;
    onresume: () => void;
    onstop: () => void;
  }

  let { state, onstart, onpause, onresume, onstop }: Props = $props();
</script>

<div class="record-controls" role="group" aria-label="Recording controls">
  {#if state === "Idle"}
    <button
      onclick={onstart}
      aria-label="Start recording"
      class="btn btn-start"
    >
      Start
    </button>
  {:else if state === "Recording"}
    <button
      onclick={onpause}
      aria-label="Pause recording"
      class="btn btn-pause"
    >
      Pause
    </button>
    <button
      onclick={onstop}
      aria-label="Stop recording"
      class="btn btn-stop"
    >
      Stop
    </button>
  {:else if state === "Paused"}
    <button
      onclick={onresume}
      aria-label="Resume recording"
      class="btn btn-resume"
    >
      Resume
    </button>
    <button
      onclick={onstop}
      aria-label="Stop recording"
      class="btn btn-stop"
    >
      Stop
    </button>
  {/if}
</div>

<style>
  .record-controls {
    display: flex;
    gap: 8px;
    justify-content: center;
    padding: 16px 0;
  }
  .btn {
    padding: 12px 24px;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    min-width: 100px;
  }
  .btn:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }
  .btn-start {
    background: #22c55e;
    color: white;
  }
  .btn-start:hover { background: #16a34a; }
  .btn-pause {
    background: #f59e0b;
    color: white;
  }
  .btn-pause:hover { background: #d97706; }
  .btn-resume {
    background: #3b82f6;
    color: white;
  }
  .btn-resume:hover { background: #2563eb; }
  .btn-stop {
    background: #ef4444;
    color: white;
  }
  .btn-stop:hover { background: #dc2626; }
</style>
```

- [ ] **Step 4: Create `src/lib/components/StatusIndicator.svelte`**

```svelte
<script lang="ts">
  import type { RecordingState } from "../types";

  interface Props {
    state: RecordingState;
    durationMs: number;
  }

  let { state, durationMs }: Props = $props();

  let formattedDuration = $derived(formatDuration(durationMs));

  function formatDuration(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  }

  let stateLabel = $derived(
    state === "Idle"
      ? "Ready"
      : state === "Recording"
        ? "Recording"
        : "Paused"
  );

  let stateClass = $derived(state.toLowerCase());
</script>

<div
  class="status-indicator"
  role="status"
  aria-live="polite"
  aria-label="Recording status: {stateLabel}, duration {formattedDuration}"
>
  <span class="state-badge {stateClass}">{stateLabel}</span>
  {#if state !== "Idle"}
    <span class="duration" aria-label="Recording duration">{formattedDuration}</span>
  {/if}
</div>

<style>
  .status-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 12px;
    font-size: 1.25rem;
  }
  .state-badge {
    padding: 4px 12px;
    border-radius: 4px;
    font-weight: 600;
    font-size: 0.875rem;
    text-transform: uppercase;
  }
  .idle { background: #e5e7eb; color: #374151; }
  .recording { background: #fecaca; color: #dc2626; }
  .paused { background: #fef3c7; color: #d97706; }
  .duration {
    font-family: monospace;
    font-size: 1.5rem;
    font-weight: 700;
  }
</style>
```

- [ ] **Step 5: Verify frontend compiles**

```bash
pnpm vite:build
```

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/
git commit -m "feat: add UI components — DeviceSelect, VolumeSlider, RecordControls, StatusIndicator"
```

---

## Task 10: App Layout & Main Wiring

**Files:**
- Modify: `src/App.svelte` — full layout
- Modify: `src/app.css` (or create) — global styles + a11y
- Modify: `src/main.ts` — mount

- [ ] **Step 1: Create/update `src/app.css`**

```css
:root {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
    Ubuntu, Cantarell, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: #1a1a1a;
  background-color: #ffffff;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
  min-height: 100vh;
}

/* Visually hidden but accessible to screen readers */
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
}

/* Focus styles for keyboard navigation */
:focus-visible {
  outline: 2px solid #0066cc;
  outline-offset: 2px;
}
```

- [ ] **Step 2: Rewrite `src/App.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import type { OutputMode } from "./lib/types";
  import { getDevices, refreshDevices } from "./lib/stores/devices.svelte";
  import {
    getRecording,
    initRecordingListener,
    startRecording,
    pauseRecording,
    resumeRecording,
    stopRecording,
    updateMicVolume,
    updateLoopbackVolume,
  } from "./lib/stores/recording.svelte";
  import DeviceSelect from "./lib/components/DeviceSelect.svelte";
  import VolumeSlider from "./lib/components/VolumeSlider.svelte";
  import RecordControls from "./lib/components/RecordControls.svelte";
  import StatusIndicator from "./lib/components/StatusIndicator.svelte";

  const devices = getDevices();
  const recording = getRecording();

  const outputModes: { value: OutputMode; label: string }[] = [
    { value: "Microphone", label: "Microphone only" },
    { value: "Loopback", label: "System audio only" },
    { value: "Mix", label: "Mix (Mic + System)" },
  ];

  const sampleRates = [8000, 16000, 44100, 48000];

  let isRecording = $derived(recording.state !== "Idle");

  onMount(async () => {
    await refreshDevices();
    await initRecordingListener();
  });

  function handleOutputModeChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    recording.outputMode = target.value as OutputMode;
  }

  function handleSampleRateChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    recording.sampleRate = parseInt(target.value);
  }
</script>

<main role="application" aria-label="AudioCaptor">
  <h1>AudioCaptor</h1>

  <StatusIndicator state={recording.state} durationMs={recording.durationMs} />

  <section aria-label="Audio devices">
    <DeviceSelect
      label="Microphone"
      devices={devices.microphones}
      value={recording.selectedMic}
      onchange={(id) => (recording.selectedMic = id)}
      disabled={isRecording}
    />

    <DeviceSelect
      label="Loopback Device"
      devices={devices.loopbacks}
      value={recording.selectedLoopback}
      onchange={(id) => (recording.selectedLoopback = id)}
      disabled={isRecording}
    />
  </section>

  <section aria-label="Output settings">
    <div class="setting-row">
      <label for="output-mode">Output Mode</label>
      <select
        id="output-mode"
        aria-label="Output mode"
        disabled={isRecording}
        onchange={handleOutputModeChange}
      >
        {#each outputModes as mode}
          <option value={mode.value} selected={mode.value === recording.outputMode}>
            {mode.label}
          </option>
        {/each}
      </select>
    </div>

    <div class="setting-row">
      <label for="sample-rate">Sample Rate</label>
      <select
        id="sample-rate"
        aria-label="Sample rate"
        disabled={isRecording}
        onchange={handleSampleRateChange}
      >
        {#each sampleRates as rate}
          <option value={rate} selected={rate === recording.sampleRate}>
            {rate} Hz
          </option>
        {/each}
      </select>
    </div>
  </section>

  <section aria-label="Volume controls">
    <VolumeSlider
      label="Microphone Volume"
      value={recording.micVolume}
      onchange={updateMicVolume}
    />

    <VolumeSlider
      label="Loopback Volume"
      value={recording.loopbackVolume}
      onchange={updateLoopbackVolume}
    />
  </section>

  <RecordControls
    state={recording.state}
    onstart={startRecording}
    onpause={pauseRecording}
    onresume={resumeRecording}
    onstop={stopRecording}
  />

  {#if recording.error}
    <div class="error" role="alert" aria-live="assertive">
      {recording.error}
    </div>
  {/if}

  <!-- Live region for screen reader announcements -->
  <div id="live-region" aria-live="polite" aria-atomic="true" class="visually-hidden"></div>
</main>

<style>
  main {
    max-width: 480px;
    margin: 0 auto;
    padding: 24px 16px;
  }

  h1 {
    text-align: center;
    margin: 0 0 16px;
    font-size: 1.5rem;
  }

  section {
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .setting-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .setting-row label {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .setting-row select {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.875rem;
    background: #fff;
  }

  .setting-row select:focus-visible {
    outline: 2px solid #0066cc;
    outline-offset: 2px;
  }

  .setting-row select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error {
    padding: 12px;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 6px;
    color: #dc2626;
    text-align: center;
    margin-top: 8px;
  }
</style>
```

- [ ] **Step 3: Update `src/main.ts`**

Ensure it imports the CSS and mounts the app:

```typescript
import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
```

- [ ] **Step 4: Update `src/index.html`**

Ensure it has the app mount point and proper lang/meta:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>AudioCaptor</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 5: Verify full app compiles and runs**

```bash
pnpm tauri dev
```

Expected: the app window opens with device selects, volume sliders, output mode selector, sample rate selector, and Start button. Devices should populate from system audio devices.

- [ ] **Step 6: Commit**

```bash
git add src/App.svelte src/app.css src/main.ts src/index.html
git commit -m "feat: add main app layout with all UI components wired to backend"
```

---

## Task 11: Integration Testing & Polish

**Files:**
- Modify: various files for bug fixes found during testing

This is a manual integration testing task. Run through the full recording workflow.

- [ ] **Step 1: Run the app**

```bash
pnpm tauri dev
```

- [ ] **Step 2: Test device enumeration**

- Open the app
- Verify microphone dropdown shows available input devices
- Verify loopback dropdown shows available output devices
- If no devices show, check the console/logs for errors

- [ ] **Step 3: Test microphone recording**

- Select a microphone from the dropdown
- Set output mode to "Microphone only"
- Click Start
- Speak into the microphone for ~5 seconds
- Click Stop
- Check `src-tauri/target/debug/Recordings/` for the WAV file
- Play the WAV file — verify audio is audible and not corrupted

- [ ] **Step 4: Test loopback recording**

- Play some music/audio on the system
- Select a loopback device
- Set output mode to "System audio only"
- Click Start, wait ~5 seconds, click Stop
- Play the WAV file — verify system audio was captured

- [ ] **Step 5: Test mix recording**

- Select both a mic and loopback device
- Set output mode to "Mix (Mic + System)"
- Click Start, speak while system audio plays, click Stop
- Play the WAV file — verify both sources are audible

- [ ] **Step 6: Test pause/resume**

- Start a recording
- Click Pause — verify timer stops
- Click Resume — verify timer continues
- Click Stop — play the file, verify no gaps or artifacts at pause point

- [ ] **Step 7: Test volume controls**

- Start a recording with mic volume at 4.0
- Verify no clipping distortion in output (clamp should work)
- Start a recording with mic volume at 0.0
- Verify silence from mic source

- [ ] **Step 8: Test keyboard navigation**

- Tab through all controls — every element should be reachable
- Enter/Space should activate buttons
- Verify focus indicators are visible

- [ ] **Step 9: Test portable setup**

Build the release exe:

```bash
pnpm build
```

Copy `src-tauri/target/release/AudioCaptor.exe` to a fresh directory. Run it. Verify:
- `settings.json` is created next to the exe
- `logs/` directory is created
- `Recordings/` directory is created
- Recording works from the portable location

- [ ] **Step 10: Test single instance**

Run `AudioCaptor.exe` twice. The second launch should focus the first window, not open a new one.

- [ ] **Step 11: Fix any issues found during testing**

Fix bugs, adjust API calls to match actual `wasapi` crate behavior, resolve any compile errors.

- [ ] **Step 12: Commit all fixes**

```bash
git add -A
git commit -m "fix: integration testing fixes and polish"
```

---

## Task 12: Final Verification & Cleanup

- [ ] **Step 1: Run `dumpbin /dependents` on the release exe**

```bash
dumpbin /dependents src-tauri/target/release/AudioCaptor.exe
```

Verify no unexpected DLL dependencies beyond Windows system DLLs (kernel32.dll, user32.dll, etc.). If `vcruntime140.dll` appears, the CRT static linking in `.cargo/config.toml` is not working — verify the config.

- [ ] **Step 2: Verify the exe size**

```bash
ls -la src-tauri/target/release/AudioCaptor.exe
```

Expected: 5-15 MB range.

- [ ] **Step 3: Clean up unused template files**

Remove any scaffold boilerplate that's no longer needed (template component files, example styles, etc.).

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "chore: cleanup scaffold boilerplate, verify release build"
```
