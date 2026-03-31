use crate::audio::types::RecordingState;
use crate::state::SharedState;
use crate::{do_pause_recording, do_resume_recording, do_start_recording, do_stop_recording};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

/// Threshold in ms: press < LONG_PRESS_MS = short, >= LONG_PRESS_MS = long.
const LONG_PRESS_MS: u128 = 500;

/// Stores the timestamp of the most recent key press.
static PRESS_TIME: Mutex<Option<Instant>> = Mutex::new(None);

/// Register the global hotkey. Safe to call multiple times (re-registration).
pub fn register(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
    let manager = app.global_shortcut();

    // Unregister all first (safe no-op if nothing registered)
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
                    if let Err(e) = do_start_recording(app) {
                        log::error!("Hotkey start failed: {e}");
                        let _ = app.emit("recording-error", e);
                    }
                }
                RecordingState::Recording => {
                    if duration_ms < LONG_PRESS_MS {
                        if let Err(e) = do_pause_recording(app) {
                            log::error!("Hotkey pause failed: {e}");
                        }
                    } else if let Err(e) = do_stop_recording(app) {
                        log::error!("Hotkey stop failed: {e}");
                    }
                }
                RecordingState::Paused => {
                    if duration_ms < LONG_PRESS_MS {
                        if let Err(e) = do_resume_recording(app) {
                            log::error!("Hotkey resume failed: {e}");
                        }
                    } else if let Err(e) = do_stop_recording(app) {
                        log::error!("Hotkey stop failed: {e}");
                    }
                }
            }
        }
    }
}
