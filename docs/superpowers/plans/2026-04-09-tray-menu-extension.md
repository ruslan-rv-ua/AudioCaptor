# Tray Menu Extension Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the system tray context menu from 2 items (About + Quit) to 6 items with dynamic recording controls.

**Architecture:** `TrayMenuRefs { toggle_item, stop_item }` is stored in Tauri managed state as `Mutex<TrayMenuRefs>`. A public `update_tray_recording_state(app, state)` function updates menu item text and enabled-state after every recording state change. Only two Rust files change: `tray.rs` (full rewrite) and `lib.rs` (four one-line additions). No frontend changes.

**Tech Stack:** Rust, Tauri v2 (`tauri::menu::{Menu, MenuItem, PredefinedMenuItem}`, `tauri::tray::TrayIconBuilder`), `std::sync::Mutex`, Windows `explorer.exe`.

**Spec:** `docs/superpowers/specs/2026-04-09-tray-menu-extension-design.md`

---

## File Map

| File | Action | What changes |
|---|---|---|
| `src-tauri/src/tray.rs` | Full rewrite | Add `TrayMenuRefs` struct, 5 new menu items + 3 separators, `update_tray_recording_state()`, refactored `handle_menu_event()` |
| `src-tauri/src/lib.rs` | Modify | 4 lines added — one `tray::update_tray_recording_state()` call per `do_*` function |

> **Note on testing:** Tauri tray code requires a running app and cannot be unit-tested in isolation. Verification is `cargo build` (catches type errors, Send+Sync violations) + the manual checklist at the end of the plan.

---

## Task 1: Rewrite `tray.rs`

**Files:**
- Modify: `src-tauri/src/tray.rs` (full replacement)

### Background for the implementer

The current file is 68 lines with one function `setup_tray()` that creates two items (About, Quit) and handles them inline. We will:

1. Add a `TrayMenuRefs` struct holding refs to the two dynamically-updated items.
2. Expand `setup_tray()` to build all 6 items + 3 separators and register `TrayMenuRefs` via `app.manage()`.
3. Extract `handle_menu_event()` as a separate function.
4. Add `update_tray_recording_state()` as a public function called from `lib.rs`.

**Key Tauri v2 APIs used:**
- `MenuItem::with_id(app, id, label, enabled, accelerator)` — creates a menu item
- `PredefinedMenuItem::separator(app)` — horizontal separator
- `Menu::with_items(app, &[...])` — builds menu from slice of `IsMenuItem` trait objects
- `item.set_text(text)` → `tauri::Result<()>` — update label
- `item.set_enabled(bool)` → `tauri::Result<()>` — enable/disable
- `app.manage(value)` — store value in Tauri state (requires `Send + Sync + 'static`)
- `app.try_state::<T>()` — retrieve managed state, returns `Option<State<T>>`

- [ ] **Step 1: Replace `src-tauri/src/tray.rs` with the new implementation**

Write the entire file as shown below. Do not preserve any of the old content.

```rust
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
            if let Ok(base) = crate::portable::exe_dir() {
                let dir = base.join(&profile.output_folder);
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::process::Command::new("explorer.exe")
                    .arg(&dir)
                    .spawn();
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

/// Update tray recording-control labels and enabled states after any
/// `RecordingState` change. Call this from `lib.rs` `do_*` functions.
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
```

- [ ] **Step 2: Verify the build compiles**

```bash
cd src-tauri && cargo build 2>&1
```

Expected: no errors. If you see a `Send`/`Sync` error on `TrayMenuRefs`, double-check that `app.manage(Mutex::new(...))` is used — not `app.manage(TrayMenuRefs { ... })` directly.

Common errors and fixes:
- `MenuItem` field not recognized → check `use tauri::menu::MenuItem;` is in scope
- `cannot move out of toggle_item` after `Menu::with_items` → `Menu::with_items` takes references (`&item`), so `toggle_item` is still owned after the call and can be moved into `TrayMenuRefs`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat(tray): add Show/Hide, recording controls, Open Recordings Folder"
```

---

## Task 2: Wire `update_tray_recording_state` into `lib.rs`

**Files:**
- Modify: `src-tauri/src/lib.rs` lines 370–408

### Background for the implementer

Four public `do_*` functions in `lib.rs` dispatch recording actions. After each succeeds, we call `tray::update_tray_recording_state(app, <new_state>)` so the tray menu reflects the current state.

Current function bodies (simplified):

```rust
pub fn do_start_recording(app: &AppHandle) -> Result<(), String> {
    // ... builds params from settings ...
    start_recording_inner(app, &state, mic_id, loopback_id, mode, rate)  // returns Result
}

pub fn do_pause_recording(app: &AppHandle) -> Result<(), String> {
    pause_recording_inner(&app.state::<SharedState>())
}

pub fn do_resume_recording(app: &AppHandle) -> Result<(), String> {
    resume_recording_inner(&app.state::<SharedState>())
}

pub fn do_stop_recording(app: &AppHandle) -> Result<(), String> {
    stop_recording_inner(&app.state::<SharedState>())?;
    let _ = app.emit("recording-state-changed", ...);
    Ok(())
}
```

The pattern: change the last expression from a bare `Result`-returning call to `call()?; tray::update...; Ok(())`.

- [ ] **Step 4: Update `do_start_recording` (lib.rs ~line 384)**

Find the end of the function (include the closing brace):
```rust
    start_recording_inner(
        app,
        &state,
        mic_id,
        loopback_id,
        profile.output_mode,
        profile.sample_rate,
    )
}
```

Replace with:
```rust
    start_recording_inner(
        app,
        &state,
        mic_id,
        loopback_id,
        profile.output_mode,
        profile.sample_rate,
    )?;
    tray::update_tray_recording_state(app, RecordingState::Recording);
    Ok(())
}
```

- [ ] **Step 5: Update `do_pause_recording` (lib.rs ~line 395)**

Find:
```rust
pub fn do_pause_recording(app: &tauri::AppHandle) -> Result<(), String> {
    pause_recording_inner(&app.state::<SharedState>())
}
```

Replace with:
```rust
pub fn do_pause_recording(app: &tauri::AppHandle) -> Result<(), String> {
    pause_recording_inner(&app.state::<SharedState>())?;
    tray::update_tray_recording_state(app, RecordingState::Paused);
    Ok(())
}
```

- [ ] **Step 6: Update `do_resume_recording` (lib.rs ~line 399)**

Find:
```rust
pub fn do_resume_recording(app: &tauri::AppHandle) -> Result<(), String> {
    resume_recording_inner(&app.state::<SharedState>())
}
```

Replace with:
```rust
pub fn do_resume_recording(app: &tauri::AppHandle) -> Result<(), String> {
    resume_recording_inner(&app.state::<SharedState>())?;
    tray::update_tray_recording_state(app, RecordingState::Recording);
    Ok(())
}
```

- [ ] **Step 7: Update `do_stop_recording` (lib.rs ~line 403)**

Find the closing `Ok(())` of `do_stop_recording`:
```rust
pub fn do_stop_recording(app: &tauri::AppHandle) -> Result<(), String> {
    stop_recording_inner(&app.state::<SharedState>())?;
    let _ = app.emit("recording-state-changed", serde_json::json!({
        "state": "Idle",
        "durationMs": 0,
    }));
    Ok(())
}
```

Replace with:
```rust
pub fn do_stop_recording(app: &tauri::AppHandle) -> Result<(), String> {
    stop_recording_inner(&app.state::<SharedState>())?;
    let _ = app.emit("recording-state-changed", serde_json::json!({
        "state": "Idle",
        "durationMs": 0,
    }));
    tray::update_tray_recording_state(app, RecordingState::Idle);
    Ok(())
}
```

- [ ] **Step 8: Final build**

```bash
cd src-tauri && cargo build 2>&1
```

Expected: clean build, zero errors, zero new warnings.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(tray): wire update_tray_recording_state into do_* functions"
```

---

## Manual Verification Checklist

Run the app (`cargo tauri dev` or the built binary) and check each item:

- [ ] Right-click tray icon → menu shows 6 items + 3 separators in correct order
- [ ] Menu labels are correct in English (default) and Ukrainian (change language in settings and restart)
- [ ] **Show / Hide Window** — click once hides window; click again shows and focuses it
- [ ] **Start** — click starts recording; toggle label changes to "Pause"; Stop becomes enabled
- [ ] **Pause** — click pauses recording; toggle label changes to "Resume"
- [ ] **Resume** — click resumes recording; toggle label changes back to "Pause"
- [ ] **Stop** — click stops recording; toggle label returns to "Start"; Stop becomes disabled (grayed out)
- [ ] **Stop when Idle** — click Stop while not recording: nothing visible happens, log shows `"Tray stop-recording failed: Not recording"` (check `logs/audiocaptor.log`)
- [ ] **Open Recordings Folder** — Explorer opens to active profile's output folder
- [ ] **About** — main window shows and About dialog opens
- [ ] **Quit (idle)** — app exits immediately
- [ ] **Quit (recording)** — main window shows with confirm-exit dialog
- [ ] Left-click on tray icon still shows and focuses window (unchanged)
