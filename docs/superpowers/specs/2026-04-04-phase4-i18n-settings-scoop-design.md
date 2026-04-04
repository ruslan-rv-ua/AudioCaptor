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

- Install `@inlang/paraglide-js` (dev dep) with Vite plugin (`@inlang/paraglide-js/vite`)
- Configure in `vite.config.ts` — works with plain Svelte (no SvelteKit required)
- Create `project.inlang/settings.json` — Paraglide project config declaring source language (`en`) and available locales (`en`, `uk`)
- Paraglide compiles messages to `src/paraglide/` at build time (tree-shakeable, typed)

### Message files

```
messages/
  en.json   ← English (fallback, default)
  uk.json   ← Ukrainian
```

No `ru.json` — prohibited by project requirements. If system language is `ru`, falls back to `uk`.

### Initialization — language applied only once at startup

New file `src/lib/i18n.ts`:
```ts
import { setLanguageTag } from "../paraglide/runtime";
export function initLanguage(lang: "en" | "uk") {
  setLanguageTag(lang);
}
```

**Critical:** `setLanguageTag()` is reactive by design in Paraglide 2.0 — calling it again would immediately re-render all localized strings without a restart. To enforce the "restart required" behavior (FR7.4), `initLanguage()` must be called **only once** — at app startup — and must **never** be called again when the user changes the language dropdown. The restart requirement is enforced by this convention, not by Paraglide itself.

**Timing — preventing flash of wrong language:** `loadSettingsIntoStore()` is async. To avoid a frame where the UI renders in the wrong language before settings resolve, render the app content behind a boolean `initialized` flag that becomes `true` only after settings are loaded and `initLanguage()` is called. During initialization, show a minimal loading state (or nothing, since startup is fast).

```ts
let initialized = $state(false);
onMount(async () => {
  await loadSettingsIntoStore();
  initLanguage(settingsStore.language);
  initialized = true;
  // ... rest of startup
});
```

```html
{#if initialized}
  <!-- full app UI -->
{/if}
```

### Language detection (first run)

```ts
function detectLanguage(): "en" | "uk" {
  const lang = navigator.language.toLowerCase();
  if (lang.startsWith("uk") || lang.startsWith("ru")) return "uk";
  return "en";
}
```

On first run (no `language` in settings / default value), detect and persist. Subsequent runs read saved value.

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

Saving a new language writes to `settings.json`. Change takes effect after app restart. The UI shows a note: "Change takes effect after restart." `setLanguageTag()` is NOT called on language change — only on startup.

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

`hotkey` and `soundEnabled` move here from `recording.svelte.ts`. The recording store no longer holds these fields.

`scheduleSave()` in `App.svelte` reads from both stores when persisting. The `version` field must be read from the settings store (loaded from disk) — never hardcoded. This fixes the existing bug where `App.svelte` hardcodes `version: 2`, which would cause downgrade on every save after migration to v3.

### Rust backend changes

**New Settings fields** (`src-tauri/src/settings.rs`):
```rust
pub language: String,                      // default: "en"
pub confirm_exit_during_recording: bool,   // default: true
```

**Settings migration: v2 → v3**

The existing migration loop is extended. The `v2` arm currently `break`s as the terminal case. To add v3, the `v2` arm must set `version = 3` without a `break` (so the loop re-enters with `3`), and a new `v3` arm breaks:

```rust
2 => {
    // existing logic: ensure at least one profile exists
    if settings.profiles.is_empty() {
        settings.profiles = vec![RecordingProfile::default()];
        settings.active_profile_id = "default".to_string();
    }
    settings.version = 3;
    // no break — loop continues to v3 arm
}
3 => {
    // v3: language and confirm_exit_during_recording added with #[serde(default)]
    // serde fills missing fields from Default automatically — no data transform needed
    break;
}
```

`Settings::default()` must return `version: 3` so fresh installs write v3 immediately and don't trigger migration on every launch.

**Test updates required:** Changing `Default::version` from `0` to `3` will break existing tests that assert `settings.version == 0` (e.g., `settings_v2_has_profiles_and_active_id`). All tests that construct `Settings::default()` and assert on the version field must be updated to expect `3`, not `0`. Tests that explicitly set `settings.version = 0` before testing migration (e.g., `migrate_settings_upgrades_v0_to_v2`) remain valid and are unaffected.

**New IPC command: `unregister_hotkey`**

Temporarily unregisters the global shortcut during hotkey capture mode, so the current hotkey doesn't fire while the user presses a new one. `hotkey::unregister()` is a thin wrapper around the plugin's existing `unregister_all()`:

```rust
pub fn unregister(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn unregister_hotkey(app: tauri::AppHandle) -> Result<(), String> {
    hotkey::unregister(&app)
}
```

**Rust `on_window_event` CloseRequested conflict — MUST BE RESOLVED**

`lib.rs` currently has an `on_window_event` handler that unconditionally calls `stop_recording_inner()` when the window is closing. This handler will silently stop recording before the frontend's `ConfirmExitDialog` can appear, making the confirm-exit feature invisible to the user.

Resolution: **Remove** the existing `on_window_event` handler from `lib.rs`. All close-during-recording logic moves to the frontend. The frontend's `onCloseRequested` handler calls `stop_recording()` IPC explicitly if the user confirms exit (or if `confirmExitDuringRecording` is false). This is safe because the frontend wraps the call in try/catch and ignores "Not recording" errors — `stop_recording_inner` returns `Err("Not recording")` on the Idle state, so the try/catch on the confirm-exit path (see ConfirmExitDialog section) is mandatory.

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

**Hotkey capture flow (moved from App.svelte into SettingsDialog):**
1. User clicks "Capture New" → call `unregister_hotkey` IPC → enter capture mode
2. Frontend listens for `keydown` with `capture: true`
3. On key press (non-modifier) → build shortcut string → call `set_hotkey` IPC → exit capture mode
4. On Escape or dialog close during capture → call `set_hotkey(currentHotkey)` to re-register the previous hotkey, exit capture mode without changing the hotkey
5. "Reset" button → call `set_hotkey("Pause")` directly

All settings auto-save on change. `SettingsDialog` receives an `onsave: (patch: Partial<Settings>) => void` prop — `App.svelte` implements this to merge the patch into the settings store and call `scheduleSave()`. This mirrors the `onsave` pattern of `ProfileDialog`.

### App.svelte changes

- Add `initialized` state flag — render full UI only after settings loaded and language initialized
- Add "⚙ Settings" button → opens `SettingsDialog`
- Remove inline hotkey/sound section from main layout (moved into `SettingsDialog`)
- Remove hotkey capture logic (moved into `SettingsDialog`)
- Add `SettingsDialog` and `ConfirmExitDialog` alongside `ProfileDialog`
- Remove `on_window_event` Rust handler (now handled here in frontend)
- Register `onCloseRequested` in `onMount`

### ConfirmExitDialog component: `src/lib/components/ConfirmExitDialog.svelte`

Custom HTML modal (not native `confirm()`). Shows when `CloseRequested` Tauri event fires and `confirmExitDuringRecording && isRecording`.

Buttons:
- "Stop and exit" → `stop_recording()` → `appWindow.close()`
- "Cancel" → dismiss dialog, recording continues

Fully accessible: `role="alertdialog"`, `aria-labelledby` pointing to dialog title element, `aria-describedby` pointing to dialog body text, focus-trapped, keyboard navigable, localized via Paraglide. Consistent approach with `ProfileDialog` (`aria-label` on backdrop).

Tauri close event handling in `App.svelte`:
```ts
import { getCurrentWindow } from "@tauri-apps/api/window";
const appWindow = getCurrentWindow();

onMount(() => {
  appWindow.onCloseRequested(async (event) => {
    if (settings.confirmExitDuringRecording && isRecording) {
      event.preventDefault();
      confirmExitOpen = true;
    }
    // if confirmExitDuringRecording is false or not recording, window closes normally
  });
});
```

When the user confirms exit in `ConfirmExitDialog`, the handler calls `stop_recording()` then `appWindow.close()`. `stop_recording` returns an error if already Idle (race condition). The frontend must wrap the call in try/catch and ignore "Not recording" errors:
```ts
try { await stopRecording(); } catch { /* already stopped */ }
await appWindow.close();
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
  if (!initialized) return; // don't announce before language is ready
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

The static manifest requires a real SHA256 hash for the initial version. For ongoing releases, `autoupdate` with a hash file URL handles verification automatically.

```json
{
  "version": "0.1.0",
  "description": "AudioCaptor — portable audio recording application",
  "homepage": "https://github.com/USERNAME/AudioCaptor",
  "license": "MIT",
  "url": "https://github.com/USERNAME/AudioCaptor/releases/download/v0.1.0/AudioCaptor.exe",
  "hash": "PLACEHOLDER_SHA256_FILL_ON_RELEASE",
  "bin": "AudioCaptor.exe",
  "persist": ["settings.json", "logs", "Recordings"],
  "checkver": { "github": "https://github.com/USERNAME/AudioCaptor" },
  "autoupdate": {
    "url": "https://github.com/USERNAME/AudioCaptor/releases/download/v$version/AudioCaptor.exe"
  }
}
```

**Hash note:** The `hash` field must be a real SHA256 hex string on release. Use `scoop hash AudioCaptor.exe` to generate it. The `autoupdate` block handles future versions automatically via `checkver`.

**Hash note:** The top-level `hash` field must be a real SHA256 hex string filled at release time. Use `scoop hash AudioCaptor.exe` to generate it. For `autoupdate`, Scoop needs a hash source — either publish a `.sha256` sidecar file alongside the release and reference it, or manage hashes manually per release. Add to `autoupdate` when a sidecar is available:
```json
"autoupdate": {
  "url": "https://github.com/USERNAME/AudioCaptor/releases/download/v$version/AudioCaptor.exe",
  "hash": { "url": "$url.sha256" }
}
```
Until a sidecar is published, `autoupdate.hash` is omitted and hash must be updated manually in the manifest on each release.

**persist behavior:** Scoop creates junction points for `settings.json`, `logs`, `Recordings`. Casing must exactly match `portable::ensure_dirs()` — which creates `logs` (lowercase) and `Recordings` (capital R). `portable::exe_dir()` uses `std::env::current_exe()` which resolves through junction points correctly on Windows.

**Verification needed:** Confirm behavior under `~/scoop/apps/audiocaptor/current/` with junction-linked `settings.json`.

---

## New Files Summary

| File | Purpose |
|------|---------|
| `messages/en.json` | English message strings |
| `messages/uk.json` | Ukrainian message strings |
| `project.inlang/settings.json` | Paraglide project config (source lang, locales) |
| `src/lib/i18n.ts` | Language initialization helper (call only once at startup) |
| `src/lib/stores/settings.svelte.ts` | Settings reactive store |
| `src/lib/components/SettingsDialog.svelte` | Settings modal dialog |
| `src/lib/components/ConfirmExitDialog.svelte` | Exit confirmation dialog |
| `bucket/audiocaptor.json` | Scoop manifest |

## Modified Files Summary

| File | Changes |
|------|---------|
| `package.json` | Add `@inlang/paraglide-js` dev dependency |
| `vite.config.ts` | Add Paraglide Vite plugin |
| `src-tauri/src/settings.rs` | Add `language`, `confirm_exit_during_recording` fields; v2→v3 migration; `Settings::default()` returns version 3 |
| `src-tauri/src/hotkey.rs` | Add public `unregister()` function (thin wrapper over `unregister_all()`) |
| `src-tauri/src/lib.rs` | Add `unregister_hotkey` IPC command; remove `on_window_event` CloseRequested handler; register new command in invoke_handler |
| `src/lib/types/index.ts` | Add `language`, `confirmExitDuringRecording` to `Settings` interface |
| `src/lib/stores/recording.svelte.ts` | Remove `hotkey`, `soundEnabled` (moved to settings store) |
| `src/lib/utils/invoke.ts` | Add `unregisterHotkey()` wrapper |
| `src/App.svelte` | Add `initialized` flag; wire up SettingsDialog, ConfirmExitDialog; move hotkey capture to SettingsDialog; i18n init; onCloseRequested handler; fix `scheduleSave` to read version from store not hardcode 2 |
| All `*.svelte` components | Replace hardcoded strings with Paraglide `m.*()` calls |

---

## Definition of Done

- [ ] UI displays in English by default
- [ ] UI displays in Ukrainian when language set to "Українська"
- [ ] All UI strings localized (no hardcoded text)
- [ ] First run auto-detects system language; `ru` falls back to `uk`
- [ ] Language change persists in `settings.json` and takes effect after restart
- [ ] No flash of wrong language on startup (`initialized` guard)
- [ ] Settings modal contains all sections: hotkey, sound, confirm-exit, language
- [ ] Hotkey capture works: `unregister_hotkey` → capture → `set_hotkey`
- [ ] Reset hotkey to Pause/Break works
- [ ] Sound toggle works
- [ ] Confirm-exit dialog appears when closing during recording (if enabled)
- [ ] Closing without recording active works normally (no dialog)
- [ ] All settings auto-save; `scheduleSave` writes correct version (3)
- [ ] `role="status"` aria-live region announces recording state changes (localized)
- [ ] Scoop manifest is valid; `persist` preserves `settings.json`, `logs/`, `Recordings/`
- [ ] App works correctly from Scoop directory
- [ ] Settings modal fully keyboard/screen-reader accessible
- [ ] `ConfirmExitDialog` has `aria-labelledby` and `aria-describedby`
