# Phase 4 Design: i18n, Settings Dialog, aria-live, Scoop

**Date:** 2026-04-04  
**Branch:** feature/pahse-4  
**Status:** Approved

---

## Overview

Phase 4 adds four capabilities to AudioCaptor:

1. **Internationalization** — Paraglide JS 2.0 with English (default) and Ukrainian
2. **Settings modal dialog** — dedicated `SettingsDialog.svelte` for hotkey, sound, confirm-exit, language
3. **aria-live regions** — localized screen reader announcements for recording state
4. **Scoop manifest** — portable install/update via Scoop with persist support

---

## Architecture: Approach B — Settings Store + SettingsDialog

Follows existing project patterns (`profiles.svelte.ts` → `settings.svelte.ts`). Clear separation of concerns: `recording` store handles audio state, `settings` store handles configuration.

---

## Section 1: Internationalization (Paraglide JS)

### Setup

- Install `@inlang/paraglide-js` with Vite plugin (`@inlang/paraglide-js/vite`)
- Configure in `vite.config.ts` — works with plain Svelte (no SvelteKit required)
- Paraglide compiles messages to `src/paraglide/` at build time (tree-shakeable, typed)

### Message files

```
messages/
  en.json   ← English (fallback, default)
  uk.json   ← Ukrainian
```

No `ru.json` — prohibited by project requirements. If system language is `ru`, falls back to `uk`.

### Initialization

New file `src/lib/i18n.ts`:
```ts
import { setLanguageTag } from "../paraglide/runtime";
export function initLanguage(lang: "en" | "uk") {
  setLanguageTag(lang);
}
```

Called once in `App.svelte` `onMount` after `loadSettingsIntoStore()`, before rendering any localized strings.

### Language detection (first run)

```ts
function detectLanguage(): "en" | "uk" {
  const lang = navigator.language.toLowerCase();
  if (lang.startsWith("uk") || lang.startsWith("ru")) return "uk";
  return "en";
}
```

On first run (no `language` in settings), detect and persist. Subsequent runs read saved value.

### Usage in components

```ts
import * as m from "../paraglide/messages";
// in template:
{m.microphone_label()}
{m.recording_started()}
```

All hardcoded UI strings replaced with typed Paraglide function calls.

### Error message mapping

Rust backend returns structured error codes (e.g., `"DEVICE_NOT_FOUND"` — already in use). Frontend maps these to localized strings via Paraglide. No translation duplication on backend.

### Language change behavior

Saving a new language writes to `settings.json`. Change takes effect after app restart (no hot reload required — per FR7.4).

---

## Section 2: Settings Store + SettingsDialog

### New store: `src/lib/stores/settings.svelte.ts`

Reactive Svelte 5 store managing all user configuration:

```ts
let language = $state<"en" | "uk">("en");
let confirmExitDuringRecording = $state(true);
let hotkey = $state("Pause");
let soundEnabled = $state(true);
```

`hotkey` and `soundEnabled` move here from `recording.svelte.ts`. The recording store delegates to settings store for these values.

`scheduleSave()` in `App.svelte` reads from both stores when persisting.

### Rust backend changes

**New Settings fields** (`src-tauri/src/settings.rs`):
```rust
pub language: String,               // default: "en"
pub confirm_exit_during_recording: bool,  // default: true
```

**Settings migration:** v2 → v3 adds new fields with defaults. Existing migration loop extended.

**New IPC command: `unregister_hotkey`**  
Temporarily unregisters the global shortcut during hotkey capture mode, so the current hotkey doesn't fire if the user presses it while capturing a new one. After capture, `set_hotkey` registers the new shortcut.

```rust
#[tauri::command]
fn unregister_hotkey(app: tauri::AppHandle) -> Result<(), String> {
    hotkey::unregister(&app)
}
```

### Frontend TypeScript types

`Settings` interface adds:
```ts
language: "en" | "uk";
confirmExitDuringRecording: boolean;
```

### SettingsDialog component: `src/lib/components/SettingsDialog.svelte`

Modal dialog following the `ProfileDialog` pattern. Props: `open: boolean`, `onclose: () => void`.

Layout:
```
┌─────────────── Settings ──────────────────┐
│ Global Hotkey                             │
│  [Pause          ] [Capture New] [Reset]  │
│                                           │
│ Sound notifications        [✓ checkbox]   │
│                                           │
│ Confirm exit during recording [✓ checkbox]│
│                                           │
│ Language        [English / Українська ▼]  │
│  * Change takes effect after restart      │
│                                           │
│                              [Close]      │
└───────────────────────────────────────────┘
```

**Hotkey capture flow:**
1. User clicks "Capture New" → call `unregister_hotkey` IPC → enter capture mode
2. Frontend listens for `keydown` with `capture: true`
3. On key press → build shortcut string → call `set_hotkey` IPC → exit capture mode
4. "Reset" button → call `set_hotkey("Pause")` directly

All settings auto-save on change via `scheduleSave()`.

### App.svelte changes

- Add "⚙ Settings" button → opens `SettingsDialog`
- Remove inline hotkey/sound section from main layout (now in dialog)
- Add `SettingsDialog` and `ConfirmExitDialog` alongside `ProfileDialog`

### ConfirmExitDialog component: `src/lib/components/ConfirmExitDialog.svelte`

Custom HTML modal (not native `confirm()`). Shows when `CloseRequested` Tauri event fires and `confirmExitDuringRecording && isRecording`.

Buttons:
- "Stop and exit" → `stop_recording()` → `appWindow.close()`
- "Cancel" → dismiss dialog, recording continues

Fully accessible: `role="alertdialog"`, focus-trapped, keyboard navigable, localized via Paraglide.

Tauri close event handling in `App.svelte`:
```ts
import { getCurrentWindow } from "@tauri-apps/api/window";
const appWindow = getCurrentWindow();
appWindow.onCloseRequested(async (event) => {
  if (settings.confirmExitDuringRecording && isRecording) {
    event.preventDefault();
    confirmExitOpen = true;
  }
});
```

---

## Section 3: aria-live Regions

The existing `<div aria-live="polite" aria-atomic="true">` in `App.svelte` is updated:

```html
<div role="status" aria-atomic="true" class="visually-hidden">
  {liveRegionText}
</div>
```

`role="status"` carries implicit `aria-live="polite"` (per ARIA spec). Text updated via `$effect` using Paraglide:

```ts
$effect(() => {
  const state = recording.state;
  if (state === "Recording") liveRegionText = m.recording_started();
  else if (state === "Paused")  liveRegionText = m.recording_paused();
  else if (state === "Idle")    liveRegionText = m.recording_stopped();
});
```

Alt+I mnemonic uses `m.status_duration({ state, duration })` for localized status announcement.

No new component needed — the existing div is sufficient.

---

## Section 4: Scoop Manifest

New file: `bucket/audiocaptor.json`

```json
{
  "version": "0.1.0",
  "description": "AudioCaptor — portable audio recording application",
  "homepage": "https://github.com/USERNAME/AudioCaptor",
  "license": "MIT",
  "url": "https://github.com/USERNAME/AudioCaptor/releases/download/v$version/AudioCaptor.exe",
  "hash": "auto:sha256",
  "bin": "AudioCaptor.exe",
  "persist": ["settings.json", "logs", "Recordings"],
  "checkver": { "github": "https://github.com/USERNAME/AudioCaptor" },
  "autoupdate": {
    "url": "https://github.com/USERNAME/AudioCaptor/releases/download/v$version/AudioCaptor.exe"
  }
}
```

**persist behavior:** Scoop creates junction points for listed paths. `portable::exe_dir()` uses `std::env::current_exe()` which resolves through junction points correctly on Windows. No Rust backend changes required.

**Verification needed:** Confirm `std::fs::canonicalize` on the exe path works correctly when installed under `~/scoop/apps/audiocaptor/current/`.

---

## New Files Summary

| File | Purpose |
|------|---------|
| `messages/en.json` | English message strings |
| `messages/uk.json` | Ukrainian message strings |
| `src/lib/i18n.ts` | Language initialization helper |
| `src/lib/stores/settings.svelte.ts` | Settings reactive store |
| `src/lib/components/SettingsDialog.svelte` | Settings modal dialog |
| `src/lib/components/ConfirmExitDialog.svelte` | Exit confirmation dialog |
| `bucket/audiocaptor.json` | Scoop manifest |

## Modified Files Summary

| File | Changes |
|------|---------|
| `vite.config.ts` | Add Paraglide Vite plugin |
| `src-tauri/src/settings.rs` | Add `language`, `confirm_exit_during_recording` fields; v2→v3 migration |
| `src-tauri/src/hotkey.rs` | Add `unregister()` function |
| `src-tauri/src/lib.rs` | Add `unregister_hotkey` IPC command; settings migration v3 |
| `src/lib/types/index.ts` | Add `language`, `confirmExitDuringRecording` to `Settings` |
| `src/lib/stores/recording.svelte.ts` | Remove `hotkey`, `soundEnabled` (moved to settings store) |
| `src/lib/utils/invoke.ts` | Add `unregisterHotkey()` wrapper |
| `src/App.svelte` | Wire up SettingsDialog, ConfirmExitDialog, i18n init, CloseRequested handler |
| All `*.svelte` components | Replace hardcoded strings with Paraglide `m.*()` calls |

---

## Definition of Done

- [ ] UI displays in English by default
- [ ] UI displays in Ukrainian when language set to "Українська"
- [ ] All UI strings localized (no hardcoded text)
- [ ] First run auto-detects system language; `ru` falls back to `uk`
- [ ] Language change persists in `settings.json` and takes effect after restart
- [ ] Settings modal contains all sections: hotkey, sound, confirm-exit, language
- [ ] Hotkey capture works: `unregister_hotkey` → capture → `set_hotkey`
- [ ] Reset hotkey to Pause/Break works
- [ ] Sound toggle works
- [ ] Confirm-exit dialog appears when closing during recording (if enabled)
- [ ] All settings auto-save
- [ ] `role="status"` aria-live region announces recording state changes (localized)
- [ ] Scoop manifest is valid; `persist` preserves `settings.json`, `logs/`, `Recordings/`
- [ ] App works correctly from Scoop directory
- [ ] Settings modal fully keyboard/screen-reader accessible
