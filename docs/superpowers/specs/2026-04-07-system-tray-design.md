# System Tray — Design Spec

**Date:** 2026-04-07
**Branch:** feature/separate-mode
**Status:** Approved

---

## Overview

Add system tray support to AudioCaptor. The app minimizes to tray instead of exiting, keeps running in the background, and can be restored via tray icon or tray menu.

---

## Behaviour

### Minimize to tray triggers

| Trigger | Condition |
|---|---|
| Escape key | Main window focused, no dialog open |
| X button (close) | Window not recording |
| Dedicated button (⊟) in header | Always |
| Window loses focus | Only when `minimizeToTrayOnFocusLoss = true` (opt-in setting, default `false`) |

### X button during recording
Unchanged — existing `ConfirmExitDialog` is shown («Зупинити та вийти» / «Скасувати»).

### Tray icon
- **Left click** → `window.show()` + `window.set_focus()`
- **Right click** → context menu with single item: **«Вийти»** / **«Quit»**

### «Вийти» / «Quit» from tray
- **Not recording** → `app.exit(0)` immediately (handled in Rust)
- **Recording** → Rust emits `tray-quit-requested` event → frontend opens existing `ConfirmExitDialog`
  - «Зупинити та вийти» → `stopRecording()` then `invoke("quit_app")` → `app.exit(0)`
  - «Скасувати» → dialog closes, recording continues, window stays hidden

---

## Architecture

### Approach chosen: dedicated `tray.rs` module

Consistent with existing module pattern (`hotkey.rs`, `device_monitor.rs`). Keeps `lib.rs` clean.

---

## File Changes

### New files

| File | Purpose |
|---|---|
| `src-tauri/src/tray.rs` | Tray icon setup (~70 lines) |
| `src/paraglide/messages/btn_minimize_to_tray_aria.js` | Generated i18n |
| `src/paraglide/messages/settings_minimize_on_focus_loss.js` | Generated i18n |

### Modified files

| File | Change |
|---|---|
| `src-tauri/Cargo.toml` | Add `"tray-icon"` feature to `tauri` |
| `src-tauri/src/lib.rs` | `pub mod tray`, call `tray::setup_tray()` in setup, add `quit_app` command |
| `src-tauri/src/settings.rs` | New field `minimize_to_tray_on_focus_loss: bool`, migration v4→v5 |
| `src-tauri/capabilities/default.json` | Add `core:window:allow-hide` |
| `src/App.svelte` | X button, Escape, focus loss, tray-quit-requested listener, new button |
| `src/lib/components/SettingsDialog.svelte` | New checkbox, new prop |
| `src/lib/stores/settings.svelte.ts` | New field + setter |
| `messages/en.json` | 2 new keys |
| `messages/uk.json` | 2 new keys |

---

## Rust: `tray.rs`

```rust
pub fn setup_tray(app: &tauri::AppHandle, language: &str) -> tauri::Result<()> {
    let quit_label = if language == "uk" { "Вийти" } else { "Quit" };
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_item])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("AudioCaptor")
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                let state = app.state::<SharedState>();
                let is_recording = state.lock()
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
                button_state: MouseButtonState::Up, ..
            } = event {
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
```

---

## Rust: `settings.rs` migration

```rust
// New field in Settings struct
pub minimize_to_tray_on_focus_loss: bool,  // NEW in v5, default false

// Default::default() — version bumped to 5
// Migration arm:
4 => {
    settings.version = 5;
    // serde #[serde(default)] fills minimize_to_tray_on_focus_loss = false
}
5 => { break; }
```

---

## Rust: `lib.rs` additions

```rust
pub mod tray;

// In setup():
tray::setup_tray(app.handle(), &settings.language)?;

// New command:
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
```

---

## Frontend: `App.svelte` changes

### X button (onCloseRequested)
```ts
await appWindow.onCloseRequested(async (event) => {
    if (isRecording) {
        event.preventDefault();
        confirmExitOpen = true;
    } else {
        event.preventDefault();
        await appWindow.hide();
    }
});
```

### Escape key (handleMnemonic)
```ts
case "Escape":
    if (!settingsOpen && !dialogOpen && !confirmExitOpen && !deleteConfirmOpen) {
        e.preventDefault();
        await appWindow.hide();
    }
    break;
```

### Focus loss (onMount)
```ts
const unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused && appSettings.minimizeToTrayOnFocusLoss) {
        void appWindow.hide();
    }
});
// in cleanup: unlistenFocus();
```

### tray-quit-requested listener (onMount)
```ts
const unlistenTrayQuit = await listen("tray-quit-requested", () => {
    if (isRecording) confirmExitOpen = true;
    else invoke("quit_app");
});
// in cleanup: unlistenTrayQuit();
```

### New button in header
```svelte
<button class="btn-tray" aria-label={m.btn_minimize_to_tray_aria()}
    onclick={() => appWindow.hide()}>⊟</button>
<button class="btn-settings" ...>⚙</button>
```
Style: identical to `.btn-settings` (36×36px, same border/radius/hover).

---

## Frontend: `SettingsDialog.svelte` changes

New prop `minimizeToTrayOnFocusLoss: boolean` and handler `onminimizetotraytoggle`.

New checkbox field (after «Confirm exit during recording»):
```svelte
<div class="field">
    <label class="checkbox-label">
        <input type="checkbox" checked={minimizeToTrayOnFocusLoss}
               onchange={handleMinimizeToTrayToggle} />
        {m.settings_minimize_on_focus_loss()}
    </label>
</div>
```

---

## i18n

### `messages/en.json`
```json
"btn_minimize_to_tray_aria": "Minimize to tray",
"settings_minimize_on_focus_loss": "Minimize to tray when window loses focus"
```

### `messages/uk.json`
```json
"btn_minimize_to_tray_aria": "Згорнути в трей",
"settings_minimize_on_focus_loss": "Згортати в трей при втраті фокуса"
```

---

## Settings serialization

```json
{
  "version": 5,
  "minimizeToTrayOnFocusLoss": false
}
```

Old settings files (v4) get `minimizeToTrayOnFocusLoss: false` automatically via `#[serde(default)]`.

---

## Capabilities

```json
"core:window:allow-hide"
```

---

## Out of scope

- Recording controls in tray menu (explicitly deferred)
- Tray icon reflecting recording state (badge/animation)
- Localization of tray menu via paraglide (hardcoded en/uk in Rust based on `settings.language`)
