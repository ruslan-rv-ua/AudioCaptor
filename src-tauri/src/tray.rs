use std::sync::Mutex;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::{audio::types::RecordingState, state::SharedState};

/// References to tray menu items that need dynamic text/enabled updates.
///
/// Wrapped in `Mutex` because `MenuItem<Wry>` may not be `Sync` on its own
/// (depends on the underlying muda platform handle). `app.manage()` requires
/// `Send + Sync + 'static`, so the Mutex wrapper is mandatory.
pub struct TrayMenuRefs {
    pub toggle_item: MenuItem<tauri::Wry>,
    pub stop_item: MenuItem<tauri::Wry>,
}

pub fn setup_tray(app: &AppHandle, language: &str) -> tauri::Result<()> {
    let uk = language == "uk";

    // --- Static items ---
    let show_hide = MenuItem::with_id(
        app,
        "show-hide",
        if uk { "Показати / Сховати вікно" } else { "Show / Hide Window" },
        true,
        None::<&str>,
    )?;

    // --- Recording controls (dynamic text + enabled) ---
    let toggle_item = MenuItem::with_id(
        app,
        "toggle-recording",
        if uk { "Старт" } else { "Start" },
        true,
        None::<&str>,
    )?;
    let stop_item = MenuItem::with_id(
        app,
        "stop-recording",
        if uk { "Зупинити" } else { "Stop" },
        false, // disabled when Idle
        None::<&str>,
    )?;

    // --- Utility items ---
    let open_folder = MenuItem::with_id(
        app,
        "open-recordings",
        if uk { "Відкрити папку записів" } else { "Open Recordings Folder" },
        true,
        None::<&str>,
    )?;
    let about_label = if uk { "Про програму" } else { "About" };
    let about_item = MenuItem::with_id(app, "about", about_label, true, None::<&str>)?;
    let quit_label = if uk { "Вийти" } else { "Quit" };
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;

    // --- Separators ---
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_hide,
            &sep1,
            &toggle_item,
            &stop_item,
            &sep2,
            &open_folder,
            &about_item,
            &sep3,
            &quit_item,
        ],
    )?;

    let Some(icon) = app.default_window_icon() else {
        log::warn!("No default window icon configured; tray icon not created");
        return Ok(());
    };

    TrayIconBuilder::new()
        .icon(icon.clone())
        .menu(&menu)
        .tooltip("AudioCaptor")
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    // Store refs for dynamic updates. Must be called after build() so the
    // tray icon exists. app.manage() panics if called twice for the same type,
    // but setup_tray() is only called once during app setup.
    app.manage(Mutex::new(TrayMenuRefs { toggle_item, stop_item }));

    Ok(())
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "show-hide" => {
            if let Some(w) = app.get_webview_window("main") {
                if w.is_visible().unwrap_or(false) {
                    let _ = w.hide();
                } else {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        }
        "toggle-recording" => {
            let state = app.state::<SharedState>();
            let rs = state
                .lock()
                .map(|s| s.recording_state.clone())
                .unwrap_or(RecordingState::Idle);
            let result = match rs {
                RecordingState::Idle => crate::do_start_recording(app),
                RecordingState::Recording => crate::do_pause_recording(app),
                RecordingState::Paused => crate::do_resume_recording(app),
            };
            if let Err(e) = result {
                log::error!("Tray toggle-recording failed: {e}");
                if let Ok(s) = app.state::<SharedState>().lock() {
                    crate::play_sound(&s, crate::sounds::SoundKind::Warning);
                }
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        }
        "stop-recording" => {
            if let Err(e) = crate::do_stop_recording(app) {
                log::error!("Tray stop-recording failed: {e}");
            }
        }
        "open-recordings" => {
            let settings = crate::settings::read_settings();
            let profile = settings
                .profiles
                .iter()
                .find(|p| p.id == settings.active_profile_id)
                .cloned()
                .unwrap_or_default();
            match crate::portable::exe_dir() {
                Ok(base) => {
                    let dir = base.join(&profile.output_folder);
                    let _ = std::fs::create_dir_all(&dir);
                    if let Err(e) = std::process::Command::new("explorer.exe")
                        .arg(&dir)
                        .spawn()
                    {
                        log::error!("Failed to open recordings folder in Explorer: {e}");
                    }
                }
                Err(e) => log::error!("open-recordings: failed to resolve exe dir: {e}"),
            }
        }
        "about" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            let _ = app.emit("tray-about-requested", ());
        }
        "quit" => {
            let state = app.state::<SharedState>();
            let is_recording = state
                .lock()
                .map(|s| s.recording_state != RecordingState::Idle)
                .unwrap_or(false);
            if is_recording {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
                let _ = app.emit("tray-quit-requested", ());
            } else {
                app.exit(0);
            }
        }
        _ => {}
    }
}

/// Update tray recording-control toggle label and both items' enabled states
/// after any `RecordingState` change. Call this from `lib.rs` recording functions.
///
/// Reads language from `settings.json` on each call so it stays in sync
/// with runtime language changes (acceptable partial-update behaviour).
///
/// Silently does nothing if `TrayMenuRefs` is not yet registered or if
/// the mutex is poisoned.
pub fn update_tray_recording_state(app: &AppHandle, state: RecordingState) {
    let Some(refs) = app.try_state::<Mutex<TrayMenuRefs>>() else {
        return;
    };
    let Ok(refs) = refs.lock() else {
        return;
    };
    let uk = crate::settings::read_settings().language == "uk";
    match state {
        RecordingState::Idle => {
            let _ = refs.toggle_item.set_text(if uk { "Старт" } else { "Start" });
            let _ = refs.toggle_item.set_enabled(true);
            let _ = refs.stop_item.set_enabled(false);
        }
        RecordingState::Recording => {
            let _ = refs.toggle_item.set_text(if uk { "Пауза" } else { "Pause" });
            let _ = refs.toggle_item.set_enabled(true);
            let _ = refs.stop_item.set_enabled(true);
        }
        RecordingState::Paused => {
            let _ = refs.toggle_item.set_text(if uk { "Продовжити" } else { "Resume" });
            let _ = refs.toggle_item.set_enabled(true);
            let _ = refs.stop_item.set_enabled(true);
        }
    }
}
