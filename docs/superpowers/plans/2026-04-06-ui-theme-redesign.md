# UI Theme Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all hardcoded colors with a CSS custom-property token system, add dark/light/auto theme with Windows detection, update all 9 UI components to use the new design language, and restructure ProfileDialog to fit without scrollbars.

**Architecture:** CSS custom properties on `:root` (light) + `[data-theme="dark"]` (dark override). A new `theme.svelte.ts` store manages the `data-theme` attribute and `matchMedia` listener. Theme preference is mirrored to `localStorage` for synchronous FOUC-prevention via an inline script in `index.html`. All component styles reference only CSS vars — zero hardcoded hex values remain after this plan.

**Tech Stack:** Svelte 5 (runes), TypeScript, Tauri v2 WebView2, paraglide-js i18n, pnpm. Verification: `pnpm check` (svelte-check + tsc). Visual verification: `pnpm vite:dev`.

---

## File Map

| File | Action | Purpose |
|---|---|---|
| `messages/en.json` | Modify | Add theme + profile section i18n keys |
| `messages/uk.json` | Modify | Ukrainian translations for same |
| `src/app.css` | Modify | All CSS custom property tokens; global slider thumb styles; body/html base |
| `src/lib/types/index.ts` | Modify | Add `Theme` type; add `theme` field to `Settings` |
| `src/lib/stores/theme.svelte.ts` | Create | Theme store: preference state, DOM application, matchMedia listener |
| `src/lib/stores/settings.svelte.ts` | Modify | Add `theme` field + `setThemePreference()`; bump `settingsVersion` to 4 |
| `src-tauri/src/settings.rs` | Modify | Add `theme` field; bump version 3→4; migration arm; tests |
| `index.html` | Modify | Inline FOUC-prevention script in `<head>` |
| `src/App.svelte` | Modify | Wire theme store; add section card styles + sec labels; fix header; add theme to save |
| `src/lib/components/DeviceSelect.svelte` | Modify | CSS vars only |
| `src/lib/components/ProfileSelector.svelte` | Modify | CSS vars + card + sec label |
| `src/lib/components/RecordControls.svelte` | Modify | CSS vars only |
| `src/lib/components/ConfirmExitDialog.svelte` | Modify | CSS vars only |
| `src/lib/components/StatusIndicator.svelte` | Modify | State-dependent border; dim timer; blinking dot |
| `src/lib/components/VolumeSlider.svelte` | Modify | Custom thumb/track with fill gradient |
| `src/lib/components/SettingsDialog.svelte` | Modify | Add `theme` prop + segmented control; CSS vars |
| `src/lib/components/ProfileDialog.svelte` | Modify | 2-column grid; remove scroll; section labels; conditional filenames |

---

## Task 1 — i18n: Add new message keys

**Files:**
- Modify: `messages/en.json`
- Modify: `messages/uk.json`

Add keys needed by the new SettingsDialog theme control and ProfileDialog section labels. All other tasks depend on these keys compiling cleanly.

- [ ] **Step 1: Add keys to `messages/en.json`**

Add after `"settings_language_restart_note"`:

```json
  "settings_theme_label": "Theme",
  "settings_theme_auto": "Auto (Windows)",
  "settings_theme_light": "Light",
  "settings_theme_dark": "Dark",
  "settings_theme_hint": "Auto follows the Windows color setting",
  "profile_section_basic": "Basic",
  "profile_section_audio": "Audio",
  "profile_section_filenames": "File names",
```

- [ ] **Step 2: Add keys to `messages/uk.json`**

Add after `"settings_language_restart_note"`:

```json
  "settings_theme_label": "Тема",
  "settings_theme_auto": "Авто (Windows)",
  "settings_theme_light": "Світла",
  "settings_theme_dark": "Темна",
  "settings_theme_hint": "Авто слідує налаштуванню кольору Windows",
  "profile_section_basic": "Основне",
  "profile_section_audio": "Аудіо",
  "profile_section_filenames": "Імена файлів",
```

- [ ] **Step 3: Verify check passes**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Commit**

```bash
git add messages/en.json messages/uk.json
git commit -m "feat(i18n): add theme picker and profile section message keys"
```

---

## Task 2 — CSS Tokens + TypeScript Types + Theme Store

**Files:**
- Modify: `src/app.css`
- Modify: `src/lib/types/index.ts`
- Create: `src/lib/stores/theme.svelte.ts`
- Modify: `src/lib/stores/settings.svelte.ts`

This is the foundation. Everything else depends on these tokens existing.

- [ ] **Step 1: Replace `src/app.css` entirely**

```css
/* ── Base reset ─────────────────────────────────────────────────── */
*,
*::before,
*::after {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
  min-height: 100vh;
  font-family: 'Segoe UI', system-ui, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  background-color: var(--bg);
  color: var(--text-primary);
}

/* ── Light theme tokens (default) ───────────────────────────────── */
:root {
  --bg:                   #f0f0f0;
  --surface:              #ffffff;
  --surface-hover:        #f5f5f5;
  --border:               rgba(0, 0, 0, 0.12);
  --track:                #e5e7eb;
  --text-primary:         #111111;
  --text-secondary:       #555555;
  --text-muted:           #6b7280;
  --accent:               #0078d4;
  --accent-hover:         #106ebe;
  --sec-label:            #0078d4;
  --focus-ring:           #0078d4;
  --btn-start-bg:         #107c10;
  --btn-pause-bg:         #9d5d00;
  --btn-stop-bg:          #c42b1c;
  --btn-resume-bg:        #0078d4;
  --status-rec-border:    #dc2626;
  --status-paused-border: #d97706;
  --error-bg:             #fef2f2;
  --error-border:         #fecaca;
  --error-text:           #dc2626;
}

/* ── Dark theme overrides ───────────────────────────────────────── */
[data-theme="dark"] {
  --bg:                   #1c1c1c;
  --surface:              #252525;
  --surface-hover:        #2e2e2e;
  --border:               rgba(255, 255, 255, 0.08);
  --track:                #3a3a3a;
  --text-primary:         #f0f0f0;
  --text-secondary:       #888888;
  --text-muted:           #6b7280;
  --accent:               #60a5fa;
  --accent-hover:         #93c5fd;
  --sec-label:            #60a5fa;
  --focus-ring:           #60a5fa;
  --btn-start-bg:         #0f7b0f;
  --btn-pause-bg:         #92400e;
  --btn-stop-bg:          #991b1b;
  --btn-resume-bg:        #1d4ed8;
  --status-rec-border:    #f87171;
  --status-paused-border: #fbbf24;
  --error-bg:             rgba(220, 38, 38, 0.1);
  --error-border:         rgba(248, 113, 113, 0.4);
  --error-text:           #f87171;
}

/* ── Accessibility ──────────────────────────────────────────────── */
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
}

:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

/* ── Range input: cross-browser custom thumb + filled track ─────── */
/*
 * The fill percentage is set as --fill on the <input> via Svelte style binding.
 * CSS custom properties are inherited by pseudo-elements in Chromium/WebView2.
 */
input[type="range"] {
  -webkit-appearance: none;
  appearance: none;
  height: 20px;        /* hit area — comfortable without being excessive */
  background: transparent;
  cursor: pointer;
  width: 100%;
  padding: 0;
  margin: 0;
}

input[type="range"]::-webkit-slider-runnable-track {
  height: 5px;
  border-radius: 3px;
  background: linear-gradient(
    to right,
    var(--accent)  0%,
    var(--accent)  var(--fill, 50%),
    var(--track)   var(--fill, 50%),
    var(--track)   100%
  );
}

input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--surface);
  border: 2px solid var(--accent);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
  margin-top: -4.5px; /* centre on 5px track: (14 - 5) / 2 = 4.5 */
  cursor: pointer;
  transition: border-color 0.1s;
}

input[type="range"]::-webkit-slider-thumb:hover {
  border-color: var(--accent-hover);
}

input[type="range"]:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 3px;
}

input[type="range"]:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

- [ ] **Step 2: Update `src/lib/types/index.ts`**

Add `Theme` type and `theme` field to `Settings`:

```typescript
export interface AudioDevice {
  id: string;
  name: string;
  isInput: boolean;
}

export type RecordingState = "Idle" | "Recording" | "Paused";

export type OutputMode = "Microphone" | "Loopback" | "Mix" | "MixPlusMicrophone" | "MixPlusLoopback";

export type Theme = "auto" | "light" | "dark";

export interface RecordingProfile {
  id: string;
  name: string;
  description: string;
  outputFolder: string;
  outputMode: OutputMode;
  sampleRate: number;
  micVolume: number;
  loopbackVolume: number;
  micFilename: string;
  loopbackFilename: string;
  mixFilename: string;
}

export interface Settings {
  version: number;
  selectedMic: string | null;
  selectedLoopback: string | null;
  hotkey: string;
  soundEnabled: boolean;
  profiles: RecordingProfile[];
  activeProfileId: string;
  language: "en" | "uk";
  confirmExitDuringRecording: boolean;
  theme: Theme;
}

export interface RecordingStateEvent {
  state: RecordingState;
  durationMs: number;
}
```

- [ ] **Step 3: Create `src/lib/stores/theme.svelte.ts`**

```typescript
import type { Theme } from "../types";

const STORAGE_KEY = "audiocaptor_theme";

let preference = $state<Theme>("auto");
let mediaQuery: MediaQueryList | null = null;

function resolve(pref: Theme): "light" | "dark" {
  if (pref === "auto") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  return pref;
}

function applyToDOM(t: "light" | "dark") {
  document.documentElement.setAttribute("data-theme", t);
}

function onMediaChange(e: MediaQueryListEvent) {
  if (preference === "auto") applyToDOM(e.matches ? "dark" : "light");
}

export function getTheme() {
  return {
    get preference() { return preference; },
  };
}

/** Call once in App.svelte onMount, after settings are loaded. */
export function initTheme(saved: Theme) {
  preference = saved;
  applyToDOM(resolve(saved));
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener("change", onMediaChange);
}

/** Call when the user changes the theme in Settings. */
export function setTheme(t: Theme) {
  preference = t;
  localStorage.setItem(STORAGE_KEY, t);
  applyToDOM(resolve(t));
}

/** Call in App.svelte onMount cleanup. */
export function cleanupTheme() {
  mediaQuery?.removeEventListener("change", onMediaChange);
}
```

- [ ] **Step 4: Update `src/lib/stores/settings.svelte.ts`**

Replace the entire file:

```typescript
import type { Settings, Theme } from "../types";
import * as api from "../utils/invoke";

let language = $state<"en" | "uk">("en");
let confirmExitDuringRecording = $state(true);
let hotkey = $state("Pause");
let soundEnabled = $state(true);
let settingsVersion = $state(3);
let theme = $state<Theme>("auto");

export function getSettings() {
  return {
    get language() { return language; },
    get confirmExitDuringRecording() { return confirmExitDuringRecording; },
    get hotkey() { return hotkey; },
    get soundEnabled() { return soundEnabled; },
    get version() { return settingsVersion; },
    get theme() { return theme; },
  };
}

export function loadSettingsFields(s: Settings) {
  language = (s.language as "en" | "uk") ?? "en";
  confirmExitDuringRecording = s.confirmExitDuringRecording ?? true;
  hotkey = s.hotkey;
  soundEnabled = s.soundEnabled;
  settingsVersion = s.version;
  theme = s.theme ?? "auto";
}

export async function updateHotkey(shortcut: string) {
  hotkey = shortcut;
  await api.setHotkey(shortcut);
}

export async function updateSoundEnabled(enabled: boolean) {
  soundEnabled = enabled;
  await api.setSoundEnabled(enabled);
}

export function setLanguage(lang: "en" | "uk") {
  language = lang;
}

export function setConfirmExitDuringRecording(value: boolean) {
  confirmExitDuringRecording = value;
}

export function setThemePreference(t: Theme) {
  theme = t;
}
```

- [ ] **Step 5: Update `src-tauri/src/settings.rs`**

5a. Add `default_theme` function after `default_sample_rate`:
```rust
fn default_theme() -> String {
    "auto".to_string()
}
```

5b. Add `theme` field after `confirm_exit_during_recording` in the struct:
```rust
#[serde(default = "default_theme")]
pub theme: String,              // NEW in v4 — "auto" | "light" | "dark"
```

5c. In `Default::default()`: change `version: 3` → `version: 4`, add `theme: "auto".to_string()`.

5d. In `migrate_settings` loop, replace the `3 =>` arm:
```rust
3 => {
    // theme added in v4; serde Default fills "auto" for existing files
    settings.version = 4;
    // fall through to v4 arm
}
4 => {
    break;
}
```

5e. Update tests:
- Rename `settings_default_has_version_3` → `settings_default_has_version_4`, expect `version == 4` and `theme == "auto"`
- Add `migrate_v3_to_v4_adds_theme`: deserialize v3 JSON (no `theme` key), migrate, assert `version == 4` and `theme == "auto"`

- [ ] **Step 6: Update `settingsVersion` in `src/lib/stores/settings.svelte.ts`**

Line 8: `let settingsVersion = $state(3)` → `$state(4)`

- [ ] **Step 7: Verify**

```bash
cargo test -p audiocaptor-lib
pnpm check
```

Expected: all Rust tests pass, 0 svelte-check errors.

- [ ] **Step 8: Commit**

```bash
git add src/app.css src/lib/types/index.ts src/lib/stores/theme.svelte.ts src/lib/stores/settings.svelte.ts src-tauri/src/settings.rs
git commit -m "feat(theme): add CSS tokens, Theme type, theme store, and settings v4 migration"
```

---

## Task 3 — FOUC Prevention + App.svelte Integration

**Files:**
- Modify: `index.html`
- Modify: `src/App.svelte`

Wire the theme store into the app lifecycle. The FOUC script runs before Svelte so the window never flashes white.

- [ ] **Step 1: Add FOUC script to `index.html`**

Replace `<head>` content:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/png" href="/favicon.png" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>AudioCaptor</title>
    <script>
      // Runs synchronously before first paint — prevents theme flash.
      // Reads the localStorage mirror written by the theme store.
      (function () {
        var t = localStorage.getItem("audiocaptor_theme") || "auto";
        if (t === "auto") {
          t = window.matchMedia("(prefers-color-scheme: dark)").matches
            ? "dark"
            : "light";
        }
        document.documentElement.setAttribute("data-theme", t);
      })();
    </script>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 2: Update `src/App.svelte` — add theme store wiring**

Add imports at the top of the `<script lang="ts">` block (after existing imports):

```typescript
import {
  initTheme,
  setTheme,
  cleanupTheme,
} from "./lib/stores/theme.svelte";
import { setThemePreference } from "./lib/stores/settings.svelte";
import type { Theme } from "./lib/types";
```

Update `scheduleSave` to include `theme`:

```typescript
function scheduleSave() {
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(async () => {
    await saveSettings({
      version: appSettings.version,
      selectedMic: recording.selectedMic,
      selectedLoopback: recording.selectedLoopback,
      hotkey: appSettings.hotkey,
      soundEnabled: appSettings.soundEnabled,
      language: appSettings.language,
      confirmExitDuringRecording: appSettings.confirmExitDuringRecording,
      profiles: profileStore.list,
      activeProfileId: profileStore.activeId,
      theme: appSettings.theme,
    });
  }, 500);
}
```

Add `handleThemeChange` function:

```typescript
function handleThemeChange(t: Theme) {
  setTheme(t);             // applies to DOM + writes localStorage
  setThemePreference(t);   // updates settings store reactive state
  scheduleSave();          // persists to Tauri backend
}
```

In `onMount`, inside the async IIFE, add `initTheme` call right after `loadSettingsFields`:

```typescript
onMount(() => {
  (async () => {
    const rawSettings = await loadSettings();
    loadSettingsFields(rawSettings);
    initTheme(appSettings.theme);   // ← add this line
    initLanguage(appSettings.language);
    // ... rest unchanged ...
  })();
  window.addEventListener("keydown", handleMnemonic);
  return () => {
    window.removeEventListener("keydown", handleMnemonic);
    cleanupTheme();   // ← add this line
  };
});
```

Update `<SettingsDialog>` to pass `theme` and `onthemechange`:

```svelte
<SettingsDialog
  open={settingsOpen}
  hotkey={appSettings.hotkey}
  soundEnabled={appSettings.soundEnabled}
  confirmExitDuringRecording={appSettings.confirmExitDuringRecording}
  language={appSettings.language}
  theme={appSettings.theme}
  onclose={() => settingsOpen = false}
  onsave={handleSettingsSave}
  onthemechange={handleThemeChange}
/>
```

- [ ] **Step 3: Update `<style>` block in `src/App.svelte`**

Replace the entire `<style>` block:

```svelte
<style>
  main {
    max-width: 480px;
    margin: 0 auto;
    padding: 20px 16px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  .header-row h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .btn-settings {
    width: 32px;
    height: 32px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 7px;
    cursor: pointer;
    font-size: 17px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
    transition: background 0.1s;
  }

  .btn-settings:hover {
    background: var(--surface-hover);
  }

  /* Section cards — Profile, Devices, Volume */
  .section-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px 11px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  }

  [data-theme="dark"] .section-card {
    box-shadow: none;
  }

  .sec-label {
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.09em;
    color: var(--sec-label);
    margin: 0 0 8px;
  }

  .error {
    padding: 10px 12px;
    background: var(--error-bg);
    border: 1px solid var(--error-border);
    border-radius: 6px;
    color: var(--error-text);
    text-align: center;
    font-size: 0.875rem;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
```

- [ ] **Step 4: Update the `<main>` template in `src/App.svelte`**

Replace the `{#if initialized}` block with:

```svelte
{#if initialized}
<main role="application" aria-label="AudioCaptor">
  <div class="header-row">
    <h1>{m.app_title()}</h1>
    <button
      type="button"
      class="btn-settings"
      aria-label={m.settings_btn_aria()}
      onclick={() => settingsOpen = true}
    >⚙</button>
  </div>

  <StatusIndicator state={recording.state} durationMs={recording.durationMs} />

  <div class="section-card">
    <h2 class="sec-label">{m.profile_label()}</h2>
    <ProfileSelector
      profiles={profileStore.list}
      activeId={profileStore.activeId}
      disabled={isRecording}
      onselect={handleProfileSelect}
      oncreate={handleProfileCreate}
      onedit={handleProfileEdit}
      ondelete={handleProfileDelete}
    />
  </div>

  <div class="section-card" aria-label={m.audio_devices_section()}>
    <h2 class="sec-label">{m.audio_devices_section()}</h2>
    <DeviceSelect
      label={m.mic_label()}
      devices={devices.microphones}
      value={recording.selectedMic}
      onchange={(id) => { recording.selectedMic = id; scheduleSave(); }}
      disabled={isRecording}
    />
    <DeviceSelect
      label={m.loopback_label()}
      devices={devices.loopbacks}
      value={recording.selectedLoopback}
      onchange={(id) => { recording.selectedLoopback = id; scheduleSave(); }}
      disabled={isRecording}
    />
  </div>

  <div class="section-card" aria-label={m.volume_controls_section()}>
    <h2 class="sec-label">{m.volume_controls_section()}</h2>
    <VolumeSlider
      label={m.mic_volume_label()}
      value={recording.micVolume}
      onchange={(v) => { updateMicVolume(v); scheduleSave(); }}
    />
    <VolumeSlider
      label={m.loopback_volume_label()}
      value={recording.loopbackVolume}
      onchange={(v) => { updateLoopbackVolume(v); scheduleSave(); }}
    />
  </div>

  <RecordControls
    recordingState={recording.state}
    canRecord={recording.canRecord}
    needsMic={recording.needsMic}
    needsLoopback={recording.needsLoopback}
    onstart={startRecording}
    onpause={pauseRecording}
    onresume={resumeRecording}
    onstop={stopRecording}
  />

  {#if recording.error}
    <div class="error" role="alert" aria-live="assertive">
      {recording.error}
    </div>
  {/if}

  <div role="status" aria-atomic="true" class="visually-hidden">{liveRegionText}</div>

  <SettingsDialog
    open={settingsOpen}
    hotkey={appSettings.hotkey}
    soundEnabled={appSettings.soundEnabled}
    confirmExitDuringRecording={appSettings.confirmExitDuringRecording}
    language={appSettings.language}
    theme={appSettings.theme}
    onclose={() => settingsOpen = false}
    onsave={handleSettingsSave}
    onthemechange={handleThemeChange}
  />

  <ConfirmExitDialog
    open={confirmExitOpen}
    onstopandexit={handleStopAndExit}
    oncancel={() => confirmExitOpen = false}
  />
</main>
{/if}
```

Note: ProfileSelector now renders inside a `.section-card` div in App.svelte, so **remove** the wrapping `<section>` from inside `ProfileSelector.svelte` itself — it now just renders its inner content (profile-row). See Task 5.

- [ ] **Step 5: Verify check passes**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings. (SettingsDialog will have a type error for `theme`/`onthemechange` props until Task 7 — do that task next if this check fails, or stub the props temporarily.)

> **Note:** If check reports errors about `theme` prop on `SettingsDialog`, add temporary stub props to the interface in `SettingsDialog.svelte`:
> ```typescript
> theme?: Theme;
> onthemechange?: (t: Theme) => void;
> ```
> **Remove the `?` at the very start of Task 7** — do not leave them as optional permanently.

- [ ] **Step 6: Commit**

```bash
git add index.html src/App.svelte
git commit -m "feat(theme): wire theme store into app lifecycle and update layout"
```

---

## Task 4 — Simple Component CSS-Var Migration

**Files:**
- Modify: `src/lib/components/DeviceSelect.svelte`
- Modify: `src/lib/components/ProfileSelector.svelte`
- Modify: `src/lib/components/RecordControls.svelte`
- Modify: `src/lib/components/ConfirmExitDialog.svelte`

Pure mechanical replacement of hardcoded hex colors with CSS custom properties. No logic changes.

- [ ] **Step 1: Replace `<style>` in `DeviceSelect.svelte`**

```svelte
<style>
  .device-select {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label {
    font-weight: 600;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  select {
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 5px;
    font-size: 0.875rem;
    background: var(--surface);
    color: var(--text-primary);
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23888'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 24px;
  }
  select:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

- [ ] **Step 2: Rewrite `ProfileSelector.svelte` — remove wrapping `<section>`, use CSS vars**

The wrapping `<section>` moves to App.svelte (Task 3). ProfileSelector now renders only the profile-row content.

Replace the entire component:

```svelte
<script lang="ts">
  import type { RecordingProfile } from "../types";
  import * as m from "../../paraglide/messages";

  interface Props {
    profiles: RecordingProfile[];
    activeId: string;
    disabled?: boolean;
    onselect: (id: string) => void;
    oncreate: () => void;
    onedit: (profile: RecordingProfile) => void;
    ondelete: (id: string) => void;
  }

  let { profiles, activeId, disabled = false, onselect, oncreate, onedit, ondelete }: Props = $props();

  let activeProfile = $derived(profiles.find(p => p.id === activeId));
  let canDelete = $derived(profiles.length > 1);

  function handleChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    onselect(target.value);
  }
</script>

<div class="profile-controls">
  <select
    aria-label={m.profile_section()}
    {disabled}
    onchange={handleChange}
  >
    {#each profiles as profile}
      <option value={profile.id} selected={profile.id === activeId}>
        {profile.name}
      </option>
    {/each}
  </select>
  <button
    type="button"
    class="btn-icon"
    aria-label={m.create_profile_aria()}
    {disabled}
    onclick={oncreate}
  >+</button>
  <button
    type="button"
    class="btn-icon"
    aria-label={m.edit_profile_aria()}
    {disabled}
    onclick={() => activeProfile && onedit(activeProfile)}
  >✎</button>
  <button
    type="button"
    class="btn-icon btn-danger"
    aria-label={m.delete_profile_aria()}
    disabled={disabled || !canDelete}
    onclick={() => ondelete(activeId)}
  >✕</button>
</div>

<style>
  .profile-controls {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .profile-controls select {
    flex: 1;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 5px;
    font-size: 0.875rem;
    background: var(--surface);
    color: var(--text-primary);
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23888'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 24px;
  }
  .profile-controls select:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
  .btn-icon {
    width: 30px;
    height: 30px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .btn-icon:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .btn-icon:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .btn-danger:hover:not(:disabled) {
    color: var(--error-text);
    border-color: var(--error-border);
  }
</style>
```

- [ ] **Step 3: Replace `<style>` in `RecordControls.svelte`**

```svelte
<style>
  .record-controls {
    display: flex;
    gap: 8px;
    justify-content: center;
    flex-wrap: wrap;
    padding: 8px 0 4px;
  }
  .btn {
    flex: 1;
    padding: 11px 20px;
    border: none;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 700;
    cursor: pointer;
    min-width: 90px;
    color: #ffffff;
  }
  .btn:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
  .btn-start { background: var(--btn-start-bg); }
  .btn-start:hover:not(.disabled) { filter: brightness(1.1); }
  .btn-start.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .hint {
    width: 100%;
    text-align: center;
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 2px 0 0;
  }
  .btn-pause  { background: var(--btn-pause-bg); }
  .btn-pause:hover  { filter: brightness(1.1); }
  .btn-resume { background: var(--btn-resume-bg); }
  .btn-resume:hover { filter: brightness(1.1); }
  .btn-stop   { background: var(--btn-stop-bg); }
  .btn-stop:hover   { filter: brightness(1.1); }
</style>
```

- [ ] **Step 4: Replace `<style>` in `ConfirmExitDialog.svelte`**

```svelte
<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 22px 24px 20px;
    width: 340px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: var(--text-primary);
  }
  h2 { margin: 0 0 8px; font-size: 1.05rem; color: var(--text-primary); }
  p  { margin: 0 0 20px; font-size: 0.9rem; color: var(--text-secondary); }
  .actions { display: flex; gap: 8px; justify-content: flex-end; }
  .btn-secondary {
    padding: 8px 16px; border: 1px solid var(--border); border-radius: 6px;
    font-size: 0.875rem; font-weight: 600; cursor: pointer;
    background: var(--surface); color: var(--text-primary);
  }
  .btn-secondary:hover { background: var(--surface-hover); }
  .btn-danger {
    padding: 8px 16px; border: none; border-radius: 6px;
    font-size: 0.875rem; font-weight: 600; cursor: pointer;
    background: var(--btn-stop-bg); color: #ffffff;
  }
  .btn-danger:hover { filter: brightness(1.1); }
</style>
```

- [ ] **Step 5: Verify check passes**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/DeviceSelect.svelte src/lib/components/ProfileSelector.svelte src/lib/components/RecordControls.svelte src/lib/components/ConfirmExitDialog.svelte
git commit -m "refactor(ui): migrate DeviceSelect, ProfileSelector, RecordControls, ConfirmExitDialog to CSS vars"
```

---

## Task 5 — StatusIndicator: State Border + Timer + Blinking Dot

**Files:**
- Modify: `src/lib/components/StatusIndicator.svelte`

- [ ] **Step 1: Rewrite `StatusIndicator.svelte`**

```svelte
<script lang="ts">
  import type { RecordingState } from "../types";
  import * as m from "../../paraglide/messages";

  interface Props {
    state: RecordingState;
    durationMs: number;
  }

  let { state, durationMs }: Props = $props();

  let formattedDuration = $derived(formatDuration(durationMs));

  function formatDuration(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  }

  let stateLabel = $derived(
    state === "Idle"      ? m.status_ready()     :
    state === "Recording" ? m.status_recording() : m.status_paused()
  );
</script>

<div
  class="status-card"
  class:recording={state === "Recording"}
  class:paused={state === "Paused"}
  aria-label={m.recording_status()}
>
  <div class="state-row">
    {#if state === "Recording"}
      <span class="rec-dot" aria-hidden="true"></span>
    {/if}
    <span class="state-label">{stateLabel}</span>
  </div>
  <span
    class="timer"
    class:timer-idle={state === "Idle"}
    role="timer"
    aria-label={m.recording_duration({ duration: formattedDuration })}
  >{formattedDuration}</span>
</div>

<style>
  .status-card {
    background: var(--surface);
    border: 2px solid transparent;
    border-radius: 10px;
    padding: 10px 14px 12px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
    transition: border-color 0.2s;
  }

  [data-theme="dark"] .status-card {
    box-shadow: none;
  }

  .status-card.recording {
    border-color: var(--status-rec-border);
    box-shadow: 0 0 0 3px rgba(220, 38, 38, 0.08);
  }

  [data-theme="dark"] .status-card.recording {
    box-shadow: 0 0 0 3px rgba(248, 113, 113, 0.08);
  }

  .status-card.paused {
    border-color: var(--status-paused-border);
    box-shadow: 0 0 0 3px rgba(217, 119, 6, 0.08);
  }

  [data-theme="dark"] .status-card.paused {
    box-shadow: 0 0 0 3px rgba(251, 191, 36, 0.08);
  }

  .state-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .state-label {
    font-size: 8.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-muted);
  }

  .status-card.recording .state-label {
    color: var(--status-rec-border);
  }

  .status-card.paused .state-label {
    color: var(--status-paused-border);
  }

  /* Blinking recording dot */
  .rec-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--status-rec-border);
    animation: blink 1.2s ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.3; }
  }

  .timer {
    font-family: 'Courier New', monospace;
    font-size: 26px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .timer.timer-idle {
    color: var(--text-muted);   /* dimmed when stopped */
  }
</style>
```

- [ ] **Step 2: Verify check passes**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/StatusIndicator.svelte
git commit -m "feat(ui): state-dependent border and blinking dot on StatusIndicator"
```

---

## Task 6 — VolumeSlider: Custom Thumb + Fill Track

**Files:**
- Modify: `src/lib/components/VolumeSlider.svelte`

The global CSS in `app.css` (Task 2) already defines `::-webkit-slider-thumb` and `::-webkit-slider-runnable-track`. This task adds the `--fill` CSS variable binding and removes the old inline slider styles.

- [ ] **Step 1: Rewrite `VolumeSlider.svelte`**

```svelte
<script lang="ts">
  interface Props {
    label: string;
    value: number;
    onchange: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
  }

  let {
    label,
    value,
    onchange,
    min = 0,
    max = 4.0,
    step = 0.1,
    disabled = false,
  }: Props = $props();

  // Percentage for the filled-track gradient defined in app.css
  let fillPct = $derived(((value - min) / (max - min)) * 100);

  let inputId = $derived(label.toLowerCase().replace(/\s/g, "-"));

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onchange(parseFloat(target.value));
  }
</script>

<div class="volume-slider">
  <label for={inputId}>
    {label}
    <span class="value-display">{value.toFixed(1)}</span>
  </label>
  <input
    id={inputId}
    type="range"
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value}
    {min}
    {max}
    {step}
    {value}
    {disabled}
    style="--fill: {fillPct}%"
    oninput={handleInput}
  />
</div>

<style>
  .volume-slider {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  label {
    font-weight: 600;
    font-size: 0.8rem;
    color: var(--text-secondary);
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .value-display {
    font-weight: 700;
    color: var(--accent);
    font-size: 0.8rem;
    min-width: 2.5ch;
    text-align: right;
  }
  /* input[type="range"] appearance is handled globally in app.css */
</style>
```

- [ ] **Step 2: Verify check passes**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/VolumeSlider.svelte
git commit -m "feat(ui): custom thumb and fill-gradient track for VolumeSlider"
```

---

## Task 7 — SettingsDialog: Theme Segmented Control

**Files:**
- Modify: `src/lib/components/SettingsDialog.svelte`

Add `theme` prop, `onthemechange` callback, and the segmented control UI. Replace all hardcoded colors with CSS vars.

- [ ] **Step 1: Rewrite `SettingsDialog.svelte`**

```svelte
<script lang="ts">
  import type { Theme } from "../types";
  import * as m from "../../paraglide/messages";
  import { updateHotkey, updateSoundEnabled, setLanguage, setConfirmExitDuringRecording } from "../stores/settings.svelte";
  import * as api from "../utils/invoke";

  interface Props {
    open: boolean;
    hotkey: string;
    soundEnabled: boolean;
    confirmExitDuringRecording: boolean;
    language: "en" | "uk";
    theme: Theme;
    onclose: () => void;
    onsave: (patch: Partial<import("../types").Settings>) => void;
    onthemechange: (t: Theme) => void;
  }

  let {
    open,
    hotkey,
    soundEnabled,
    confirmExitDuringRecording,
    language,
    theme,
    onclose,
    onsave,
    onthemechange,
  }: Props = $props();

  const themeOptions: { value: Theme; label: () => string }[] = [
    { value: "auto",  label: () => m.settings_theme_auto() },
    { value: "light", label: () => m.settings_theme_light() },
    { value: "dark",  label: () => m.settings_theme_dark() },
  ];

  let capturingHotkey = $state(false);
  let localHotkey = $state("");
  $effect.pre(() => { localHotkey = hotkey; });

  async function startHotkeyCapture() {
    capturingHotkey = true;
    await api.unregisterHotkey();

    async function onKeyDown(e: KeyboardEvent) {
      e.preventDefault();
      e.stopPropagation();
      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

      const parts: string[] = [];
      if (e.ctrlKey)  parts.push("Ctrl");
      if (e.altKey)   parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      let key = e.key;
      if (key === " ")         key = "Space";
      else if (key.length === 1) key = key.toUpperCase();
      else if (key === "Escape") {
        window.removeEventListener("keydown", onKeyDown, true);
        await cancelCapture();
        return;
      }
      parts.push(key);
      const shortcut = parts.join("+");
      localHotkey = shortcut;
      await updateHotkey(shortcut);
      onsave({ hotkey: shortcut });
      capturingHotkey = false;
      window.removeEventListener("keydown", onKeyDown, true);
    }

    window.addEventListener("keydown", onKeyDown, true);
  }

  async function cancelCapture() {
    capturingHotkey = false;
    await api.setHotkey(localHotkey);
  }

  async function resetHotkey() {
    localHotkey = "Pause";
    await updateHotkey("Pause");
    onsave({ hotkey: "Pause" });
  }

  function handleSoundToggle(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    updateSoundEnabled(checked);
    onsave({ soundEnabled: checked });
  }

  function handleConfirmExitToggle(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    setConfirmExitDuringRecording(checked);
    onsave({ confirmExitDuringRecording: checked });
  }

  function handleLanguageChange(e: Event) {
    const lang = (e.target as HTMLSelectElement).value as "en" | "uk";
    setLanguage(lang);
    onsave({ language: lang });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !capturingHotkey) { onclose(); return; }
    if (e.key === "Escape" &&  capturingHotkey) { void cancelCapture(); return; }
    if (e.key === "Tab") {
      const dialog = e.currentTarget as HTMLElement;
      const focusable = dialog.querySelectorAll<HTMLElement>(
        'input, select, button, [tabindex]:not([tabindex="-1"])'
      );
      if (!focusable.length) return;
      const first = focusable[0];
      const last  = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last)  { e.preventDefault(); first.focus(); }
    }
  }

  $effect(() => {
    if (open) requestAnimationFrame(() => document.getElementById("settings-hotkey-display")?.focus());
  });
</script>

{#if open}
  <div class="dialog-backdrop">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-dialog-title"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <h2 id="settings-dialog-title">{m.settings_dialog_title()}</h2>

      <!-- Theme -->
      <div class="field">
        <label class="field-label">{m.settings_theme_label()}</label>
        <div class="theme-seg" role="group" aria-label={m.settings_theme_label()}>
          {#each themeOptions as opt}
            <button
              type="button"
              class="theme-btn"
              class:active={theme === opt.value}
              onclick={() => onthemechange(opt.value)}
              aria-pressed={theme === opt.value}
            >{opt.label()}</button>
          {/each}
        </div>
        <p class="hint">{m.settings_theme_hint()}</p>
      </div>

      <div class="divider"></div>

      <!-- Hotkey -->
      <div class="field">
        <label for="settings-hotkey-display" class="field-label">{m.settings_hotkey_label()}</label>
        <div class="hotkey-row">
          <input
            id="settings-hotkey-display"
            type="text"
            value={localHotkey}
            readonly
            aria-label={m.settings_hotkey_current_aria({ hotkey: localHotkey })}
            class="hotkey-input"
          />
          <button type="button" class="btn-sm" onclick={startHotkeyCapture} disabled={capturingHotkey}>
            {capturingHotkey ? m.settings_hotkey_capturing() : m.settings_hotkey_capture_btn()}
          </button>
          <button type="button" class="btn-sm btn-secondary" onclick={resetHotkey} disabled={capturingHotkey}>
            {m.settings_hotkey_reset_btn()}
          </button>
        </div>
      </div>

      <!-- Sound -->
      <div class="field">
        <label class="checkbox-label">
          <input type="checkbox" checked={soundEnabled} onchange={handleSoundToggle} />
          {m.settings_sound_label()}
        </label>
      </div>

      <!-- Confirm exit -->
      <div class="field">
        <label class="checkbox-label">
          <input type="checkbox" checked={confirmExitDuringRecording} onchange={handleConfirmExitToggle} />
          {m.settings_confirm_exit_label()}
        </label>
      </div>

      <!-- Language -->
      <div class="field">
        <label for="settings-language" class="field-label">{m.settings_language_label()}</label>
        <select id="settings-language" value={language} onchange={handleLanguageChange}>
          <option value="en">{m.settings_language_en()}</option>
          <option value="uk">{m.settings_language_uk()}</option>
        </select>
        <p class="hint">{m.settings_language_restart_note()}</p>
      </div>

      <div class="actions">
        <button
          type="button"
          class="btn-primary"
          onclick={async () => { if (capturingHotkey) await cancelCapture(); onclose(); }}
        >{m.btn_close()}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 22px 22px 20px;
    width: 380px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: var(--text-primary);
  }
  h2 { margin: 0 0 16px; font-size: 1.1rem; color: var(--text-primary); }

  .field { display: flex; flex-direction: column; gap: 5px; margin-bottom: 14px; }
  .field:last-of-type { margin-bottom: 0; }
  .field-label { font-weight: 700; font-size: 0.8rem; color: var(--text-secondary); }

  /* Theme segmented control */
  .theme-seg { display: flex; border-radius: 7px; overflow: hidden; border: 1px solid var(--border); }
  .theme-btn {
    flex: 1; padding: 6px 4px; text-align: center;
    font-size: 0.8rem; font-weight: 600; cursor: pointer;
    border: none; border-right: 1px solid var(--border);
    background: var(--surface); color: var(--text-secondary);
    transition: background 0.1s, color 0.1s;
  }
  .theme-btn:last-child { border-right: none; }
  .theme-btn:hover:not(.active) { background: var(--surface-hover); }
  .theme-btn.active { background: var(--accent); color: #ffffff; }

  .divider { height: 1px; background: var(--border); margin: 2px 0 14px; }

  select {
    padding: 7px 8px; border: 1px solid var(--border); border-radius: 5px;
    font-size: 0.875rem; background: var(--surface); color: var(--text-primary);
  }
  .hotkey-row { display: flex; gap: 6px; align-items: center; }
  .hotkey-input {
    flex: 1; padding: 7px 8px; border: 1px solid var(--border); border-radius: 5px;
    font-size: 0.875rem; background: var(--surface-hover); color: var(--text-primary);
    cursor: default;
  }
  .checkbox-label {
    display: flex; align-items: center; gap: 8px;
    cursor: pointer; font-size: 0.875rem; font-weight: normal;
    color: var(--text-primary);
  }
  .hint { margin: 0; font-size: 0.75rem; color: var(--text-muted); }

  .btn-sm {
    padding: 6px 10px; border: 1px solid var(--border); border-radius: 5px;
    font-size: 0.8rem; font-weight: 600; cursor: pointer;
    background: var(--surface-hover); color: var(--text-primary); white-space: nowrap;
  }
  .btn-sm:hover:not(:disabled) { background: var(--border); }
  .btn-sm:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-sm.btn-secondary { background: var(--surface); }

  .actions { display: flex; justify-content: flex-end; margin-top: 16px; }
  .btn-primary {
    padding: 8px 18px; border: none; border-radius: 6px;
    font-size: 0.875rem; font-weight: 700; cursor: pointer;
    background: var(--accent); color: #ffffff;
  }
  .btn-primary:hover { background: var(--accent-hover); }
</style>
```

- [ ] **Step 2: Verify check passes**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/SettingsDialog.svelte
git commit -m "feat(settings): add theme segmented control (Auto/Light/Dark)"
```

---

## Task 8 — ProfileDialog: 2-Column Layout, No Scroll

**Files:**
- Modify: `src/lib/components/ProfileDialog.svelte`

Restructure from single-column with `overflow-y: auto` to a 2-column grid with section labels. All 10 fields fit in ~430px — no scroll needed. Also adds conditional filename visibility based on `outputMode`.

- [ ] **Step 1: Rewrite `ProfileDialog.svelte`**

```svelte
<script lang="ts">
  import type { RecordingProfile, OutputMode } from "../types";
  import * as m from "../../paraglide/messages";

  interface Props {
    profile: RecordingProfile | null;
    open: boolean;
    onclose: () => void;
    onsave: (profile: RecordingProfile) => void;
  }

  let { profile, open, onclose, onsave }: Props = $props();

  let name         = $state("");
  let description  = $state("");
  let outputFolder = $state("Recordings");
  let outputMode   = $state<OutputMode>("Mix");
  let sampleRate   = $state(48000);
  let micVolume    = $state(1.0);
  let loopbackVolume = $state(0.5);
  let micFilename      = $state("mic");
  let loopbackFilename = $state("loopback");
  let mixFilename      = $state("mix");
  let error  = $state("");
  let editId = $state<string | null>(null);

  const outputModes: { value: OutputMode; label: string }[] = [
    { value: "Microphone",        label: m.mode_microphone() },
    { value: "Loopback",          label: m.mode_loopback() },
    { value: "Mix",               label: m.mode_mix() },
    { value: "MixPlusMicrophone", label: m.mode_mix_plus_mic() },
    { value: "MixPlusLoopback",   label: m.mode_mix_plus_loopback() },
  ];
  const sampleRates = [8000, 16000, 44100, 48000];

  // Filename visibility depends on which streams are produced
  let showMicFilename = $derived(outputMode !== "Loopback");
  let showLoopbackFilename = $derived(outputMode !== "Microphone");
  let showMixFilename = $derived(
    outputMode === "Mix" || outputMode === "MixPlusMicrophone" || outputMode === "MixPlusLoopback"
  );

  let micFillPct      = $derived(((micVolume     - 0) / (4 - 0)) * 100);
  let loopbackFillPct = $derived(((loopbackVolume - 0) / (4 - 0)) * 100);

  $effect(() => {
    if (open && profile) {
      editId           = profile.id;
      name             = profile.name;
      description      = profile.description;
      outputFolder     = profile.outputFolder;
      outputMode       = profile.outputMode;
      sampleRate       = profile.sampleRate;
      micVolume        = profile.micVolume;
      loopbackVolume   = profile.loopbackVolume;
      micFilename      = profile.micFilename;
      loopbackFilename = profile.loopbackFilename;
      mixFilename      = profile.mixFilename;
    } else if (open && !profile) {
      editId = null; name = ""; description = ""; outputFolder = "Recordings";
      outputMode = "Mix"; sampleRate = 48000; micVolume = 1.0; loopbackVolume = 0.5;
      micFilename = "mic"; loopbackFilename = "loopback"; mixFilename = "mix";
    }
    error = "";
  });

  let isEdit = $derived(editId !== null);
  let title  = $derived(isEdit ? m.profile_dialog_edit_title() : m.profile_dialog_create_title());

  function handleSubmit() {
    if (!name.trim()) { error = m.error_profile_name_required(); return; }
    const forbidden = /[<>:"/\\|?*]/;
    const filenames = [
      ...(showMicFilename      ? [micFilename]      : []),
      ...(showLoopbackFilename ? [loopbackFilename]  : []),
      ...(showMixFilename      ? [mixFilename]       : []),
    ];
    for (const fn of filenames) {
      if (!fn.trim() || forbidden.test(fn)) { error = m.error_filename_invalid(); return; }
    }
    onsave({
      id: editId ?? crypto.randomUUID(),
      name: name.trim(), description: description.trim(),
      outputFolder, outputMode, sampleRate,
      micVolume, loopbackVolume,
      micFilename: micFilename.trim(),
      loopbackFilename: loopbackFilename.trim(),
      mixFilename: mixFilename.trim(),
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { onclose(); return; }
    if (e.key === "Tab") {
      const dialog = e.currentTarget as HTMLElement;
      const focusable = dialog.querySelectorAll<HTMLElement>(
        'input, select, button, [tabindex]:not([tabindex="-1"])'
      );
      if (!focusable.length) return;
      const first = focusable[0];
      const last  = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  }

  $effect(() => {
    if (open) requestAnimationFrame(() => document.getElementById("profile-name")?.focus());
  });
</script>

{#if open}
  <div class="dialog-backdrop">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <h2>{title}</h2>

      <!-- BASIC -->
      <p class="sec-label">{m.profile_section_basic()}</p>
      <div class="grid-2">
        <div class="field">
          <label for="profile-name">{m.field_name()}</label>
          <input id="profile-name" type="text" bind:value={name} />
        </div>
        <div class="field">
          <label for="profile-desc">{m.field_description()}</label>
          <input id="profile-desc" type="text" bind:value={description} />
        </div>
        <div class="field col-span">
          <label for="profile-folder">{m.field_output_folder()}</label>
          <input id="profile-folder" type="text" bind:value={outputFolder} />
        </div>
      </div>

      <!-- AUDIO -->
      <p class="sec-label">{m.profile_section_audio()}</p>
      <div class="grid-2">
        <div class="field">
          <label for="profile-mode">{m.field_output_mode()}</label>
          <select id="profile-mode" bind:value={outputMode}>
            {#each outputModes as mode}
              <option value={mode.value}>{mode.label}</option>
            {/each}
          </select>
        </div>
        <div class="field">
          <label for="profile-rate">{m.field_sample_rate()}</label>
          <select id="profile-rate" bind:value={sampleRate}>
            {#each sampleRates as rate}
              <option value={rate}>{rate} Hz</option>
            {/each}
          </select>
        </div>
        <div class="field">
          <label for="profile-mic-vol">
            {m.field_mic_volume()}
            <span class="slider-val">{micVolume.toFixed(1)}</span>
          </label>
          <input
            id="profile-mic-vol" type="range" min="0" max="4" step="0.1"
            bind:value={micVolume}
            style="--fill: {micFillPct}%"
          />
        </div>
        <div class="field">
          <label for="profile-loop-vol">
            {m.field_loopback_volume()}
            <span class="slider-val">{loopbackVolume.toFixed(1)}</span>
          </label>
          <input
            id="profile-loop-vol" type="range" min="0" max="4" step="0.1"
            bind:value={loopbackVolume}
            style="--fill: {loopbackFillPct}%"
          />
        </div>
      </div>

      <!-- FILE NAMES -->
      <p class="sec-label">{m.profile_section_filenames()}</p>
      <div class="grid-2">
        {#if showMicFilename}
          <div class="field">
            <label for="profile-mic-fn">{m.field_mic_filename()}</label>
            <input id="profile-mic-fn" type="text" bind:value={micFilename} />
          </div>
        {/if}
        {#if showLoopbackFilename}
          <div class="field">
            <label for="profile-loop-fn">{m.field_loopback_filename()}</label>
            <input id="profile-loop-fn" type="text" bind:value={loopbackFilename} />
          </div>
        {/if}
        {#if showMixFilename}
          <div class="field">
            <label for="profile-mix-fn">{m.field_mix_filename()}</label>
            <input id="profile-mix-fn" type="text" bind:value={mixFilename} />
          </div>
        {/if}
      </div>

      {#if error}
        <div class="error" role="alert">{error}</div>
      {/if}

      <div class="actions">
        <button type="button" class="btn-secondary" onclick={onclose}>{m.btn_cancel()}</button>
        <button type="button" class="btn-primary"   onclick={handleSubmit}>
          {isEdit ? m.btn_save() : m.btn_create()}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px 20px 18px;
    width: 400px;
    /* No max-height / overflow — all fields fit in ~430px */
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: var(--text-primary);
  }
  h2 { margin: 0 0 10px; font-size: 1.1rem; color: var(--text-primary); }

  .sec-label {
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.09em;
    color: var(--sec-label);
    margin: 8px 0 6px;
  }

  /* 2-column grid */
  .grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0 10px;
  }
  .col-span { grid-column: 1 / -1; }

  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-bottom: 8px;
  }
  .field label {
    font-weight: 600;
    font-size: 0.75rem;
    color: var(--text-secondary);
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .slider-val {
    font-weight: 700;
    color: var(--accent);
  }

  input[type="text"],
  select {
    padding: 5px 7px;
    border: 1px solid var(--border);
    border-radius: 5px;
    font-size: 0.8rem;
    background: var(--surface);
    color: var(--text-primary);
    width: 100%;
  }
  input[type="text"]:focus-visible,
  select:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }

  /* range sliders inherit global styles from app.css */
  input[type="range"] {
    margin-top: 2px;
  }

  .error {
    padding: 7px 10px;
    background: var(--error-bg);
    border: 1px solid var(--error-border);
    border-radius: 5px;
    color: var(--error-text);
    font-size: 0.8rem;
    margin-bottom: 10px;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 10px;
  }
  .btn-primary, .btn-secondary {
    padding: 7px 16px;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 700;
    cursor: pointer;
  }
  .btn-primary  { border: none; background: var(--accent); color: #ffffff; }
  .btn-primary:hover  { background: var(--accent-hover); }
  .btn-secondary { border: 1px solid var(--border); background: var(--surface); color: var(--text-primary); }
  .btn-secondary:hover { background: var(--surface-hover); }
</style>
```

- [ ] **Step 2: Verify check passes**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ProfileDialog.svelte
git commit -m "feat(ui): 2-column ProfileDialog layout — no scrollbar, section labels, conditional filenames"
```

---

## Task 9 — Visual Smoke-Test + Final Cleanup

Run the dev server and visually verify all states and themes.

- [ ] **Step 1: Start dev server and open the app**

```bash
pnpm vite:dev
```

Open http://localhost:1420 in a browser (or run `pnpm dev` for full Tauri window).

- [ ] **Step 2: Verify light theme (Auto / Windows default)**

Check in order:
1. App background is `#f0f0f0`, cards are white, text is dark — no hardcoded colors visible
2. Status card: no border in Idle, gray dimmed timer
3. Volume sliders: visible white thumb with blue border on gray track
4. Settings gear is 32px, title is bold 16px
5. Open Settings → theme segmented control shows, "Auto" is highlighted blue
6. Switch to Dark → app switches instantly
7. Switch back to Light → switches instantly
8. Switch to Auto → follows OS setting

- [ ] **Step 3: Verify dark theme states**

1. Set theme to Dark
2. Status card: no border, dark gray dimmed timer
3. Start a recording: red border appears on status card, blinking dot
4. Pause: amber border appears, timer is amber-tinted
5. Stop: border disappears, timer dims

- [ ] **Step 4: Verify ProfileDialog**

1. Click + to create a new profile
2. Dialog opens: 3 sections (Основне / Аудіо / Імена файлів), 2-column layout, no scrollbar
3. Change Output Mode to "Microphone only" — Loopback filename field disappears, Mix filename disappears
4. Change to "System audio only" — Mic filename disappears
5. Change to "Mix" — all 3 filename fields visible

- [ ] **Step 5: Run final check**

```bash
pnpm check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "chore: visual smoke-test passed — UI theme redesign complete"
```

---

## Quick Reference

| Command | Purpose |
|---|---|
| `pnpm check` | svelte-check + tsc (run after every task) |
| `pnpm vite:dev` | Frontend only, fast reload at http://localhost:1420 |
| `pnpm dev` | Full Tauri app (slower, use for final smoke-test) |

## CSS Token Reference

All tokens live in `src/app.css` on `:root` (light) and `[data-theme="dark"]`.
Use `var(--token-name)` everywhere. Never write hex color values in component `<style>` blocks.

| Need | Token |
|---|---|
| Page/window background | `--bg` |
| Card / dialog / input bg | `--surface` |
| Hover state on surface | `--surface-hover` |
| Border (card, input) | `--border` |
| Unfilled slider track | `--track` |
| Primary text | `--text-primary` |
| Label text | `--text-secondary` |
| Hint / muted text | `--text-muted` |
| Blue accent (slider, link) | `--accent` |
| Section title color | `--sec-label` |
| Start button | `--btn-start-bg` |
| Pause button | `--btn-pause-bg` |
| Stop button | `--btn-stop-bg` |
| Resume button | `--btn-resume-bg` |
| Error text/border/bg | `--error-*` |
