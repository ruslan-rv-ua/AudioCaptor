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
| X button (close) — not recording | Always minimizes to tray |
| X button (close) — recording, `confirmExitDuringRecording = true` | Shows ConfirmExitDialog (unchanged) |
| X button (close) — recording, `confirmExitDuringRecording = false` | Minimizes to tray (recording continues silently — deliberate improvement over old "exit immediately") |
| Alt+F4 | Same as X button — goes through `onCloseRequested` |
| Dedicated button (⊟) in header | Always |
| Window loses focus | Only when `minimizeToTrayOnFocusLoss = true` (opt-in, default `false`), and no dialog open |

### Tray icon
- **Left click** → `window.show()` + `window.set_focus()` (safe when already visible — Tauri no-op)
- **Right click** → context menu with single item: **«Вийти»** / **«Quit»**

### «Вийти» / «Quit» from tray
- **Not recording** → `app.exit(0)` immediately (handled in Rust)
- **Recording** → Rust emits `tray-quit-requested` event → frontend checks `confirmExitDuringRecording`:
  - `true` → opens existing `ConfirmExitDialog`
    - «Зупинити та вийти» → `handleStopAndExit()` → `stopRecording()` + `api.quitApp()`
    - «Скасувати» → dialog closes, recording continues, window stays hidden
  - `false` → calls `handleStopAndExit()` directly, no dialog shown (consistent with X button behavior)

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
| `src-tauri/tauri.conf.json` | Add `"trayIcon"` config block under `"app"` |
| `src-tauri/src/lib.rs` | `pub mod tray`, call `tray::setup_tray()` in setup, add `quit_app` command to handler list |
| `src-tauri/src/settings.rs` | New field `minimize_to_tray_on_focus_loss: bool`, migration v4→v5 |
| `src-tauri/capabilities/default.json` | Add `core:window:allow-hide` and `core:tray:default` |
| `src/lib/types/index.ts` | Add `minimizeToTrayOnFocusLoss: boolean` to `Settings` interface |
| `src/lib/utils/invoke.ts` | Add `quitApp()` wrapper |
| `src/lib/stores/settings.svelte.ts` | New state var, getter, `loadSettingsFields`, setter |
| `src/App.svelte` | X button, Escape, focus loss, tray-quit-requested listener, new button, update `handleStopAndExit`, update `scheduleSave` |
| `src/lib/components/SettingsDialog.svelte` | New checkbox, new prop |
| `messages/en.json` | 2 new keys |
| `messages/uk.json` | 2 new keys |

---

## Rust: `tray.rs`

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
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
#[serde(default)]
pub minimize_to_tray_on_focus_loss: bool,  // NEW in v5

// impl Default for Settings — bump version to 5:
version: 5,
minimize_to_tray_on_focus_loss: false,

// migrate_settings — REPLACE existing `4 => { break; }` arm with:
4 => {
    settings.version = 5;
    // serde #[serde(default)] fills minimize_to_tray_on_focus_loss = false for old files
}
5 => {
    break; // terminal version
}
```

**Note:** The existing `4 => { break; }` arm must be **replaced** (not supplemented). The new `5 => { break; }` is the terminal arm.

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

// Add quit_app to invoke_handler — the full updated list:
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
    quit_app,          // NEW
])
```

---

## Rust: `Cargo.toml`

```toml
tauri = { version = "2", features = ["tray-icon"] }
```

---

## Config: `tauri.conf.json`

Add inside the `"app"` object:

```json
"trayIcon": {
  "iconPath": "icons/icon.ico",
  "iconAsTemplate": false
}
```

---

## TypeScript: `src/lib/types/index.ts`

Add to the `Settings` interface:

```ts
minimizeToTrayOnFocusLoss: boolean;
```

---

## TypeScript: `src/lib/utils/invoke.ts`

Add one function following the existing pattern:

```ts
export async function quitApp(): Promise<void> {
  return invoke("quit_app");
}
```

---

## TypeScript: `src/lib/stores/settings.svelte.ts`

```ts
// New $state variable (alongside existing ones):
let minimizeToTrayOnFocusLoss = $state(false);

// Add getter to getSettings() return object:
get minimizeToTrayOnFocusLoss() { return minimizeToTrayOnFocusLoss; },

// Add to loadSettingsFields():
minimizeToTrayOnFocusLoss = s.minimizeToTrayOnFocusLoss ?? false;

// New setter (alongside existing setters):
export function setMinimizeToTrayOnFocusLoss(value: boolean) {
  minimizeToTrayOnFocusLoss = value;
}
```

---

## Frontend: `App.svelte` changes

### Imports — add namespace import for `invoke.ts`

Add alongside the existing `{ saveSettings, loadSettings }` import:

```ts
import * as api from "./lib/utils/invoke";
```

### X button + Alt+F4 (onCloseRequested)

`Alt+F4` already calls `appWindow.close()` via the existing mnemonic handler, so it also flows through `onCloseRequested` — same behavior as X button, no extra change needed.

```ts
await appWindow.onCloseRequested(async (event) => {
    if (appSettings.confirmExitDuringRecording && isRecording) {
        event.preventDefault();
        confirmExitOpen = true;
    } else {
        // Not recording → minimize to tray.
        // Recording + confirmExitDuringRecording=false → also minimize to tray
        // (recording continues silently; deliberate improvement over old "exit immediately").
        event.preventDefault();
        await appWindow.hide();
    }
});
```

### handleStopAndExit — use `api.quitApp()` instead of `appWindow.close()`

`appWindow.close()` would now trigger `onCloseRequested` → minimize to tray instead of quitting.

```ts
async function handleStopAndExit() {
    confirmExitOpen = false;
    try { await stopRecording(); } catch { /* already stopped */ }
    await api.quitApp();
}
```

### scheduleSave — add new field to payload

```ts
await saveSettings({
    version: appSettings.version,
    selectedMic: recording.selectedMic,
    selectedLoopback: recording.selectedLoopback,
    hotkey: appSettings.hotkey,
    soundEnabled: appSettings.soundEnabled,
    language: appSettings.language,
    confirmExitDuringRecording: appSettings.confirmExitDuringRecording,
    minimizeToTrayOnFocusLoss: appSettings.minimizeToTrayOnFocusLoss,  // NEW
    profiles: profileStore.list,
    activeProfileId: profileStore.activeId,
    theme: appSettings.theme,
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

### Focus loss (onMount) — guard against open dialogs

```ts
const unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused
        && appSettings.minimizeToTrayOnFocusLoss
        && !settingsOpen && !dialogOpen && !confirmExitOpen && !deleteConfirmOpen) {
        void appWindow.hide();
    }
});
// in cleanup return: unlistenFocus();
```

### tray-quit-requested listener (onMount)

Respects `confirmExitDuringRecording` for consistency with the X button:

```ts
const unlistenTrayQuit = await listen("tray-quit-requested", async () => {
    if (isRecording) {
        if (appSettings.confirmExitDuringRecording) {
            if (!confirmExitOpen) confirmExitOpen = true;
        } else {
            await handleStopAndExit();
        }
    } else {
        void api.quitApp();
    }
});
// in cleanup return: unlistenTrayQuit();
```

### New button in header (left of settings button)

```svelte
<button
  type="button"
  class="btn-tray"
  aria-label={m.btn_minimize_to_tray_aria()}
  onclick={() => appWindow.hide()}
>⊟</button>
<button type="button" class="btn-settings" ...>⚙</button>
```

Style — identical to `.btn-settings` (36×36px, same border, radius, hover):

```css
.btn-tray {
  width: 36px; height: 36px;
  background: var(--surface); border: 1px solid var(--border);
  border-radius: 7px; cursor: pointer; font-size: 20px;
  display: flex; align-items: center; justify-content: center;
  color: var(--text-primary);
  box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  transition: background 0.1s;
}
.btn-tray:hover { background: var(--surface-hover); }
```

---

## Frontend: `SettingsDialog.svelte` changes

New prop and handler:
```ts
interface Props {
  // ... existing props ...
  minimizeToTrayOnFocusLoss: boolean;
  onminimizetotraytoggle: (v: boolean) => void;
}
```

New checkbox field (after «Confirm exit during recording»):
```svelte
<div class="field">
    <label class="checkbox-label">
        <input type="checkbox" checked={minimizeToTrayOnFocusLoss}
               onchange={(e) => onminimizetotraytoggle((e.target as HTMLInputElement).checked)} />
        {m.settings_minimize_on_focus_loss()}
    </label>
</div>
```

In `App.svelte`, pass the new props to `<SettingsDialog>`:
```svelte
minimizeToTrayOnFocusLoss={appSettings.minimizeToTrayOnFocusLoss}
onminimizetotraytoggle={(v) => {
    setMinimizeToTrayOnFocusLoss(v);
    scheduleSave();
}}
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
"core:window:allow-hide",
"core:tray:default"
```

---

## Out of scope

- Recording controls in tray menu (explicitly deferred)
- Tray icon reflecting recording state (badge/animation)
- Localization of tray menu via paraglide (hardcoded en/uk in Rust based on `settings.language`)
