use crate::audio::capture::CaptureHandle;
use crate::audio::mixer::MixerHandle;
use crate::audio::types::{AudioCommand, OutputMode, RecordingState};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

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
    pub sound_engine: Option<crate::sounds::SoundEngine>,
    pub sounds_enabled: bool,
    pub active_profile_id: String,
    pub recording_mic_id: Option<String>,
    pub recording_loopback_id: Option<String>,
    // Active recording handles (None when idle)
    pub mic_capture: Option<CaptureHandle>,
    pub loopback_capture: Option<CaptureHandle>,
    pub mixer: Option<MixerHandle>,
    pub command_tx: Option<Sender<AudioCommand>>,
    pub emitter_running: Arc<AtomicBool>,
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
            sound_engine: None,
            sounds_enabled: true,
            active_profile_id: "default".to_string(),
            recording_mic_id: None,
            recording_loopback_id: None,
            mic_capture: None,
            loopback_capture: None,
            mixer: None,
            command_tx: None,
            emitter_running: Arc::new(AtomicBool::new(false)),
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

pub type SharedState = Arc<Mutex<AppState>>;
