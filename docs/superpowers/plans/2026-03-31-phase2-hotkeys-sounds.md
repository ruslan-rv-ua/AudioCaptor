# Phase 2: Global Hotkeys & Sound Notifications — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable recording control from any Windows window via a global hotkey with audio feedback, add mnemonics, and validate devices before recording.

**Architecture:** Four independent subsystems — (1) sound notification engine via `rodio` with embedded OGG files, (2) global hotkey via `tauri-plugin-global-shortcut` with short/long press FSM, (3) frontend mnemonics via keydown handlers, (4) device validation with `DEVICE_NOT_FOUND` error code. The sound engine is built first because the hotkey subsystem depends on it for audio feedback.

**Tech Stack:** Rust (`rodio` with `vorbis` feature, `tauri-plugin-global-shortcut`), Svelte 5, Tauri 2 IPC

---

## File Structure

### New Files (Rust)
- `src-tauri/src/sounds.rs` — Sound notification engine: dedicated thread owns `OutputStream`, receives play commands via channel
- `src-tauri/src/hotkey.rs` — Global hotkey registration, short/long press FSM, dispatches recording commands
- `src-tauri/sounds/start.ogg` — Start recording sound (~10-30 KB)
- `src-tauri/sounds/pause.ogg` — Pause recording sound (~10-30 KB)
- `src-tauri/sounds/stop.ogg` — Stop recording sound (~10-30 KB)

### Modified Files (Rust)
- `src-tauri/Cargo.toml` — Add `rodio`, `tauri-plugin-global-shortcut` dependencies
- `src-tauri/src/lib.rs` — Register hotkey plugin, add new Tauri commands, wire up sounds + hotkey modules
- `src-tauri/src/settings.rs` — Add `hotkey` and `soundEnabled` fields to `Settings`
- `src-tauri/src/state.rs` — Add `sounds_enabled` field to `AppState`
- `src-tauri/tauri.conf.json` — Add `global-shortcut` plugin
- `src-tauri/capabilities/default.json` — Add `global-shortcut:default` permission

### Modified Files (Frontend)
- `src/lib/types/index.ts` — Add `hotkey` and `soundEnabled` to `Settings` type
- `src/lib/utils/invoke.ts` — Add `setHotkey`, `setSoundEnabled` invoke wrappers
- `src/lib/stores/recording.svelte.ts` — Add `hotkey`, `soundEnabled` state; listen for `recording-error` event
- `src/App.svelte` — Add hotkey settings UI section, sound toggle, mnemonic handler
- `src/lib/components/RecordControls.svelte` — Add mnemonic underlines for Alt+key

---

## Task 1: Add rodio dependency and sound notification engine

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/sounds/start.ogg`
- Create: `src-tauri/sounds/pause.ogg`
- Create: `src-tauri/sounds/stop.ogg`
- Create: `src-tauri/src/sounds.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod sounds;`)

- [ ] **Step 1: Add rodio dependency to Cargo.toml**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
rodio = { version = "0.20", default-features = false, features = ["vorbis"] }
```

- [ ] **Step 2: Generate three distinct OGG notification sounds**

Create `src-tauri/sounds/` directory and generate sounds with Python. Install deps first:

```bash
pip install numpy soundfile
```

Then run this script to generate all three sounds:

```python
import numpy as np
import soundfile as sf
import os

sr = 44100
os.makedirs("src-tauri/sounds", exist_ok=True)

def tone(freq, duration, sr=44100):
    t = np.linspace(0, duration, int(sr * duration), endpoint=False)
    # Apply fade-in/out envelope (10ms)
    env = np.ones_like(t)
    fade = int(0.01 * sr)
    env[:fade] = np.linspace(0, 1, fade)
    env[-fade:] = np.linspace(1, 0, fade)
    return (0.5 * np.sin(2 * np.pi * freq * t) * env).astype(np.float32)

# Start: ascending two-note chime (C5 → E5), ~200ms total
start = np.concatenate([tone(523, 0.1), tone(659, 0.15)])
sf.write("src-tauri/sounds/start.ogg", start, sr, format="OGG", subtype="VORBIS")

# Pause: two short identical beeps (A4), ~200ms total with gap
beep = tone(440, 0.06)
gap = np.zeros(int(sr * 0.04), dtype=np.float32)
pause = np.concatenate([beep, gap, beep])
sf.write("src-tauri/sounds/pause.ogg", pause, sr, format="OGG", subtype="VORBIS")

# Stop: descending two-note (E5 → C4), ~250ms total
stop = np.concatenate([tone(659, 0.1), tone(262, 0.15)])
sf.write("src-tauri/sounds/stop.ogg", stop, sr, format="OGG", subtype="VORBIS")

for f in ["start.ogg", "pause.ogg", "stop.ogg"]:
    size = os.path.getsize(f"src-tauri/sounds/{f}")
    print(f"{f}: {size} bytes")
```

Expected output: three OGG files, each < 30 KB, clearly distinguishable:
- `start.ogg` — ascending chime (C5→E5) — "go"
- `pause.ogg` — two short beeps (A4) — "wait"
- `stop.ogg` — descending tone (E5→C4) — "done"

- [ ] **Step 3: Create sounds.rs — sound notification engine**

Create `src-tauri/src/sounds.rs`:

**Important:** `rodio::OutputStream` is `!Send`, so it cannot live inside `AppState` (which is behind `Arc<Mutex<>>`). The solution: a dedicated thread owns the `OutputStream` and receives play commands via a channel. `SoundEngine` stores only the `Sender` (which IS `Send`).

```rust
use crossbeam_channel::{Sender, bounded};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

const START_SOUND: &[u8] = include_bytes!("../sounds/start.ogg");
const PAUSE_SOUND: &[u8] = include_bytes!("../sounds/pause.ogg");
const STOP_SOUND: &[u8] = include_bytes!("../sounds/stop.ogg");

#[derive(Debug, Clone, Copy)]
pub enum SoundKind {
    Start,
    Pause,
    Stop,
}

/// Thread-safe handle to the sound playback thread.
/// Stores only a Sender, so it is Send + Sync.
pub struct SoundEngine {
    tx: Sender<SoundKind>,
}

impl SoundEngine {
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = bounded::<SoundKind>(8);

        std::thread::Builder::new()
            .name("sound-engine".into())
            .spawn(move || {
                // OutputStream must be created on this thread and kept alive
                let (_stream, handle) = match OutputStream::try_default() {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Sound engine: failed to open output: {e}");
                        return;
                    }
                };

                while let Ok(kind) = rx.recv() {
                    let data: &[u8] = match kind {
                        SoundKind::Start => START_SOUND,
                        SoundKind::Pause => PAUSE_SOUND,
                        SoundKind::Stop => STOP_SOUND,
                    };

                    let cursor = Cursor::new(data);
                    let decoder = match Decoder::new(cursor) {
                        Ok(d) => d,
                        Err(e) => {
                            log::warn!("Failed to decode notification sound: {e}");
                            continue;
                        }
                    };

                    match Sink::try_new(&handle) {
                        Ok(sink) => {
                            sink.append(decoder);
                            sink.detach(); // Play to completion without blocking
                        }
                        Err(e) => {
                            log::warn!("Failed to create audio sink: {e}");
                        }
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn sound thread: {e}"))?;

        Ok(Self { tx })
    }

    /// Send a play command to the sound thread (non-blocking).
    pub fn play(&self, kind: SoundKind) {
        let _ = self.tx.try_send(kind);
    }
}
```

- [ ] **Step 4: Register sounds module in lib.rs**

In `src-tauri/src/lib.rs`, add at the top with other module declarations:

```rust
pub mod sounds;
```

- [ ] **Step 5: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors (rodio + embedded OGG files resolve correctly)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/sounds.rs src-tauri/src/lib.rs src-tauri/sounds/
git commit -m "feat(phase2): add rodio sound engine with embedded OGG notifications"
```

---

## Task 2: Extend Settings with hotkey and soundEnabled fields

**Files:**
- Modify: `src-tauri/src/settings.rs`
- Modify: `src/lib/types/index.ts`

- [ ] **Step 1: Add fields to Rust Settings struct**

In `src-tauri/src/settings.rs`, add two fields to the `Settings` struct:

```rust
pub struct Settings {
    pub selected_mic: Option<String>,
    pub selected_loopback: Option<String>,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub output_mode: String,
    pub sample_rate: u32,
    pub hotkey: String,           // NEW — default: "Pause"
    pub sound_enabled: bool,      // NEW — default: true
}
```

And update the `Default` impl:

```rust
impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_mic: None,
            selected_loopback: None,
            mic_volume: 1.0,
            loopback_volume: 0.5,
            output_mode: "mix".to_string(),
            sample_rate: 48000,
            hotkey: "Pause".to_string(),
            sound_enabled: true,
        }
    }
}
```

Note: `#[serde(default)]` on the struct ensures old `settings.json` files without these fields still deserialize correctly.

- [ ] **Step 2: Add fields to TypeScript Settings type**

In `src/lib/types/index.ts`, add to the `Settings` interface:

```typescript
export interface Settings {
  selectedMic: string | null;
  selectedLoopback: string | null;
  micVolume: number;
  loopbackVolume: number;
  outputMode: OutputMode;
  sampleRate: number;
  hotkey: string;          // NEW
  soundEnabled: boolean;   // NEW
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Run: `pnpm exec svelte-check`
Expected: Both pass without errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/settings.rs src/lib/types/index.ts
git commit -m "feat(phase2): add hotkey and soundEnabled to Settings"
```

---

## Task 3: Wire sound engine into AppState and play on recording state changes

**Files:**
- Modify: `src-tauri/src/state.rs` — Add `sound_engine` and `sounds_enabled` to `AppState`
- Modify: `src-tauri/src/lib.rs` — Play sounds in start/pause/resume/stop commands; add `set_sound_enabled` command
- Modify: `src/lib/utils/invoke.ts` — Add `setSoundEnabled` wrapper
- Modify: `src/lib/stores/recording.svelte.ts` — Add `soundEnabled` state

- [ ] **Step 1: Add sound engine to AppState**

In `src-tauri/src/state.rs`, add fields and initialize:

```rust
use crate::sounds::SoundEngine;

pub struct AppState {
    // ... existing fields ...
    pub sound_engine: Option<SoundEngine>,
    pub sounds_enabled: bool,
}
```

In `Default`:
```rust
sound_engine: None, // Initialized during app setup
sounds_enabled: true,
```

- [ ] **Step 2: Initialize sound engine in lib.rs setup**

In `src-tauri/src/lib.rs`, in the `.setup(|app| { ... })` block, after `ensure_dirs()`:

```rust
.setup(|app| {
    portable::ensure_dirs().map_err(|e| e.to_string())?;

    // Initialize sound engine
    let settings = settings::read_settings();
    if let Ok(mut s) = app.state::<SharedState>().lock() {
        match sounds::SoundEngine::new() {
            Ok(engine) => s.sound_engine = Some(engine),
            Err(e) => log::warn!("Sound engine init failed: {e}"),
        }
        s.sounds_enabled = settings.sound_enabled;
    }

    log::info!("AudioCaptor started");
    Ok(())
})
```

- [ ] **Step 3: Add helper to play sound conditionally**

In `src-tauri/src/lib.rs`, add a helper function:

```rust
fn play_sound(state: &state::AppState, kind: sounds::SoundKind) {
    if state.sounds_enabled {
        if let Some(ref engine) = state.sound_engine {
            engine.play(kind);
        }
    }
}
```

- [ ] **Step 4: Play sounds in recording commands**

In `start_recording`, after `s.recording_state = RecordingState::Recording;`:
```rust
play_sound(&s, sounds::SoundKind::Start);
```

In `pause_recording`, after `s.recording_state = RecordingState::Paused;`:
```rust
play_sound(&s, sounds::SoundKind::Pause);
```

In `resume_recording`, after `s.recording_state = RecordingState::Recording;`:
```rust
play_sound(&s, sounds::SoundKind::Start);
```

In `stop_recording`, inside the lock block, after `s.recording_state = RecordingState::Idle;`:
```rust
play_sound(&s, sounds::SoundKind::Stop);
```

- [ ] **Step 5: Add set_sound_enabled Tauri command**

In `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn set_sound_enabled(state: tauri::State<'_, SharedState>, enabled: bool) {
    if let Ok(mut s) = state.lock() {
        s.sounds_enabled = enabled;
    }
}
```

Register it in `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing ...
    set_sound_enabled,
])
```

- [ ] **Step 6: Add frontend invoke wrapper**

In `src/lib/utils/invoke.ts`:

```typescript
export async function setSoundEnabled(enabled: boolean): Promise<void> {
  return invoke("set_sound_enabled", { enabled });
}
```

- [ ] **Step 7: Add soundEnabled to recording store**

In `src/lib/stores/recording.svelte.ts`, add state variable:

```typescript
let soundEnabled = $state(true);
```

Add to the `getRecording()` return object:
```typescript
get soundEnabled() { return soundEnabled; },
set soundEnabled(v: boolean) { soundEnabled = v; },
```

Add an exported function:
```typescript
export async function updateSoundEnabled(enabled: boolean) {
  soundEnabled = enabled;
  await api.setSoundEnabled(enabled);
}
```

- [ ] **Step 8: Verify compilation**

Run: `cd src-tauri && cargo check`
Run: `pnpm exec svelte-check`
Expected: Both pass

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/lib.rs src/lib/utils/invoke.ts src/lib/stores/recording.svelte.ts
git commit -m "feat(phase2): wire sound engine into recording commands"
```

---

## Task 4: Add tauri-plugin-global-shortcut and hotkey FSM

**Files:**
- Modify: `src-tauri/Cargo.toml` — Add `tauri-plugin-global-shortcut`
- Modify: `src-tauri/tauri.conf.json` — Register plugin
- Modify: `src-tauri/capabilities/default.json` — Add permission
- Create: `src-tauri/src/hotkey.rs` — FSM for short/long press, registration/re-registration
- Modify: `src-tauri/src/lib.rs` — Register hotkey plugin, add `set_hotkey` command, initialize hotkey on setup

- [ ] **Step 1: Add global-shortcut plugin dependency**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
tauri-plugin-global-shortcut = "2"
```

- [ ] **Step 2: Add plugin to tauri.conf.json**

No changes needed — `tauri-plugin-global-shortcut` is registered via Rust code, not JSON config. The plugin auto-registers its permissions.

- [ ] **Step 3: Add permission to capabilities**

In `src-tauri/capabilities/default.json`, add to the `permissions` array:

```json
"global-shortcut:default"
```

- [ ] **Step 4: Create hotkey.rs — FSM and registration logic**

Create `src-tauri/src/hotkey.rs`:

```rust
use crate::audio::types::RecordingState;
use crate::state::SharedState;
use crate::{do_start_recording, do_pause_recording, do_resume_recording, do_stop_recording};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

/// Threshold in ms: press < LONG_PRESS_MS = short, >= LONG_PRESS_MS = long.
const LONG_PRESS_MS: u128 = 500;

/// Stores the timestamp of the most recent key press.
static PRESS_TIME: Mutex<Option<Instant>> = Mutex::new(None);

/// Register the global hotkey. Call during app setup and when hotkey changes.
pub fn register(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
    let manager = app.global_shortcut();

    // Unregister all existing shortcuts first (safe for re-registration)
    let _ = manager.unregister_all();

    let shortcut: Shortcut = shortcut_str
        .parse()
        .map_err(|e| format!("Invalid shortcut '{shortcut_str}': {e}"))?;

    let app_handle = app.clone();
    manager
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            handle_shortcut_event(&app_handle, event);
        })
        .map_err(|e| format!("Failed to register shortcut: {e}"))?;

    log::info!("Global hotkey registered: {shortcut_str}");
    Ok(())
}

fn handle_shortcut_event(app: &AppHandle, event: ShortcutEvent) {
    match event.state {
        ShortcutState::Pressed => {
            if let Ok(mut t) = PRESS_TIME.lock() {
                *t = Some(Instant::now());
            }
        }
        ShortcutState::Released => {
            let duration_ms = PRESS_TIME
                .lock()
                .ok()
                .and_then(|mut t| t.take())
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0);

            let state = match app.try_state::<SharedState>() {
                Some(s) => s,
                None => return,
            };

            let current_state = match state.lock() {
                Ok(s) => s.recording_state.clone(),
                Err(_) => return,
            };

            match current_state {
                RecordingState::Idle => {
                    // Any press duration → start
                    if let Err(e) = do_start_recording(app) {
                        log::error!("Hotkey start failed: {e}");
                        let _ = app.emit("recording-error", e);
                    }
                }
                RecordingState::Recording => {
                    if duration_ms < LONG_PRESS_MS {
                        // Short press → pause
                        if let Err(e) = do_pause_recording(app) {
                            log::error!("Hotkey pause failed: {e}");
                        }
                    } else {
                        // Long press → stop
                        if let Err(e) = do_stop_recording(app) {
                            log::error!("Hotkey stop failed: {e}");
                        }
                    }
                }
                RecordingState::Paused => {
                    if duration_ms < LONG_PRESS_MS {
                        // Short press → resume
                        if let Err(e) = do_resume_recording(app) {
                            log::error!("Hotkey resume failed: {e}");
                        }
                    } else {
                        // Long press → stop
                        if let Err(e) = do_stop_recording(app) {
                            log::error!("Hotkey stop failed: {e}");
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 5: Refactor recording commands into reusable functions**

The hotkey handler needs to call the same logic as the Tauri commands but from a non-command context (it has `AppHandle`, not `tauri::State`). Extract the core logic into `do_*` functions in `lib.rs`:

```rust
/// Core start logic, callable from both Tauri command and hotkey handler.
/// When called from hotkey, reads device IDs and mode from settings.json
/// (the frontend persists selections to settings.json on every change).
pub fn do_start_recording(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<SharedState>();

    // Read settings.json — the frontend saves device selections here on every change
    let settings = settings::read_settings();
    let mic_id = settings.selected_mic;
    let loopback_id = settings.selected_loopback;
    let mode: OutputMode = match settings.output_mode.as_str() {
        "Microphone" | "microphone" => OutputMode::Microphone,
        "Loopback" | "loopback" => OutputMode::Loopback,
        _ => OutputMode::Mix,
    };
    let sample_rate = settings.sample_rate;

    start_recording_inner(app, &state, mic_id, loopback_id, mode, sample_rate)
}

pub fn do_pause_recording(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<SharedState>();
    pause_recording_inner(&state)
}

pub fn do_resume_recording(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<SharedState>();
    resume_recording_inner(&state)
}

pub fn do_stop_recording(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<SharedState>();
    stop_recording_inner(&state)
}
```

The existing `#[tauri::command]` functions should delegate to these `*_inner` functions. For example:

```rust
#[tauri::command]
fn start_recording(
    state: tauri::State<'_, SharedState>,
    app: tauri::AppHandle,
    mic_id: Option<String>,
    loopback_id: Option<String>,
    mode: OutputMode,
    sample_rate: u32,
) -> Result<(), String> {
    start_recording_inner(&app, &state, mic_id, loopback_id, mode, sample_rate)
}

fn start_recording_inner(
    app: &tauri::AppHandle,
    state: &SharedState,
    mic_id: Option<String>,
    loopback_id: Option<String>,
    mode: OutputMode,
    sample_rate: u32,
) -> Result<(), String> {
    // ... existing start_recording body, but using `state` param instead of `state.lock()`
    // ... also emit state event at end via app.emit(...)
}
```

Similarly for `pause_recording_inner`, `resume_recording_inner`, `stop_recording_inner`.

The key change: the `start_recording_inner` function also needs the `AppHandle` for spawning the state emitter thread, so it takes `&AppHandle` as a parameter.

- [ ] **Step 6: Register hotkey module and plugin in lib.rs**

In `src-tauri/src/lib.rs`:

```rust
pub mod hotkey;
```

In the `run()` function, add the plugin and initialize in setup:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    // ... existing plugins ...
    .setup(|app| {
        portable::ensure_dirs().map_err(|e| e.to_string())?;

        let settings = settings::read_settings();

        // Initialize sound engine
        if let Ok(mut s) = app.state::<SharedState>().lock() {
            match sounds::SoundEngine::new() {
                Ok(engine) => s.sound_engine = Some(engine),
                Err(e) => log::warn!("Sound engine init failed: {e}"),
            }
            s.sounds_enabled = settings.sound_enabled;
        }

        // Register global hotkey
        if let Err(e) = hotkey::register(&app.handle(), &settings.hotkey) {
            log::error!("Failed to register hotkey: {e}");
        }

        log::info!("AudioCaptor started");
        Ok(())
    })
```

- [ ] **Step 7: Add set_hotkey Tauri command**

In `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn set_hotkey(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    hotkey::register(&app, &shortcut)
}
```

Register in `invoke_handler`:
```rust
set_hotkey,
```

- [ ] **Step 8: Add frontend invoke wrapper for set_hotkey**

In `src/lib/utils/invoke.ts`:

```typescript
export async function setHotkey(shortcut: string): Promise<void> {
  return invoke("set_hotkey", { shortcut });
}
```

- [ ] **Step 9: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors

- [ ] **Step 10: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/capabilities/default.json src-tauri/src/hotkey.rs src-tauri/src/lib.rs src/lib/utils/invoke.ts
git commit -m "feat(phase2): add global hotkey with short/long press FSM"
```

---

## Task 5: Emit recording-error event and handle frontend-side

**Files:**
- Modify: `src-tauri/src/lib.rs` — Emit `recording-error` on device validation failure
- Modify: `src/lib/stores/recording.svelte.ts` — Listen for `recording-error` event
- Modify: `src/lib/types/index.ts` — (no changes needed, error is a string)

- [ ] **Step 1: Emit recording-error in hotkey handler**

The `hotkey.rs` handler already emits `recording-error` on start failure (from Task 4, Step 4). The `start_recording_inner` function already returns device validation errors like `"Microphone device required for this mode"`.

For a more structured error, wrap the error with a code. In `src-tauri/src/lib.rs`, in `start_recording_inner`, change the device validation errors to include a code:

```rust
// Existing validation already returns descriptive errors.
// Add DEVICE_NOT_FOUND prefix for frontend parsing:
OutputMode::Microphone | OutputMode::Mix => {
    if mic_id.is_none() {
        return Err("DEVICE_NOT_FOUND: Microphone device required for this mode".into());
    }
}
```

- [ ] **Step 2: Listen for recording-error in frontend**

In `src/lib/stores/recording.svelte.ts`, in `initRecordingListener()`:

```typescript
export async function initRecordingListener() {
  await listen<RecordingStateEvent>("recording-state-changed", (event) => {
    recordingState = event.payload.state;
    durationMs = event.payload.durationMs;
  });

  // Listen for errors from hotkey-triggered actions
  await listen<string>("recording-error", (event) => {
    recordingError = event.payload;
  });
}
```

- [ ] **Step 3: Verify compilation**

Run: `pnpm exec svelte-check`
Expected: Passes

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src/lib/stores/recording.svelte.ts
git commit -m "feat(phase2): emit and handle recording-error events from hotkey"
```

---

## Task 6: Frontend — Hotkey settings UI and sound toggle

**Files:**
- Modify: `src/lib/stores/recording.svelte.ts` — Add `hotkey` state, load from settings
- Modify: `src/App.svelte` — Add hotkey display/change UI, sound toggle checkbox

- [ ] **Step 1: Add hotkey state to recording store**

In `src/lib/stores/recording.svelte.ts`, add:

```typescript
let hotkey = $state("Pause");
```

Add to `getRecording()`:
```typescript
get hotkey() { return hotkey; },
set hotkey(v: string) { hotkey = v; },
```

Add exported function:
```typescript
export async function updateHotkey(shortcut: string) {
  hotkey = shortcut;
  await api.setHotkey(shortcut);
}
```

- [ ] **Step 2: Load hotkey and soundEnabled from settings on mount**

In `src/lib/stores/recording.svelte.ts`, add a function to load all settings:

```typescript
export async function loadSettingsIntoStore() {
  const s = await api.loadSettings();
  selectedMic = s.selectedMic;
  selectedLoopback = s.selectedLoopback;
  micVolume = s.micVolume;
  loopbackVolume = s.loopbackVolume;
  outputMode = s.outputMode;
  sampleRate = s.sampleRate;
  hotkey = s.hotkey;
  soundEnabled = s.soundEnabled;
}
```

- [ ] **Step 3: Add settings UI to App.svelte**

In `src/App.svelte`, add a new section after the volume controls section and before `RecordControls`:

```svelte
<section aria-label="Settings">
  <div class="setting-row">
    <label for="hotkey-display">Global Hotkey</label>
    <div class="hotkey-row">
      <input
        id="hotkey-display"
        type="text"
        value={recording.hotkey}
        readonly
        aria-label="Current hotkey"
        class="hotkey-input"
      />
      <button
        type="button"
        class="btn btn-small"
        onclick={startHotkeyCapture}
        disabled={isRecording || capturingHotkey}
      >
        {capturingHotkey ? "Press a key..." : "Change"}
      </button>
    </div>
  </div>

  <div class="setting-row">
    <label>
      <input
        type="checkbox"
        checked={recording.soundEnabled}
        onchange={(e) => {
          const target = e.target as HTMLInputElement;
          updateSoundEnabled(target.checked);
        }}
      />
      Sound notifications
    </label>
  </div>
</section>
```

Add the hotkey capture logic in the `<script>`:

```typescript
import { updateSoundEnabled, updateHotkey, loadSettingsIntoStore } from "./lib/stores/recording.svelte";

let capturingHotkey = $state(false);

function startHotkeyCapture() {
  capturingHotkey = true;

  function onKeyDown(e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();

    // Ignore modifier-only presses
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");

    // Map key names to Tauri shortcut format
    let key = e.key;
    if (key === " ") key = "Space";
    else if (key.length === 1) key = key.toUpperCase();
    else if (key === "Pause") key = "Pause";
    // Keep other special keys as-is (F1-F12, etc.)

    parts.push(key);
    const shortcut = parts.join("+");

    updateHotkey(shortcut);
    capturingHotkey = false;
    window.removeEventListener("keydown", onKeyDown, true);
  }

  window.addEventListener("keydown", onKeyDown, true);
}
```

Update `onMount` to load settings:

```typescript
onMount(async () => {
  await loadSettingsIntoStore();
  await refreshDevices();
  await initRecordingListener();
});
```

- [ ] **Step 4: Add CSS for hotkey row**

In `src/App.svelte` `<style>`:

```css
.hotkey-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.hotkey-input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 0.875rem;
  background: #f5f5f5;
  cursor: default;
}

.btn-small {
  padding: 8px 12px;
  border: none;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  background: #6b7280;
  color: white;
}

.btn-small:hover { background: #4b5563; }
.btn-small:disabled { opacity: 0.6; cursor: not-allowed; }
```

- [ ] **Step 5: Verify compilation**

Run: `pnpm exec svelte-check`
Expected: Passes

- [ ] **Step 6: Commit**

```bash
git add src/App.svelte src/lib/stores/recording.svelte.ts
git commit -m "feat(phase2): add hotkey settings UI and sound toggle"
```

---

## Task 7: Persist hotkey and sound settings to settings.json

**Files:**
- Modify: `src/App.svelte` — Save settings on change

- [ ] **Step 1: Add settings save on relevant changes**

In `src/App.svelte`, add an `$effect` that saves settings whenever relevant values change. Use a debounced approach to avoid excessive writes:

```typescript
import { saveSettings } from "./lib/utils/invoke";

let saveTimeout: ReturnType<typeof setTimeout> | undefined;

function scheduleSave() {
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(async () => {
    await saveSettings({
      selectedMic: recording.selectedMic,
      selectedLoopback: recording.selectedLoopback,
      micVolume: recording.micVolume,
      loopbackVolume: recording.loopbackVolume,
      outputMode: recording.outputMode,
      sampleRate: recording.sampleRate,
      hotkey: recording.hotkey,
      soundEnabled: recording.soundEnabled,
    });
  }, 500);
}
```

Call `scheduleSave()` after hotkey change and sound toggle change. The simplest way: add it to `updateHotkey` and `updateSoundEnabled` in the store, or call it inline in the event handlers.

If the app already has settings persistence from Phase 1, integrate into that existing mechanism rather than duplicating.

- [ ] **Step 2: Verify end-to-end**

1. Change hotkey via UI → check `settings.json` updates
2. Toggle sound → check `settings.json` updates
3. Restart app → verify settings restored

- [ ] **Step 3: Commit**

```bash
git add src/App.svelte
git commit -m "feat(phase2): persist hotkey and sound settings to settings.json"
```

---

## Task 8: Mnemonics — Alt+letter hotkeys for UI controls

**Files:**
- Modify: `src/App.svelte` — Add keydown handler for Alt+letter mnemonics
- Modify: `src/lib/components/RecordControls.svelte` — Add mnemonic underline styles

- [ ] **Step 1: Define mnemonic mapping**

In `src/App.svelte`, define the mnemonic keys (using Ukrainian-friendly Latin letters or common keys):

```typescript
// Mnemonics: Alt+S = Start, Alt+P = Pause, Alt+R = Resume, Alt+T = Stop
function handleMnemonic(e: KeyboardEvent) {
  if (!e.altKey) return;

  const key = e.key.toLowerCase();
  switch (key) {
    case "s":
      if (recording.state === "Idle") { e.preventDefault(); startRecording(); }
      break;
    case "p":
      if (recording.state === "Recording") { e.preventDefault(); pauseRecording(); }
      break;
    case "r":
      if (recording.state === "Paused") { e.preventDefault(); resumeRecording(); }
      break;
    case "t":
      if (recording.state !== "Idle") { e.preventDefault(); stopRecording(); }
      break;
  }
}
```

- [ ] **Step 2: Attach keydown listener**

In `src/App.svelte`, in `onMount`:

```typescript
onMount(async () => {
  await loadSettingsIntoStore();
  await refreshDevices();
  await initRecordingListener();
  window.addEventListener("keydown", handleMnemonic);
  return () => window.removeEventListener("keydown", handleMnemonic);
});
```

Note: Svelte 5 `onMount` return value is a cleanup function.

- [ ] **Step 3: Add mnemonic underlines to buttons**

In `src/lib/components/RecordControls.svelte`, update button labels to show mnemonic letters underlined when Alt is held. Track Alt key state:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import type { RecordingState } from "../types";

  interface Props {
    state: RecordingState;
    onstart: () => void;
    onpause: () => void;
    onresume: () => void;
    onstop: () => void;
  }

  let { state, onstart, onpause, onresume, onstop }: Props = $props();

  let altPressed = $state(false);

  onMount(() => {
    function onKeyDown(e: KeyboardEvent) { if (e.key === "Alt") altPressed = true; }
    function onKeyUp(e: KeyboardEvent) { if (e.key === "Alt") altPressed = false; }
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  });
</script>
```

Update button labels to underline the mnemonic letter:

```svelte
{#if state === "Idle"}
  <button type="button" onclick={onstart} aria-label="Start recording (Alt+S)" class="btn btn-start">
    {#if altPressed}<u>S</u>tart{:else}Start{/if}
  </button>
{:else if state === "Recording"}
  <button type="button" onclick={onpause} aria-label="Pause recording (Alt+P)" class="btn btn-pause">
    {#if altPressed}<u>P</u>ause{:else}Pause{/if}
  </button>
  <button type="button" onclick={onstop} aria-label="Stop recording (Alt+T)" class="btn btn-stop">
    S{#if altPressed}<u>t</u>op{:else}top{/if}
  </button>
{:else if state === "Paused"}
  <button type="button" onclick={onresume} aria-label="Resume recording (Alt+R)" class="btn btn-resume">
    {#if altPressed}<u>R</u>esume{:else}Resume{/if}
  </button>
  <button type="button" onclick={onstop} aria-label="Stop recording (Alt+T)" class="btn btn-stop">
    S{#if altPressed}<u>t</u>op{:else}top{/if}
  </button>
{/if}
```

- [ ] **Step 4: Verify compilation and visual**

Run: `pnpm exec svelte-check`
Run: `pnpm tauri dev` — hold Alt and verify underlines appear on buttons

- [ ] **Step 5: Commit**

```bash
git add src/App.svelte src/lib/components/RecordControls.svelte
git commit -m "feat(phase2): add Alt+letter mnemonics for recording controls"
```

---

## Task 9: Sync frontend state from hotkey-triggered backend changes

**Files:**
- Modify: `src-tauri/src/lib.rs` — Ensure state emitter emits final Idle event on stop
- Modify: `src/lib/stores/recording.svelte.ts` — Reset durationMs on Idle event

- [ ] **Step 1: Ensure state emitter sends Idle on stop**

Currently, the state emitter loop exits when `recording_state == Idle`. This means when the hotkey triggers a stop, the frontend may never receive the final `Idle` state.

In `stop_recording_inner`, after setting state to Idle, emit one final event:

```rust
// After resetting state in stop_recording_inner, emit final state
let _ = app.emit("recording-state-changed", serde_json::json!({
    "state": "Idle",
    "durationMs": 0,
}));
```

This requires `stop_recording_inner` to take `&AppHandle`. Adjust its signature.

- [ ] **Step 2: Handle Idle state in frontend listener**

In `src/lib/stores/recording.svelte.ts`, the listener already updates state from the event. Ensure `durationMs` resets:

```typescript
await listen<RecordingStateEvent>("recording-state-changed", (event) => {
  recordingState = event.payload.state;
  durationMs = event.payload.durationMs;
});
```

This already works — when the backend emits `{state: "Idle", durationMs: 0}`, both are updated. The `liveRegionText` effect in `App.svelte` will also fire, announcing "Recording stopped" to screen readers.

- [ ] **Step 3: Verify end-to-end flow**

1. Start recording via hotkey → frontend shows Recording state
2. Short press hotkey → frontend shows Paused
3. Long press hotkey → frontend shows Idle, duration resets

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src/lib/stores/recording.svelte.ts
git commit -m "feat(phase2): sync frontend state on hotkey-triggered stop"
```

---

## Task 10: Final integration and manual testing

- [ ] **Step 1: Build and run**

```bash
pnpm tauri dev
```

- [ ] **Step 2: Test hotkey FSM (Definition of Done checklist)**

| Test | Expected | Pass? |
|------|----------|-------|
| Press Pause/Break from any window while Idle | Recording starts | |
| Short press (< 500ms) while Recording | Pauses recording | |
| Long press (>= 500ms) while Recording | Stops recording | |
| Short press while Paused | Resumes recording | |
| Long press while Paused | Stops recording | |

- [ ] **Step 3: Test sound notifications**

| Test | Expected | Pass? |
|------|----------|-------|
| Start recording (UI or hotkey) | Start sound plays | |
| Pause recording | Pause sound plays | |
| Stop recording | Stop sound plays | |
| Three sounds are distinguishable | Yes | |
| Disable sounds in settings | No sounds play | |
| Re-enable sounds | Sounds play again | |

- [ ] **Step 4: Test settings persistence**

| Test | Expected | Pass? |
|------|----------|-------|
| Change hotkey → restart | Hotkey restored | |
| Toggle sound → restart | Sound setting restored | |
| Old settings.json without new fields | Defaults applied | |

- [ ] **Step 5: Test mnemonics**

| Test | Expected | Pass? |
|------|----------|-------|
| Alt+S while Idle | Starts recording | |
| Alt+P while Recording | Pauses recording | |
| Alt+R while Paused | Resumes recording | |
| Alt+T while Recording/Paused | Stops recording | |
| Hold Alt | Mnemonic letters underlined | |

- [ ] **Step 6: Test device validation**

| Test | Expected | Pass? |
|------|----------|-------|
| Start with no mic selected in Mic mode | Error shown, screen reader announces | |
| Start with no loopback in Loopback mode | Error shown | |
| Start with no devices in Mix mode | Error shown | |

- [ ] **Step 7: Fix any issues found during testing**

- [ ] **Step 8: Final commit**

```bash
git add -A
git commit -m "feat(phase2): complete global hotkeys, sound notifications, mnemonics, device validation"
```
