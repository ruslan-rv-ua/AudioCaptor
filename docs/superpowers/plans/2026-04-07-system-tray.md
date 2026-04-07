# System Tray Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add system tray support so AudioCaptor minimizes to tray instead of exiting, with a tray icon that restores the window on left-click and offers a «Вийти»/«Quit» menu item on right-click.

**Architecture:** A new `tray.rs` Rust module (consistent with `hotkey.rs`, `device_monitor.rs`) handles tray setup and quit logic. Frontend (`App.svelte`) wires the minimize triggers (Escape, X button, focus loss, dedicated button) and listens for `tray-quit-requested` events from Rust. Settings gain a `minimizeToTrayOnFocusLoss` boolean (v5 migration).

**Tech Stack:** Tauri v2 (`tauri::tray`, `tauri::menu`), Svelte 5 (runes), Rust 2021, paraglide i18n (en/uk)

**Spec:** `docs/superpowers/specs/2026-04-07-system-tray-design.md`

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `src-tauri/src/tray.rs` | **Create** | Tray icon + menu setup, quit event handling |
| `src-tauri/src/settings.rs` | **Modify** | New field `minimize_to_tray_on_focus_loss`, v4→v5 migration |
| `src-tauri/src/lib.rs` | **Modify** | Register `tray` module, wire `setup_tray()`, add `quit_app` command |
| `src-tauri/Cargo.toml` | **Modify** | Add `"tray-icon"` feature to `tauri` dependency |
| `src-tauri/tauri.conf.json` | **Modify** | Add `trayIcon` config block |
| `src-tauri/capabilities/default.json` | **Modify** | Add `core:window:allow-hide`, `core:tray:default` |
| `messages/en.json` | **Modify** | 2 new i18n keys |
| `messages/uk.json` | **Modify** | 2 new i18n keys (Ukrainian) |
| `src/paraglide/messages/btn_minimize_to_tray_aria.js` | **Create** | Generated i18n message |
| `src/paraglide/messages/settings_minimize_on_focus_loss.js` | **Create** | Generated i18n message |
| `src/paraglide/messages/_index.js` | **Modify** | Export 2 new message files |
| `src/lib/types/index.ts` | **Modify** | Add `minimizeToTrayOnFocusLoss: boolean` to `Settings` |
| `src/lib/utils/invoke.ts` | **Modify** | Add `quitApp()` typed wrapper |
| `src/lib/stores/settings.svelte.ts` | **Modify** | New state var, getter, `loadSettingsFields`, setter |
| `src/lib/components/SettingsDialog.svelte` | **Modify** | New checkbox + prop for focus-loss setting |
| `src/App.svelte` | **Modify** | X button, Escape, focus loss, tray-quit listener, ⊟ button |

---

## Task 1: Settings — v4→v5 Migration

**Files:**
- Modify: `src-tauri/src/settings.rs`

> Write tests first, then implement. All existing tests must still pass.

- [ ] **Step 1.1: Write failing tests**

Open `src-tauri/src/settings.rs`. In the `#[cfg(test)] mod tests` block, add these tests after the existing `migrate_v3_to_v4_adds_theme` test:

```rust
#[test]
fn settings_default_has_version_5() {
    let s = Settings::default();
    assert_eq!(s.version, 5);
    assert!(!s.minimize_to_tray_on_focus_loss);
}

#[test]
fn migrate_v4_to_v5_adds_minimize_to_tray_on_focus_loss() {
    let json = r#"{
        "version": 4,
        "selectedMic": null,
        "selectedLoopback": null,
        "hotkey": "Pause",
        "soundEnabled": true,
        "profiles": [],
        "activeProfileId": "default",
        "language": "en",
        "confirmExitDuringRecording": true,
        "theme": "auto"
    }"#;
    let settings: Settings = serde_json::from_str(json).unwrap();
    let migrated = migrate_settings(settings);
    assert_eq!(migrated.version, 5);
    assert!(!migrated.minimize_to_tray_on_focus_loss);
}

#[test]
fn migrate_v4_preserves_existing_minimize_to_tray_false() {
    // If somehow the field is already present, serde preserves it
    let json = r#"{
        "version": 4,
        "selectedMic": null,
        "selectedLoopback": null,
        "hotkey": "Pause",
        "soundEnabled": true,
        "profiles": [],
        "activeProfileId": "default",
        "language": "en",
        "confirmExitDuringRecording": true,
        "theme": "auto",
        "minimizeToTrayOnFocusLoss": false
    }"#;
    let settings: Settings = serde_json::from_str(json).unwrap();
    let migrated = migrate_settings(settings);
    assert_eq!(migrated.version, 5);
    assert!(!migrated.minimize_to_tray_on_focus_loss);
}
```

Also update the existing `settings_default_has_version_4` test — rename it and update the assertion:

```rust
// RENAME: settings_default_has_version_4 → settings_default_has_version_5 (done above)
// DELETE the old test named settings_default_has_version_4
```

- [ ] **Step 1.2: Run tests — expect failures**

```bash
cd src-tauri && cargo test settings_default_has_version
```

Expected: compile errors (field doesn't exist yet) or test failures.

- [ ] **Step 1.3: Implement the migration**

In `src-tauri/src/settings.rs`, make these changes:

**a) Add the new field to `Settings` struct** (after `theme`):

```rust
#[serde(default)]
pub minimize_to_tray_on_focus_loss: bool,  // NEW in v5
```

**b) Update `impl Default for Settings`** — change `version: 4` to `version: 5` and add the new field:

```rust
version: 5,
// ... existing fields ...
minimize_to_tray_on_focus_loss: false,
```

**c) Replace the terminal migration arm** — find `4 => { // Terminal version. break; }` and replace the entire arm:

```rust
4 => {
    settings.version = 5;
    // serde #[serde(default)] fills minimize_to_tray_on_focus_loss = false for old files
}
5 => {
    break; // terminal version
}
```

**WARNING:** Replace the existing `4 => { break; }` arm entirely. Do not add a new arm below it — that would be unreachable.

**d) Delete the old `settings_default_has_version_4` test** (it's now replaced by `settings_default_has_version_5`).

- [ ] **Step 1.4: Run all settings tests**

```bash
cd src-tauri && cargo test settings
```

Expected: all pass. Pay attention to any test that asserts `version == 4` — those need updating to `5`.

- [ ] **Step 1.5: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat(settings): migrate v4→v5, add minimize_to_tray_on_focus_loss"
```

---

## Task 2: Rust Infrastructure — Cargo, Config, Capabilities, quit_app

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 2.1: Add tray-icon feature to Cargo.toml**

In `src-tauri/Cargo.toml`, find:
```toml
tauri = { version = "2", features = [] }
```
Replace with:
```toml
tauri = { version = "2", features = ["tray-icon"] }
```

- [ ] **Step 2.2: Add trayIcon config to tauri.conf.json**

In `src-tauri/tauri.conf.json`, inside the `"app"` object (after `"windows": [...]`), add:

```json
"trayIcon": {
  "iconPath": "icons/icon.ico",
  "iconAsTemplate": false
},
```

- [ ] **Step 2.3: Add capabilities**

In `src-tauri/capabilities/default.json`, add two permissions to the `"permissions"` array:

```json
"core:window:allow-hide",
"core:tray:default"
```

- [ ] **Step 2.4: Add quit_app command and tray module declaration to lib.rs**

In `src-tauri/src/lib.rs`:

**a) Add module declaration** at the top with the other `pub mod` lines:

```rust
pub mod tray;
```

**b) Add quit_app command** (place it near the other simple commands, e.g., after `set_sound_enabled`):

```rust
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
```

**c) Add `quit_app` to the `invoke_handler`** — find `tauri::generate_handler![` and add `quit_app,` as the last entry before `]`.

- [ ] **Step 2.5: Create empty tray.rs placeholder**

Create `src-tauri/src/tray.rs` with just enough to compile:

```rust
pub fn setup_tray(_app: &tauri::AppHandle, _language: &str) -> tauri::Result<()> {
    Ok(())
}
```

- [ ] **Step 2.6: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: compiles without errors. Fix any issues before proceeding.

- [ ] **Step 2.7: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/capabilities/default.json src-tauri/src/lib.rs src-tauri/src/tray.rs
git commit -m "feat(tray): add infrastructure — cargo feature, config, capabilities, quit_app command"
```

---

## Task 3: Rust — tray.rs Implementation

**Files:**
- Modify: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs` (wire setup_tray call)

- [ ] **Step 3.1: Implement tray.rs**

Replace the placeholder content of `src-tauri/src/tray.rs` with the full implementation:

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
```

- [ ] **Step 3.2: Wire setup_tray in lib.rs**

In `src-tauri/src/lib.rs`, inside the `.setup(|app| { ... })` closure, find the line:

```rust
log::info!("AudioCaptor started");
```

Add the tray setup call **before** that log line:

```rust
// Set up system tray icon
if let Err(e) = tray::setup_tray(app.handle(), &settings.language) {
    log::error!("Failed to set up tray icon: {e}");
}

log::info!("AudioCaptor started");
```

- [ ] **Step 3.3: Compile check**

```bash
cd src-tauri && cargo check
```

Expected: compiles cleanly. If there are import/type errors in `tray.rs`, fix them before proceeding. Common issues:
- `TrayIconEvent` import path — verify it is `tauri::tray::TrayIconEvent`
- `MouseButton`/`MouseButtonState` — verify they are in `tauri::tray`

- [ ] **Step 3.4: Run all Rust tests**

```bash
cd src-tauri && cargo test
```

Expected: all tests pass.

- [ ] **Step 3.5: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat(tray): implement tray.rs — icon, quit menu, left-click restore"
```

---

## Task 4: TypeScript Foundation — Types, Invoke, Store

**Files:**
- Modify: `src/lib/types/index.ts`
- Modify: `src/lib/utils/invoke.ts`
- Modify: `src/lib/stores/settings.svelte.ts`

- [ ] **Step 4.1: Add field to Settings interface**

In `src/lib/types/index.ts`, find the `Settings` interface and add `minimizeToTrayOnFocusLoss` after `confirmExitDuringRecording`:

```ts
export interface Settings {
  version: number;
  selectedMic: string | null;
  selectedLoopback: string | null;
  hotkey: string | null;
  soundEnabled: boolean;
  profiles: RecordingProfile[];
  activeProfileId: string;
  language: "en" | "uk";
  confirmExitDuringRecording: boolean;
  minimizeToTrayOnFocusLoss: boolean;  // NEW
  theme: Theme;
}
```

- [ ] **Step 4.2: Add quitApp to invoke.ts**

In `src/lib/utils/invoke.ts`, add at the end of the file, following the existing pattern:

```ts
export async function quitApp(): Promise<void> {
  return invoke("quit_app");
}
```

- [ ] **Step 4.3: Update settings store**

In `src/lib/stores/settings.svelte.ts`:

**a)** Add new state variable alongside the existing ones (e.g., after `let theme`):

```ts
let minimizeToTrayOnFocusLoss = $state(false);
```

**b)** Add getter to the `getSettings()` return object (after `get theme()`):

```ts
get minimizeToTrayOnFocusLoss() { return minimizeToTrayOnFocusLoss; },
```

**c)** Add to `loadSettingsFields()` (after `theme = s.theme ?? "auto";`):

```ts
minimizeToTrayOnFocusLoss = s.minimizeToTrayOnFocusLoss ?? false;
```

**d)** Add new setter at the end of the file:

```ts
export function setMinimizeToTrayOnFocusLoss(value: boolean) {
  minimizeToTrayOnFocusLoss = value;
}
```

- [ ] **Step 4.4: Verify frontend build**

```bash
pnpm vite:build
```

Expected: builds without TypeScript errors. If there are errors about `minimizeToTrayOnFocusLoss` being missing somewhere, fix them now.

- [ ] **Step 4.5: Commit**

```bash
git add src/lib/types/index.ts src/lib/utils/invoke.ts src/lib/stores/settings.svelte.ts
git commit -m "feat(ts): add types, invoke wrapper, and store for minimize-to-tray"
```

---

## Task 5: i18n — New Message Keys

**Files:**
- Modify: `messages/en.json`
- Modify: `messages/uk.json`
- Create: `src/paraglide/messages/btn_minimize_to_tray_aria.js`
- Create: `src/paraglide/messages/settings_minimize_on_focus_loss.js`
- Modify: `src/paraglide/messages/_index.js`

- [ ] **Step 5.1: Add keys to messages/en.json**

In `messages/en.json`, add two entries at the end (before the closing `}`):

```json
"btn_minimize_to_tray_aria": "Minimize to tray",
"settings_minimize_on_focus_loss": "Minimize to tray when window loses focus"
```

- [ ] **Step 5.2: Add keys to messages/uk.json**

In `messages/uk.json`, add the same two keys at the end:

```json
"btn_minimize_to_tray_aria": "Згорнути в трей",
"settings_minimize_on_focus_loss": "Згортати в трей при втраті фокуса"
```

- [ ] **Step 5.3: Create btn_minimize_to_tray_aria.js**

Create `src/paraglide/messages/btn_minimize_to_tray_aria.js` following the exact pattern of existing message files (e.g., `settings_sound_label.js`):

```js
/* eslint-disable */
import { getLocale, experimentalStaticLocale } from '../runtime.js';

/** @typedef {import('../runtime.js').LocalizedString} LocalizedString */

/** @typedef {{}} Btn_Minimize_To_Tray_AriaInputs */

const en_btn_minimize_to_tray_aria = /** @type {(inputs: Btn_Minimize_To_Tray_AriaInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Minimize to tray`)
};

const uk_btn_minimize_to_tray_aria = /** @type {(inputs: Btn_Minimize_To_Tray_AriaInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Згорнути в трей`)
};

/**
* | output |
* | --- |
* | "Minimize to tray" |
*
* @param {Btn_Minimize_To_Tray_AriaInputs} inputs
* @param {{ locale?: "en" | "uk" }} options
* @returns {LocalizedString}
*/
export const btn_minimize_to_tray_aria = /** @type {((inputs?: Btn_Minimize_To_Tray_AriaInputs, options?: { locale?: "en" | "uk" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Btn_Minimize_To_Tray_AriaInputs, { locale?: "en" | "uk" }, {}>} */ ((inputs = {}, options = {}) => {
	const locale = experimentalStaticLocale ?? options.locale ?? getLocale()
	if (locale === "en") return en_btn_minimize_to_tray_aria(inputs)
	return uk_btn_minimize_to_tray_aria(inputs)
});
```

- [ ] **Step 5.4: Create settings_minimize_on_focus_loss.js**

Create `src/paraglide/messages/settings_minimize_on_focus_loss.js`:

```js
/* eslint-disable */
import { getLocale, experimentalStaticLocale } from '../runtime.js';

/** @typedef {import('../runtime.js').LocalizedString} LocalizedString */

/** @typedef {{}} Settings_Minimize_On_Focus_LossInputs */

const en_settings_minimize_on_focus_loss = /** @type {(inputs: Settings_Minimize_On_Focus_LossInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Minimize to tray when window loses focus`)
};

const uk_settings_minimize_on_focus_loss = /** @type {(inputs: Settings_Minimize_On_Focus_LossInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Згортати в трей при втраті фокуса`)
};

/**
* | output |
* | --- |
* | "Minimize to tray when window loses focus" |
*
* @param {Settings_Minimize_On_Focus_LossInputs} inputs
* @param {{ locale?: "en" | "uk" }} options
* @returns {LocalizedString}
*/
export const settings_minimize_on_focus_loss = /** @type {((inputs?: Settings_Minimize_On_Focus_LossInputs, options?: { locale?: "en" | "uk" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Settings_Minimize_On_Focus_LossInputs, { locale?: "en" | "uk" }, {}>} */ ((inputs = {}, options = {}) => {
	const locale = experimentalStaticLocale ?? options.locale ?? getLocale()
	if (locale === "en") return en_settings_minimize_on_focus_loss(inputs)
	return uk_settings_minimize_on_focus_loss(inputs)
});
```

- [ ] **Step 5.5: Add exports to _index.js**

In `src/paraglide/messages/_index.js`, append two lines at the end:

```js
export * from './btn_minimize_to_tray_aria.js'
export * from './settings_minimize_on_focus_loss.js'
```

- [ ] **Step 5.6: Commit**

```bash
git add messages/en.json messages/uk.json src/paraglide/messages/btn_minimize_to_tray_aria.js src/paraglide/messages/settings_minimize_on_focus_loss.js src/paraglide/messages/_index.js
git commit -m "feat(i18n): add tray-related message keys (en/uk)"
```

---

## Task 6: SettingsDialog — New Checkbox

**Files:**
- Modify: `src/lib/components/SettingsDialog.svelte`

- [ ] **Step 6.1: Add props and handler to SettingsDialog**

In `src/lib/components/SettingsDialog.svelte`, find the `interface Props` block and add two new entries:

```ts
interface Props {
  open: boolean;
  hotkey: string | null;
  soundEnabled: boolean;
  confirmExitDuringRecording: boolean;
  minimizeToTrayOnFocusLoss: boolean;   // NEW
  language: "en" | "uk";
  theme: Theme;
  onclose: () => void;
  onsave: (patch: Partial<import("../types").Settings>) => void;
  onthemechange: (t: Theme) => void;
  onminimizetotraytoggle: (v: boolean) => void;  // NEW
}
```

Update the destructuring to include the new props (find the `let { open, ... }: Props = $props();` line):

```ts
let {
  open,
  hotkey,
  soundEnabled,
  confirmExitDuringRecording,
  minimizeToTrayOnFocusLoss,           // NEW
  language,
  theme,
  onclose,
  onsave,
  onthemechange,
  onminimizetotraytoggle,              // NEW
}: Props = $props();
```

- [ ] **Step 6.2: Add checkbox to SettingsDialog template**

Find the «Confirm exit» checkbox field block (the `<div class="field">` containing `settings_confirm_exit_label`) and add the new field **after** it:

```svelte
<!-- Minimize to tray on focus loss -->
<div class="field">
  <label class="checkbox-label">
    <input
      type="checkbox"
      checked={minimizeToTrayOnFocusLoss}
      onchange={(e) => onminimizetotraytoggle((e.target as HTMLInputElement).checked)}
    />
    {m.settings_minimize_on_focus_loss()}
  </label>
</div>
```

- [ ] **Step 6.3: Commit**

```bash
git add src/lib/components/SettingsDialog.svelte
git commit -m "feat(settings-dialog): add minimize-to-tray-on-focus-loss checkbox"
```

---

## Task 7: App.svelte — Wire All Behaviors

**Files:**
- Modify: `src/App.svelte`

This task wires all the tray behaviors together. Make all changes to `App.svelte` in this task.

- [ ] **Step 7.1: Add imports**

At the top of the `<script>` block, add two new imports alongside the existing ones:

```ts
import { listen } from "@tauri-apps/api/event";
import * as api from "./lib/utils/invoke";
```

Also add the new store setter to the existing `settings.svelte` import:

```ts
import { getSettings, loadSettingsFields, setMinimizeToTrayOnFocusLoss } from "./lib/stores/settings.svelte";
```

- [ ] **Step 7.2: Update handleStopAndExit**

Find `handleStopAndExit` (currently calls `appWindow.close()`). Replace the body:

```ts
async function handleStopAndExit() {
  confirmExitOpen = false;
  try { await stopRecording(); } catch { /* already stopped */ }
  await api.quitApp();
}
```

- [ ] **Step 7.3: Update scheduleSave**

Find the `scheduleSave` function and the `saveSettings({...})` call. Add `minimizeToTrayOnFocusLoss` to the payload (after `confirmExitDuringRecording`):

```ts
confirmExitDuringRecording: appSettings.confirmExitDuringRecording,
minimizeToTrayOnFocusLoss: appSettings.minimizeToTrayOnFocusLoss,  // NEW
```

- [ ] **Step 7.4: Update onCloseRequested**

In the `onMount` async block, find the existing `onCloseRequested` handler:

```ts
// OLD:
await appWindow.onCloseRequested(async (event) => {
  if (appSettings.confirmExitDuringRecording && isRecording) {
    event.preventDefault();
    confirmExitOpen = true;
  }
});
```

Replace with:

```ts
// NEW:
await appWindow.onCloseRequested(async (event) => {
  if (appSettings.confirmExitDuringRecording && isRecording) {
    event.preventDefault();
    confirmExitOpen = true;
  } else {
    // Not recording → minimize to tray.
    // Recording + confirmExitDuringRecording=false → minimize to tray too
    // (recording continues silently; better than old "exit immediately").
    event.preventDefault();
    await appWindow.hide();
  }
});
```

- [ ] **Step 7.5: Add focus loss listener**

In the `onMount` async block, after the `onCloseRequested` setup, add:

```ts
const unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
  if (
    !focused &&
    appSettings.minimizeToTrayOnFocusLoss &&
    !settingsOpen && !dialogOpen && !confirmExitOpen && !deleteConfirmOpen
  ) {
    void appWindow.hide();
  }
});
```

- [ ] **Step 7.6: Add tray-quit-requested listener**

In the `onMount` async block, after the focus listener, add:

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
```

- [ ] **Step 7.7: Add Escape key to handleMnemonic**

`handleMnemonic` has `if (!e.altKey) return;` near the top — this would block Escape before reaching any switch case. **Do NOT add to the switch.** Instead, add an early-return check at the very top of `handleMnemonic`, before the `altKey` guard:

```ts
function handleMnemonic(e: KeyboardEvent) {
  // Escape: minimize to tray (only when no dialog is open)
  if (e.key === "Escape" && !settingsOpen && !dialogOpen && !confirmExitOpen && !deleteConfirmOpen) {
    e.preventDefault();
    void appWindow.hide();
    return;
  }

  if (!e.altKey) return;
  // ... existing switch unchanged ...
}
```

- [ ] **Step 7.8: Register cleanup for new listeners**

In the `onMount` return (cleanup function), add the two new unlisten calls:

```ts
return () => {
  window.removeEventListener("keydown", handleMnemonic);
  cleanupTheme();
  unlistenFocus();       // NEW
  unlistenTrayQuit();    // NEW
};
```

**Note:** `unlistenFocus` and `unlistenTrayQuit` are declared inside the async IIFE in `onMount`. They need to be declared in the outer scope so the cleanup function can access them. Declare them before the async IIFE:

```ts
let unlistenFocus: (() => void) | undefined;
let unlistenTrayQuit: (() => void) | undefined;
```

Then assign inside the async IIFE:
```ts
unlistenFocus = await appWindow.onFocusChanged(...);
unlistenTrayQuit = await listen("tray-quit-requested", ...);
```

And cleanup:
```ts
return () => {
  window.removeEventListener("keydown", handleMnemonic);
  cleanupTheme();
  unlistenFocus?.();
  unlistenTrayQuit?.();
};
```

- [ ] **Step 7.9: Add the ⊟ button to the header**

In the `{#if initialized}` template section, find the `.header-row` div:

```svelte
<div class="header-row">
  <h1>{m.app_title()}</h1>
  <button
    type="button"
    class="btn-settings"
    ...
  >⚙</button>
</div>
```

Add the new button **before** `btn-settings`:

```svelte
<div class="header-row">
  <h1>{m.app_title()}</h1>
  <button
    type="button"
    class="btn-tray"
    aria-label={m.btn_minimize_to_tray_aria()}
    onclick={() => appWindow.hide()}
  >⊟</button>
  <button
    type="button"
    class="btn-settings"
    aria-label={m.settings_btn_aria()}
    onclick={() => settingsOpen = true}
  >⚙</button>
</div>
```

Add the CSS for `.btn-tray` in the `<style>` section, after the `.btn-settings` rule:

```css
.btn-tray {
  width: 36px;
  height: 36px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
  cursor: pointer;
  font-size: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  transition: background 0.1s;
}
.btn-tray:hover {
  background: var(--surface-hover);
}
```

- [ ] **Step 7.10: Wire SettingsDialog new props**

Find `<SettingsDialog` in the template and add the two new props:

```svelte
<SettingsDialog
  open={settingsOpen}
  hotkey={appSettings.hotkey}
  soundEnabled={appSettings.soundEnabled}
  confirmExitDuringRecording={appSettings.confirmExitDuringRecording}
  minimizeToTrayOnFocusLoss={appSettings.minimizeToTrayOnFocusLoss}
  language={appSettings.language}
  theme={appSettings.theme}
  onclose={async () => { settingsOpen = false; await focusProfileSelect(); }}
  onsave={handleSettingsSave}
  onthemechange={handleThemeChange}
  onminimizetotraytoggle={(v) => {
    setMinimizeToTrayOnFocusLoss(v);
    scheduleSave();
  }}
/>
```

- [ ] **Step 7.11: Build check**

```bash
pnpm vite:build
```

Expected: builds without TypeScript or Svelte errors. Fix any type errors before proceeding.

- [ ] **Step 7.12: Full Rust build check**

```bash
cd src-tauri && cargo check
```

Expected: clean.

- [ ] **Step 7.13: Commit**

```bash
git add src/App.svelte
git commit -m "feat(app): wire minimize-to-tray behaviors — X button, Escape, focus loss, tray events, header button"
```

---

## Task 8: Final Verification

- [ ] **Step 8.1: Run all Rust tests**

```bash
cd src-tauri && cargo test
```

Expected: all tests pass.

- [ ] **Step 8.2: Manual smoke test checklist**

Build and run the app (`pnpm tauri dev` or run the release binary). Verify each behavior:

```
[ ] ⊟ button in header minimizes window to tray
[ ] Tray icon appears in Windows system tray
[ ] Left-click tray icon → window shows and gets focus
[ ] Right-click tray icon → menu appears with single "Quit"/"Вийти" item
[ ] "Quit" when not recording → app exits
[ ] "Quit" when recording, confirmExitDuringRecording=true → ConfirmExitDialog appears
[ ] "Quit" when recording, confirmExitDuringRecording=false → app stops recording and exits immediately
[ ] X button when not recording → window minimizes to tray (does NOT exit)
[ ] X button when recording, confirmExitDuringRecording=true → ConfirmExitDialog
[ ] «Зупинити та вийти» in ConfirmExitDialog → stops recording and fully exits
[ ] Escape key with no dialogs open → minimizes to tray
[ ] Escape key while SettingsDialog open → closes dialog only, does NOT minimize
[ ] Settings: "Minimize to tray when window loses focus" checkbox appears
[ ] Focus-loss minimize: enable checkbox, click another window → app minimizes to tray
[ ] Focus-loss minimize: disabled by default (checkbox unchecked on fresh install)
[ ] Settings version in settings.json is 5 after first run
[ ] Old settings.json (version 4) is migrated to version 5 on startup
[ ] Language uk: tray menu shows "Вийти", language en: shows "Quit"
```

- [ ] **Step 8.3: Final commit if any fixups**

```bash
git add -p  # stage only intentional changes
git commit -m "fix(tray): address smoke test findings"
```

---

## Reference

- Spec: `docs/superpowers/specs/2026-04-07-system-tray-design.md`
- Tauri v2 tray API: `tauri::tray::TrayIconBuilder`
- Tauri v2 menu API: `tauri::menu::Menu`, `tauri::menu::MenuItem`
- Existing patterns: `src-tauri/src/hotkey.rs` (module structure), `src-tauri/src/settings.rs` (migration pattern)
