# Tray Menu Extension — Design Spec

**Date:** 2026-04-09  
**Branch:** feature/tray-menu  
**Stack:** Rust, Tauri v2 Menu API, i18n (en/uk)  
**Platform:** Windows desktop, portable app

---

## Summary

Extend the system tray context menu from its current 2-item state (About + Quit) to 6 items: Show/Hide Window, recording controls (Toggle + Stop), Open Recordings Folder, About, Quit — with dynamic `enabled`/text updates for recording controls.

---

## Current State

[src-tauri/src/tray.rs](../../src-tauri/src/tray.rs) — 68 lines, one `setup_tray()` function.

**Current menu:**
```
About          ← label is "About" / "Про програму" (not "About AudioCaptor")
Quit
```

**Existing behaviour:**
- Left-click on icon → show + focus main window
- About → emit `tray-about-requested` (frontend listens in App.svelte)
- Quit → if recording: show window + emit `tray-quit-requested`; else `app.exit(0)`

---

## Final Menu Structure

```
┌──────────────────────────────┐
│ Show / Hide Window           │  fixed label, toggles visibility
├──────────────────────────────┤
│ ▶ Start                      │  toggle: Start / Pause / Resume
│   Stop               (gray)  │  disabled when Idle
├──────────────────────────────┤
│ Open Recordings Folder       │  active profile's output folder
│ About                        │  unchanged (label stays "About"/"Про програму")
├──────────────────────────────┤
│ Quit                         │  unchanged
└──────────────────────────────┘
```

Menu item IDs: `show-hide`, `toggle-recording`, `stop-recording`, `open-recordings`, `about`, `quit`.  
Separators via `PredefinedMenuItem::separator()`.

---

## Architecture

### Approach

Store references to dynamically-updated `MenuItem`s in Tauri managed state (`app.manage()`), consistent with the existing `SharedState` pattern.

### `TrayMenuRefs`

`MenuItem<tauri::Wry>` may not implement `Sync` (it depends on the underlying `muda` platform implementation). To guarantee `app.manage()` compiles (which requires `Send + Sync + 'static`), wrap `TrayMenuRefs` in a `std::sync::Mutex`:

```rust
use std::sync::Mutex;

/// References to tray menu items that need dynamic updates.
/// Wrapped in Mutex because MenuItem<Wry> may not be Sync on its own.
/// Registered once via app.manage() in setup_tray().
pub struct TrayMenuRefs {
    pub toggle_item: MenuItem<tauri::Wry>,
    pub stop_item: MenuItem<tauri::Wry>,
}
```

Registered as: `app.manage(Mutex::new(TrayMenuRefs { toggle_item, stop_item }));`

Accessed via: `app.try_state::<Mutex<TrayMenuRefs>>()`

### Files Changed

| File | Change |
|---|---|
| `src-tauri/src/tray.rs` | Full rewrite — new menu items, `TrayMenuRefs`, `update_tray_recording_state()`, `handle_menu_event()` |
| `src-tauri/src/lib.rs` | Add one `tray::update_tray_recording_state(app, ...)` call in each of the four `do_*` functions |

Frontend (`src/App.svelte`) — **no changes**. Already listens for `tray-about-requested`.

---

## Data Flow

### Recording state updates

After any `RecordingState` change in `lib.rs`, the relevant `do_*` function calls:

```rust
tray::update_tray_recording_state(app, RecordingState::Recording); // or Paused / Idle
```

`update_tray_recording_state` reads language internally via `settings::read_settings().language`, then locks `Mutex<TrayMenuRefs>` and calls `set_text()`/`set_enabled()` — both return `tauri::Result<()>`, errors are silently ignored with `let _ =`.

### Toggle item state machine

| RecordingState | `toggle_item` text (en/uk) | `toggle_item` enabled | `stop_item` enabled |
|---|---|---|---|
| Idle | Start / Старт | true | **false** |
| Recording | Pause / Пауза | true | true |
| Paused | Resume / Продовжити | true | true |

### Language behaviour

- **Static items** (Show/Hide, Open Recordings Folder, About, Quit): labels are set once at startup and do **not** update when user changes language in settings — requires app restart.
- **Dynamic items** (Toggle, Stop): text is set on every `update_tray_recording_state` call via fresh `settings::read_settings().language`. This means their labels **will** reflect a language change on the next recording state transition. This is an acceptable partial-update; no action needed.

---

## Event Handlers

### `show-hide`
```rust
if let Some(w) = app.get_webview_window("main") {
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
    } else {
        let _ = w.show();
        let _ = w.set_focus();
    }
}
```

### `toggle-recording`
Read `SharedState.recording_state`, dispatch:
- Idle → `do_start_recording(app)`
- Recording → `do_pause_recording(app)`
- Paused → `do_resume_recording(app)`

Log errors via `log::error!`; no UI notification for MVP (consistent with hotkey behaviour).

### `stop-recording`
Call `do_stop_recording(app)`. Log errors.

`do_stop_recording` calls `stop_recording_inner` which returns `Err("Not recording")` when `RecordingState == Idle`. This error is safely logged and ignored — no crash, no state corruption.

### `open-recordings`
```rust
let settings = settings::read_settings();
let profile = settings.profiles.iter()
    .find(|p| p.id == settings.active_profile_id)
    .cloned()
    .unwrap_or_default();
if let Ok(base) = portable::exe_dir() {
    let dir = base.join(&profile.output_folder);
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::process::Command::new("explorer.exe").arg(&dir).spawn();
}
```
Opens the **active profile's** output folder (defaults to `Recordings/`, but respects per-profile customisation). All errors are silently ignored (best-effort). Windows-only; no cross-platform shim needed.

### `about` and `quit`
Unchanged from current implementation.

---

## i18n

Hard-coded `if language == "uk"` checks on the Rust side (same pattern as existing `tray.rs`). No Paraglide — tray labels are Rust-only.

New strings:

| Key (conceptual) | en | uk |
|---|---|---|
| show_hide | Show / Hide Window | Показати / Сховати вікно |
| start | Start | Старт |
| pause | Pause | Пауза |
| resume | Resume | Продовжити |
| stop | Stop | Зупинити |
| open_recordings | Open Recordings Folder | Відкрити папку записів |

Existing strings `About`/`Про програму` and `Quit`/`Вийти` remain unchanged.

---

## Constraints and Non-Goals

- **No tooltip status** — tooltip remains `"AudioCaptor"` permanently.
- **No dynamic Show/Hide label** — fixed `"Show / Hide Window"` text regardless of window visibility.
- **No full language-change rebuild** — static item labels do not update until app restart. Dynamic item labels (Toggle/Stop) update on the next state transition. See Language behaviour section above.
- **No device validation** in tray toggle-recording — `do_start_recording` returns `Err` on missing device; error is logged only.
- **No profile submenu** — out of scope.
- **Windows-only** `explorer.exe` for opening folder — app is Windows-only.
- **About label unchanged** — remains `"About"` / `"Про програму"`, not changed to `"About AudioCaptor"`.

---

## Error Handling

All tray event handlers are fire-and-forget. Errors are logged via `log::error!`. No user-facing error dialogs from tray actions in MVP.

`update_tray_recording_state` silently does nothing if `TrayMenuRefs` is not yet registered (tray failed to initialise) or if the mutex is poisoned.

`set_text()` and `set_enabled()` return `tauri::Result<()>` — all call sites use `let _ =` to explicitly discard errors.

---

## Testing

Manual verification checklist:
- [ ] Menu opens with correct items in both en and uk
- [ ] Show/Hide toggles window visibility; clicking again restores it
- [ ] Start → recording starts; toggle label becomes "Pause"; Stop becomes enabled
- [ ] Pause → recording pauses; toggle label becomes "Resume"
- [ ] Resume → recording resumes; toggle label becomes "Pause"
- [ ] Stop → recording stops; toggle label becomes "Start"; Stop becomes disabled
- [ ] Stop item click during Idle does nothing and does not crash (Err is logged)
- [ ] Open Recordings Folder opens correct active-profile folder in Explorer
- [ ] About opens About dialog
- [ ] Quit during recording shows confirm dialog; Quit when idle exits app
- [ ] `cargo build` compiles without errors (validates Send+Sync for TrayMenuRefs)
