use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

use crate::{audio::types::RecordingState, state::SharedState};

pub fn setup_tray(app: &tauri::AppHandle, language: &str) -> tauri::Result<()> {
    let quit_label = if language == "uk" { "Вийти" } else { "Quit" };
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_item])?;

    let Some(icon) = app.default_window_icon() else {
        log::warn!("No default window icon configured; tray icon not created");
        return Ok(());
    };

    TrayIconBuilder::new()
        .icon(icon.clone())
        .menu(&menu)
        .tooltip("AudioCaptor")
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                let state = app.state::<SharedState>();
                let is_recording = state
                    .lock()
                    .map(|s| s.recording_state != RecordingState::Idle)
                    .unwrap_or(false);
                if is_recording {
                    let _ = app.emit("tray-quit-requested", ());
                } else {
                    app.exit(0);
                }
            }
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

    Ok(())
}
