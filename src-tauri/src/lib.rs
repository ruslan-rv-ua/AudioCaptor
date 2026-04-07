pub mod audio;
pub mod device_monitor;
pub mod hotkey;
pub mod portable;
pub mod profiles;
pub mod settings;
pub mod sounds;
pub mod state;
pub mod tray;

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
fn refresh_devices() -> Vec<AudioDevice> {
    match audio::devices::list_all_devices() {
        Ok(devices) => devices,
        Err(e) => {
            log::error!("Failed to refresh audio devices: {}", e);
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
        OutputMode::Microphone | OutputMode::Mix
        | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback => {
            if mic_id.is_none() {
                return Err("DEVICE_NOT_FOUND: Microphone device required for this mode".into());
            }
        }
        _ => {}
    }
    match mode {
        OutputMode::Loopback | OutputMode::Mix
        | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback => {
            if loopback_id.is_none() {
                return Err("DEVICE_NOT_FOUND: Loopback device required for this mode".into());
            }
        }
        _ => {}
    }

    // Get active profile for output path and filenames
    let settings = settings::read_settings();
    let profile = settings.profiles.iter()
        .find(|p| p.id == settings.active_profile_id)
        .cloned()
        .unwrap_or_default();

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let base_dir = portable::exe_dir().map_err(|e| e.to_string())?;
    let output_dir = base_dir.join(&profile.output_folder);
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    // Play notification sound BEFORE starting capture so it isn't recorded.
    // The mixer's warmup_discard_ms handles any residual audio that the WASAPI
    // render pipeline delivers after rodio's sink finishes.
    play_sound_blocking(&s, sounds::SoundKind::Start);

    // Start capture threads as needed
    let (mic_handle, mic_consumer) = if let Some(ref id) = mic_id {
        match mode {
            OutputMode::Microphone | OutputMode::Mix
            | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback => {
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
            OutputMode::Loopback | OutputMode::Mix
            | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback => {
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

    // Create WAV writer(s) based on mode
    let (writer, secondary_writer): (Box<dyn audio::writer::OutputWriter>, Option<Box<dyn audio::writer::OutputWriter>>) = match mode {
        OutputMode::MixPlusMicrophone => {
            let mix_path = output_dir.join(format!("{}_{}.wav", profile.mix_filename, timestamp));
            let mic_path = output_dir.join(format!("{}_{}.wav", profile.mic_filename, timestamp));
            let w1 = Box::new(WavOutputWriter::new(mix_path, sample_rate, channels).map_err(|e| e.to_string())?);
            let w2 = Box::new(WavOutputWriter::new(mic_path, sample_rate, channels).map_err(|e| e.to_string())?);
            (w1, Some(w2))
        }
        OutputMode::MixPlusLoopback => {
            let mix_path = output_dir.join(format!("{}_{}.wav", profile.mix_filename, timestamp));
            let loop_path = output_dir.join(format!("{}_{}.wav", profile.loopback_filename, timestamp));
            let w1 = Box::new(WavOutputWriter::new(mix_path, sample_rate, channels).map_err(|e| e.to_string())?);
            let w2 = Box::new(WavOutputWriter::new(loop_path, sample_rate, channels).map_err(|e| e.to_string())?);
            (w1, Some(w2))
        }
        OutputMode::Microphone => {
            let path = output_dir.join(format!("{}_{}.wav", profile.mic_filename, timestamp));
            (Box::new(WavOutputWriter::new(path, sample_rate, channels).map_err(|e| e.to_string())?), None)
        }
        OutputMode::Loopback => {
            let path = output_dir.join(format!("{}_{}.wav", profile.loopback_filename, timestamp));
            (Box::new(WavOutputWriter::new(path, sample_rate, channels).map_err(|e| e.to_string())?), None)
        }
        OutputMode::Mix => {
            let path = output_dir.join(format!("{}_{}.wav", profile.mix_filename, timestamp));
            (Box::new(WavOutputWriter::new(path, sample_rate, channels).map_err(|e| e.to_string())?), None)
        }
    };

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
        secondary_writer,
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
    s.recording_mic_id = mic_id.clone();
    s.recording_loopback_id = loopback_id.clone();
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
        s.recording_mic_id = None;
        s.recording_loopback_id = None;

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
    let settings = settings::read_settings();
    let mic_id = settings.selected_mic;
    let loopback_id = settings.selected_loopback;

    // Get params from active profile
    let profile = settings
        .profiles
        .iter()
        .find(|p| p.id == settings.active_profile_id)
        .cloned()
        .unwrap_or_default();

    start_recording_inner(
        app,
        &state,
        mic_id,
        loopback_id,
        profile.output_mode,
        profile.sample_rate,
    )
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
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn set_hotkey(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    hotkey::register(&app, &shortcut)
}

#[tauri::command]
fn unregister_hotkey(app: tauri::AppHandle) -> Result<(), String> {
    hotkey::unregister(&app)
}

#[tauri::command]
fn cmd_list_profiles() -> Vec<profiles::RecordingProfile> {
    let settings = settings::read_settings();
    settings.profiles
}

#[tauri::command]
fn cmd_save_profile(profile: profiles::RecordingProfile) -> Result<(), String> {
    let mut settings = settings::read_settings();
    profiles::save_profile(&mut settings.profiles, profile)?;
    settings::write_settings(&settings).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn cmd_delete_profile(id: String) -> Result<String, String> {
    let mut settings = settings::read_settings();
    profiles::delete_profile(&mut settings.profiles, &id)?;
    let new_active = if settings.active_profile_id == id {
        settings.profiles[0].id.clone()
    } else {
        settings.active_profile_id.clone()
    };
    settings.active_profile_id = new_active.clone();
    settings::write_settings(&settings).map_err(|e| e.to_string())?;
    Ok(new_active)
}

#[tauri::command]
fn cmd_select_profile(state: tauri::State<'_, SharedState>, id: String) -> Result<(), String> {
    let mut settings = settings::read_settings();
    if !settings.profiles.iter().any(|p| p.id == id) {
        return Err(format!("Profile not found: {id}"));
    }
    settings.active_profile_id = id.clone();
    settings::write_settings(&settings).map_err(|e| e.to_string())?;

    if let Some(profile) = settings.profiles.iter().find(|p| p.id == id) {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.mic_volume = profile.mic_volume;
        s.loopback_volume = profile.loopback_volume;
        s.active_profile_id = id;
    }
    Ok(())
}

#[tauri::command]
fn cmd_get_active_profile() -> Result<profiles::RecordingProfile, String> {
    let settings = settings::read_settings();
    settings.profiles.iter()
        .find(|p| p.id == settings.active_profile_id)
        .cloned()
        .ok_or_else(|| "Active profile not found".into())
}

#[tauri::command]
fn get_recordings_dir() -> String {
    portable::exe_dir()
        .map(|p| p.join("Recordings").to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Recordings".to_string())
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
        .plugin(tauri_plugin_dialog::init())
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
            let _ = settings::write_settings(&settings); // persist migration if version was bumped
            if let Ok(mut s) = app.state::<SharedState>().lock() {
                match sounds::SoundEngine::new() {
                    Ok(engine) => s.sound_engine = Some(engine),
                    Err(e) => log::warn!("Sound engine init failed: {e}"),
                }
                s.sounds_enabled = settings.sound_enabled;
                if let Some(profile) = settings.profiles.iter().find(|p| p.id == settings.active_profile_id) {
                    s.mic_volume = profile.mic_volume;
                    s.loopback_volume = profile.loopback_volume;
                }
                s.active_profile_id = settings.active_profile_id.clone();
            }

            // Register global hotkey (only if one is configured)
            if let Some(ref hk) = settings.hotkey {
                if let Err(e) = hotkey::register(&app.handle(), hk) {
                    log::error!("Failed to register hotkey: {e}");
                }
            }

            // Start device monitor
            match device_monitor::start_device_monitor(app.handle().clone()) {
                Ok(handle) => {
                    if let Ok(mut s) = app.state::<SharedState>().lock() {
                        s.device_monitor = Some(handle);
                    }
                }
                Err(e) => log::error!("Failed to start device monitor: {e}"),
            }

            // Set up system tray icon
            if let Err(e) = tray::setup_tray(app.handle(), &settings.language) {
                log::error!("Failed to set up tray icon: {e}");
            }

            log::info!("AudioCaptor started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            get_audio_devices,
            refresh_devices,
            start_recording,
            pause_recording,
            resume_recording,
            stop_recording,
            set_mic_volume,
            set_loopback_volume,
            set_sound_enabled,
            set_hotkey,
            unregister_hotkey,
            cmd_list_profiles,
            cmd_save_profile,
            cmd_delete_profile,
            cmd_select_profile,
            cmd_get_active_profile,
            get_recordings_dir,
            quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
