pub mod audio;
pub mod hotkey;
pub mod portable;
pub mod settings;
pub mod sounds;
pub mod state;

use audio::capture;
use audio::mixer::{self, MixerConfig};
use audio::types::{AudioCommand, AudioDevice, OutputMode, RecordingState};
use audio::writer::WavOutputWriter;
use state::SharedState;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

#[tauri::command]
fn load_settings() -> settings::Settings {
    settings::read_settings()
}

#[tauri::command]
fn save_settings(settings: settings::Settings) -> Result<(), String> {
    settings::write_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_audio_devices() -> Vec<AudioDevice> {
    match audio::devices::list_all_devices() {
        Ok(devices) => devices,
        Err(e) => {
            log::error!("Failed to enumerate audio devices: {}", e);
            vec![]
        }
    }
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
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if s.recording_state != RecordingState::Idle {
        return Err("Already recording".into());
    }

    // Validate devices for the selected mode (FR3.13)
    match mode {
        OutputMode::Microphone | OutputMode::Mix => {
            if mic_id.is_none() {
                return Err("DEVICE_NOT_FOUND: Microphone device required for this mode".into());
            }
        }
        _ => {}
    }
    match mode {
        OutputMode::Loopback | OutputMode::Mix => {
            if loopback_id.is_none() {
                return Err("DEVICE_NOT_FOUND: Loopback device required for this mode".into());
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

    // Play notification sound BEFORE starting capture so it isn't recorded.
    // The mixer's warmup_discard_ms handles any residual audio that the WASAPI
    // render pipeline delivers after rodio's sink finishes.
    play_sound_blocking(&s, sounds::SoundKind::Start);

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
    let writer: Box<dyn audio::writer::OutputWriter> =
        Box::new(WavOutputWriter::new(output_path, sample_rate, channels).map_err(|e| e.to_string())?);

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
        // Give the mixer time to flush any residual notification sound that
        // the WASAPI render pipeline may still deliver to the loopback capture
        // after rodio's sink reports completion.
        warmup_discard_ms: 300,
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

    // Clone the Arc for the state emitter thread
    let state_arc = Arc::clone(state);
    let emitter_flag = s.emitter_running.clone();
    let app_clone = app.clone();
    drop(s); // release lock before spawning

    if emitter_flag
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        std::thread::Builder::new()
            .name("state-emitter".into())
            .spawn(move || {
                state_event_loop(app_clone, state_arc, emitter_flag);
            })
            .map_err(|e| e.to_string())?;
    }

    log::info!("Recording started");
    Ok(())
}

#[tauri::command]
fn pause_recording(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    pause_recording_inner(&state)
}

fn pause_recording_inner(state: &SharedState) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if s.recording_state != RecordingState::Recording {
        return Err("Not recording".into());
    }
    if let Some(ref tx) = s.command_tx {
        tx.send(AudioCommand::Pause).map_err(|e| e.to_string())?;
    }
    s.recording_state = RecordingState::Paused;
    play_sound(&s, sounds::SoundKind::Pause);
    s.pause_start_time = Some(std::time::Instant::now());
    log::info!("Recording paused");
    Ok(())
}

#[tauri::command]
fn resume_recording(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    resume_recording_inner(&state)
}

fn resume_recording_inner(state: &SharedState) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if s.recording_state != RecordingState::Paused {
        return Err("Not paused".into());
    }
    // Play notification sound BEFORE resuming mixer so it isn't recorded.
    // The mixer's pending-resume drain detection handles any residual sound
    // data that the loopback capture delivers after rodio's sink finishes.
    play_sound_blocking(&s, sounds::SoundKind::Start);
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
fn stop_recording(state: tauri::State<'_, SharedState>, app: tauri::AppHandle) -> Result<(), String> {
    stop_recording_inner(&state)?;
    let _ = app.emit("recording-state-changed", serde_json::json!({
        "state": "Idle",
        "durationMs": 0,
    }));
    Ok(())
}

fn stop_recording_inner(state: &SharedState) -> Result<(), String> {
    // Extract handles and reset state under the lock
    let (mic_handle, loopback_handle, mixer_handle, command_tx) = {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        if s.recording_state == RecordingState::Idle {
            return Err("Not recording".into());
        }

        // Accumulate any in-progress pause duration
        if let Some(p) = s.pause_start_time.take() {
            s.paused_duration += p.elapsed();
        }

        // Reset state
        s.recording_state = RecordingState::Idle;
        play_sound(&s, sounds::SoundKind::Stop);
        s.recording_start_time = None;
        s.paused_duration = std::time::Duration::ZERO;
        s.pause_start_time = None;
        s.emitter_running.store(false, Ordering::SeqCst);

        // Extract handles
        (
            s.mic_capture.take(),
            s.loopback_capture.take(),
            s.mixer.take(),
            s.command_tx.take(),
        )
    }; // lock released here

    // Send stop command to mixer before joining (outside the lock)
    if let Some(ref tx) = command_tx {
        let _ = tx.send(AudioCommand::Stop);
    }

    // Stop threads outside the lock to avoid holding mutex during join
    if let Some(mut h) = mic_handle {
        h.stop();
    }
    if let Some(mut h) = loopback_handle {
        h.stop();
    }
    if let Some(mut h) = mixer_handle {
        h.stop();
    }

    log::info!("Recording stopped");
    Ok(())
}

// --- Public do_* functions for hotkey module ---

pub fn do_start_recording(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<SharedState>();
    // Read device params from settings.json (frontend persists selections there)
    let settings = settings::read_settings();
    let mic_id = settings.selected_mic;
    let loopback_id = settings.selected_loopback;
    let mode = settings.output_mode;
    let sample_rate = settings.sample_rate;
    start_recording_inner(app, &state, mic_id, loopback_id, mode, sample_rate)
}

pub fn do_pause_recording(app: &tauri::AppHandle) -> Result<(), String> {
    pause_recording_inner(&app.state::<SharedState>())
}

pub fn do_resume_recording(app: &tauri::AppHandle) -> Result<(), String> {
    resume_recording_inner(&app.state::<SharedState>())
}

pub fn do_stop_recording(app: &tauri::AppHandle) -> Result<(), String> {
    stop_recording_inner(&app.state::<SharedState>())?;
    let _ = app.emit("recording-state-changed", serde_json::json!({
        "state": "Idle",
        "durationMs": 0,
    }));
    Ok(())
}

#[tauri::command]
fn set_mic_volume(state: tauri::State<'_, SharedState>, volume: f32) {
    let volume = volume.clamp(0.0, 4.0);
    if let Ok(mut s) = state.lock() {
        s.mic_volume = volume;
        if let Some(ref tx) = s.command_tx {
            let _ = tx.send(AudioCommand::SetMicVolume(volume));
        }
    }
}

#[tauri::command]
fn set_loopback_volume(state: tauri::State<'_, SharedState>, volume: f32) {
    let volume = volume.clamp(0.0, 4.0);
    if let Ok(mut s) = state.lock() {
        s.loopback_volume = volume;
        if let Some(ref tx) = s.command_tx {
            let _ = tx.send(AudioCommand::SetLoopbackVolume(volume));
        }
    }
}

#[tauri::command]
fn set_sound_enabled(state: tauri::State<'_, SharedState>, enabled: bool) {
    if let Ok(mut s) = state.lock() {
        s.sounds_enabled = enabled;
    }
}

#[tauri::command]
fn set_hotkey(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    hotkey::register(&app, &shortcut)
}

pub fn play_sound(state: &state::AppState, kind: sounds::SoundKind) {
    if state.sounds_enabled {
        if let Some(ref engine) = state.sound_engine {
            engine.play(kind);
        }
    }
}

fn play_sound_blocking(state: &state::AppState, kind: sounds::SoundKind) {
    if state.sounds_enabled {
        if let Some(ref engine) = state.sound_engine {
            engine.play_blocking(kind);
        }
    }
}

/// Background thread that emits recording state events at ~4 Hz.
fn state_event_loop(app: tauri::AppHandle, state: SharedState, running: Arc<AtomicBool>) {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(250));

        if !running.load(Ordering::SeqCst) {
            break;
        }

        let (payload, should_stop) = {
            let s = match state.lock() {
                Ok(s) => s,
                Err(_) => break,
            };

            if s.recording_state == RecordingState::Idle {
                break;
            }

            let payload = serde_json::json!({
                "state": format!("{:?}", s.recording_state),
                "durationMs": s.duration_ms(),
            });
            let should_stop = false;
            (payload, should_stop)
        }; // lock released before emit

        if should_stop {
            break;
        }

        let _ = app.emit("recording-state-changed", payload);
    }
    running.store(false, Ordering::SeqCst);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
        .manage(std::sync::Arc::new(std::sync::Mutex::new(
            state::AppState::default(),
        )))
        .setup(|app| {
            portable::ensure_dirs().map_err(|e| e.to_string())?;

            let settings = settings::read_settings();
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
            set_sound_enabled,
            set_hotkey,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                if let Some(state) = app.try_state::<SharedState>() {
                    let is_recording = state
                        .lock()
                        .map(|s| s.recording_state != RecordingState::Idle)
                        .unwrap_or(false);
                    if is_recording {
                        log::info!("Window closing during recording — stopping recording");
                        let _ = stop_recording_inner(&state);
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
