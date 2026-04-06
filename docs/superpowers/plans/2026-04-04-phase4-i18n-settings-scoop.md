# Phase 4: i18n, Settings Dialog, aria-live, Scoop — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Paraglide JS i18n (en + uk), a Settings modal dialog, localized aria-live announcements, and a Scoop install manifest to AudioCaptor.

**Architecture:** New `settings.svelte.ts` store owns language/hotkey/sound/confirm-exit configuration; `SettingsDialog.svelte` and `ConfirmExitDialog.svelte` follow the `ProfileDialog` modal pattern; Paraglide compiles typed message functions from `messages/*.json` at build time; the existing `on_window_event` Rust handler is removed and replaced with a frontend `onCloseRequested` handler.

**Tech Stack:** Tauri v2 · Svelte 5 · Paraglide JS (`@inlang/paraglide-js`) · Rust (serde v3 migration) · Scoop (manifest)

---

## Spec

`docs/superpowers/specs/2026-04-04-phase4-i18n-settings-scoop-design.md`

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `src-tauri/src/settings.rs` | Add `language`, `confirm_exit_during_recording`; v2→v3 migration; `Default::version = 3` |
| Modify | `src-tauri/src/hotkey.rs` | Add public `unregister()` function |
| Modify | `src-tauri/src/lib.rs` | Add `unregister_hotkey` IPC; remove `on_window_event` handler |
| Modify | `package.json` | Add `@inlang/paraglide-js` dev dep |
| Modify | `vite.config.ts` | Add Paraglide Vite plugin |
| Create | `project.inlang/settings.json` | Paraglide project config |
| Create | `messages/en.json` | English message strings |
| Create | `messages/uk.json` | Ukrainian message strings |
| Modify | `src/lib/types/index.ts` | Add `language`, `confirmExitDuringRecording` to `Settings` |
| Modify | `src/lib/utils/invoke.ts` | Add `unregisterHotkey()` |
| Create | `src/lib/i18n.ts` | `initLanguage()` — call exactly once at startup |
| Create | `src/lib/stores/settings.svelte.ts` | Reactive store for language/hotkey/sound/confirm-exit |
| Modify | `src/lib/stores/recording.svelte.ts` | Remove `hotkey`, `soundEnabled`, `updateHotkey`, `updateSoundEnabled`; add `needsMic`, `needsLoopback` getters; remove `readinessHint` |
| Create | `src/lib/components/SettingsDialog.svelte` | Settings modal (hotkey capture, sound, confirm-exit, language) |
| Create | `src/lib/components/ConfirmExitDialog.svelte` | "Stop and exit?" modal shown on close-during-recording |
| Modify | `src/App.svelte` | `initialized` guard; wire dialogs; `onCloseRequested`; fix `scheduleSave`; ⚙ button |
| Modify | `src/lib/components/StatusIndicator.svelte` | Localize state labels |
| Modify | `src/lib/components/RecordControls.svelte` | Localize button labels + aria-labels |
| Modify | `src/lib/components/DeviceSelect.svelte` | Localize placeholder via `m.select_device_placeholder()` |
| Modify | `src/lib/components/ProfileSelector.svelte` | Localize labels |
| Modify | `src/lib/components/ProfileDialog.svelte` | Localize labels + errors |
| Create | `bucket/audiocaptor.json` | Scoop manifest |

---

## Task 1: Rust — Settings v2→v3 migration

**Files:**
- Modify: `src-tauri/src/settings.rs`

- [ ] **Step 1: Add new fields and update Default**

  In `src-tauri/src/settings.rs`, add two fields to the `Settings` struct and update `Default::version` to `3`:

  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase", default)]
  pub struct Settings {
      pub version: u32,
      pub selected_mic: Option<String>,
      pub selected_loopback: Option<String>,
      pub hotkey: String,
      pub sound_enabled: bool,
      pub profiles: Vec<RecordingProfile>,
      pub active_profile_id: String,
      pub language: String,                        // NEW
      pub confirm_exit_during_recording: bool,     // NEW
      // Legacy fields — used only during migration from v1
      #[serde(default, skip_serializing)]
      mic_volume: f32,
      #[serde(default, skip_serializing)]
      loopback_volume: f32,
      #[serde(default = "default_output_mode", skip_serializing)]
      output_mode: OutputMode,
      #[serde(default = "default_sample_rate", skip_serializing)]
      sample_rate: u32,
  }
  ```

  Update `Default` implementation — change `version: 0` to `version: 3` and add new fields:

  ```rust
  impl Default for Settings {
      fn default() -> Self {
          Self {
              version: 3,                               // was 0
              selected_mic: None,
              selected_loopback: None,
              hotkey: "Pause".to_string(),
              sound_enabled: true,
              profiles: vec![RecordingProfile::default()],
              active_profile_id: "default".to_string(),
              language: "en".to_string(),               // NEW
              confirm_exit_during_recording: true,      // NEW
              mic_volume: 1.0,
              loopback_volume: 0.5,
              output_mode: OutputMode::Mix,
              sample_rate: 48000,
          }
      }
  }
  ```

- [ ] **Step 2: Extend the migration loop**

  In `migrate_settings`, replace the current `v2` arm (which `break`s) with a fall-through that sets version to `3`, and add a `v3` terminal arm:

  ```rust
  2 => {
      // Ensure at least one profile exists
      if settings.profiles.is_empty() {
          settings.profiles = vec![RecordingProfile::default()];
          settings.active_profile_id = "default".to_string();
      }
      settings.version = 3;
      // No break — loop continues to v3 arm below
  }
  3 => {
      // language and confirm_exit_during_recording have #[serde(default)]
      // serde fills missing fields automatically — no data transform needed
      break;
  }
  ```

- [ ] **Step 3: Update tests that assert version == 0 on Default**

  The test `settings_v2_has_profiles_and_active_id` asserts `assert_eq!(settings.version, 0)` — this will fail. Update it:

  ```rust
  #[test]
  fn settings_v2_has_profiles_and_active_id() {
      let settings = Settings::default();
      assert_eq!(settings.version, 3);           // was 0
      assert_eq!(settings.profiles.len(), 1);
      assert_eq!(settings.active_profile_id, "default");
      assert_eq!(settings.profiles[0].name, "Default");
  }
  ```

  Also update `settings_serialization_roundtrip` — it manually sets `version: 2`, which is fine; no change needed there.

  Add a new test for v3 migration and new fields:

  ```rust
  #[test]
  fn migrate_v2_to_v3_adds_language_and_confirm_exit() {
      let json = r#"{
          "version": 2,
          "selectedMic": null,
          "selectedLoopback": null,
          "hotkey": "Pause",
          "soundEnabled": true,
          "profiles": [],
          "activeProfileId": "default"
      }"#;
      let settings: Settings = serde_json::from_str(json).unwrap();
      let migrated = migrate_settings(settings);
      assert_eq!(migrated.version, 3);
      assert_eq!(migrated.language, "en");
      assert!(migrated.confirm_exit_during_recording);
  }

  #[test]
  fn settings_default_has_version_3() {
      let s = Settings::default();
      assert_eq!(s.version, 3);
      assert_eq!(s.language, "en");
      assert!(s.confirm_exit_during_recording);
  }
  ```

- [ ] **Step 4: Run Rust tests**

  ```bash
  cd src-tauri && cargo test --lib settings
  ```

  Expected: all tests pass. If `settings_missing_version_defaults_to_zero` fails, note it still tests JSON parsing (version field absent → 0 via serde default) — this test is for serde deserialization, not `Default::default()`. It should still pass unchanged.

- [ ] **Step 5: Commit**

  ```bash
  git add src-tauri/src/settings.rs
  git commit -m "feat: settings v2→v3 migration with language and confirm_exit fields"
  ```

---

## Task 2: Rust — hotkey::unregister + IPC command

**Files:**
- Modify: `src-tauri/src/hotkey.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add `unregister()` to hotkey.rs**

  After the existing `register()` function, add:

  ```rust
  /// Unregister all global shortcuts. Used during hotkey capture mode so the
  /// current shortcut does not fire while the user presses a new one.
  pub fn unregister(app: &AppHandle) -> Result<(), String> {
      app.global_shortcut()
          .unregister_all()
          .map_err(|e| format!("Failed to unregister hotkey: {e}"))
  }
  ```

- [ ] **Step 2: Add `unregister_hotkey` IPC command to lib.rs**

  After the existing `set_hotkey` command:

  ```rust
  #[tauri::command]
  fn unregister_hotkey(app: tauri::AppHandle) -> Result<(), String> {
      hotkey::unregister(&app)
  }
  ```

  Add to the `invoke_handler!` macro at the bottom of `lib.rs`:

  ```rust
  tauri::generate_handler![
      // ... existing commands ...
      set_hotkey,
      unregister_hotkey,   // ADD THIS
      // ...
  ]
  ```

- [ ] **Step 3: Run Rust tests**

  ```bash
  cd src-tauri && cargo test --lib
  ```

  Expected: all tests pass (no new tests needed — `unregister` wraps an existing plugin method).

- [ ] **Step 4: Commit**

  ```bash
  git add src-tauri/src/hotkey.rs src-tauri/src/lib.rs
  git commit -m "feat: add unregister_hotkey IPC command for hotkey capture mode"
  ```

---

## Task 3: Rust — remove on_window_event CloseRequested handler

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Remove the on_window_event block**

  In `lib.rs`, find and remove the entire `.on_window_event(...)` block (currently lines 617–631):

  ```rust
  // REMOVE THIS ENTIRE BLOCK:
  .on_window_event(|window, event| {
      if let tauri::WindowEvent::CloseRequested { .. } = event {
          let app = window.app_handle();
          if let Some(state) = app.try_state::<SharedState>() {
              let is_recording = state
                  .lock()
                  .map(|s| s.recording_state != RecordingState::Idle)
                  .unwrap_or(false);
              if is_recording {
                  log::info!("Window closing during recording — stopping recording");
                  let _ = stop_recording_inner(&state);
              }
          }
      }
  })
  ```

  Close-during-recording logic now lives entirely in the frontend (`ConfirmExitDialog` + `onCloseRequested`).

- [ ] **Step 2: Run Rust build to verify no compile errors**

  ```bash
  cd src-tauri && cargo check
  ```

  Expected: compiles cleanly.

- [ ] **Step 3: Commit**

  ```bash
  git add src-tauri/src/lib.rs
  git commit -m "refactor: move close-during-recording handling to frontend"
  ```

---

## Task 4: Paraglide JS setup

**Files:**
- Modify: `package.json`
- Create: `project.inlang/settings.json`
- Modify: `vite.config.ts`

- [ ] **Step 1: Install Paraglide**

  ```bash
  pnpm add -D @inlang/paraglide-js
  ```

- [ ] **Step 2: Create project.inlang/settings.json**

  ```json
  {
    "$schema": "https://inlang.com/schema/project-settings",
    "sourceLanguageTag": "en",
    "languageTags": ["en", "uk"],
    "modules": [
      "https://cdn.jsdelivr.net/npm/@inlang/message-lint-rule-missing-translation@latest/dist/index.js",
      "https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@latest/dist/index.js"
    ],
    "plugin.inlang.messageFormat": {
      "pathPattern": "./messages/{languageTag}.json"
    }
  }
  ```

- [ ] **Step 3: Update vite.config.ts**

  ```ts
  import { defineConfig } from "vite";
  import { svelte } from "@sveltejs/vite-plugin-svelte";
  import { paraglide } from "@inlang/paraglide-js/vite";

  export default defineConfig({
    plugins: [
      svelte(),
      paraglide({
        project: "./project.inlang",
        outdir: "./src/paraglide",
      }),
    ],
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
    },
    build: {
      outDir: "dist",
      emptyOutDir: true,
    },
  });
  ```

- [ ] **Step 4: Create placeholder message files (to allow build)**

  Create `messages/en.json`:
  ```json
  {
    "app_title": "AudioCaptor"
  }
  ```

  Create `messages/uk.json`:
  ```json
  {
    "app_title": "AudioCaptor"
  }
  ```

- [ ] **Step 5: Run vite build to verify Paraglide compiles**

  ```bash
  pnpm vite:build
  ```

  Expected: `src/paraglide/` directory is created with `runtime.js`, `messages.js`, etc. If the Paraglide plugin API differs from above, consult [https://inlang.com/m/gerre34r/library-inlang-paraglideJs](https://inlang.com/m/gerre34r/library-inlang-paraglideJs) for the correct vite plugin import path.

- [ ] **Step 6: Commit**

  ```bash
  git add package.json pnpm-lock.yaml project.inlang/ messages/ vite.config.ts src/paraglide/
  git commit -m "feat: add Paraglide JS i18n setup"
  ```

---

## Task 5: Message files — all UI strings

**Files:**
- Modify: `messages/en.json`
- Create: `messages/uk.json`

- [ ] **Step 1: Write complete messages/en.json**

  ```json
  {
    "app_title": "AudioCaptor",
    "recording_status": "Recording status",
    "status_ready": "Ready",
    "status_recording": "Recording",
    "status_paused": "Paused",
    "recording_duration": "Recording duration: {duration}",
    "audio_devices_section": "Audio devices",
    "select_device_placeholder": "-- Select {label} --",
    "volume_controls_section": "Volume controls",
    "mic_label": "Microphone",
    "loopback_label": "Loopback Device",
    "mic_volume_label": "Microphone Volume",
    "loopback_volume_label": "Loopback Volume",
    "recording_controls": "Recording controls",
    "btn_start": "Start",
    "btn_pause": "Pause",
    "btn_resume": "Resume",
    "btn_stop": "Stop",
    "start_recording_aria": "Start recording (Alt+S)",
    "start_recording_disabled_aria": "Start recording (Alt+S). {hint}",
    "pause_recording_aria": "Pause recording (Alt+P)",
    "resume_recording_aria": "Resume recording (Alt+R)",
    "stop_recording_aria": "Stop recording (Alt+T)",
    "hint_select_mic": "Select a microphone",
    "hint_select_loopback": "Select a system audio device",
    "hint_select_both": "Select microphone and system audio device",
    "profile_section": "Recording profile",
    "profile_label": "Profile",
    "create_profile_aria": "Create new profile",
    "edit_profile_aria": "Edit profile",
    "delete_profile_aria": "Delete profile",
    "delete_profile_confirm": "Delete this profile?",
    "profile_dialog_create_title": "New Profile",
    "profile_dialog_edit_title": "Edit Profile",
    "field_name": "Name",
    "field_description": "Description",
    "field_output_folder": "Output Folder",
    "field_output_mode": "Output Mode",
    "field_sample_rate": "Sample Rate",
    "field_mic_volume": "Mic Volume",
    "field_loopback_volume": "Loopback Volume",
    "field_mic_filename": "Mic Filename",
    "field_loopback_filename": "Loopback Filename",
    "field_mix_filename": "Mix Filename",
    "mode_microphone": "Microphone only",
    "mode_loopback": "System audio only",
    "mode_mix": "Mix (Mic + System)",
    "mode_mix_plus_mic": "Mix + Microphone (2 files)",
    "mode_mix_plus_loopback": "Mix + System audio (2 files)",
    "error_profile_name_required": "Profile name is required",
    "error_filename_invalid": "Filenames must not be empty or contain < > : \" / \\ | ? *",
    "btn_cancel": "Cancel",
    "btn_save": "Save",
    "btn_create": "Create",
    "settings_btn_aria": "Open settings",
    "settings_dialog_title": "Settings",
    "settings_hotkey_label": "Global Hotkey",
    "settings_hotkey_current_aria": "Current hotkey: {hotkey}",
    "settings_hotkey_capture_btn": "Capture New",
    "settings_hotkey_reset_btn": "Reset to Pause",
    "settings_hotkey_capturing": "Press a key...",
    "settings_sound_label": "Sound notifications",
    "settings_confirm_exit_label": "Confirm exit during recording",
    "settings_language_label": "Language",
    "settings_language_en": "English",
    "settings_language_uk": "Українська",
    "settings_language_restart_note": "Change takes effect after restart",
    "btn_close": "Close",
    "confirm_exit_title": "Stop recording?",
    "confirm_exit_body": "Recording is in progress. Do you want to stop and exit?",
    "btn_stop_and_exit": "Stop and exit",
    "live_recording_started": "Recording started",
    "live_recording_paused": "Recording paused",
    "live_recording_stopped": "Recording stopped",
    "live_status_info": "{state}, {duration}"
  }
  ```

- [ ] **Step 2: Write complete messages/uk.json**

  ```json
  {
    "app_title": "AudioCaptor",
    "recording_status": "Статус запису",
    "status_ready": "Готовий",
    "status_recording": "Запис",
    "status_paused": "Пауза",
    "recording_duration": "Тривалість запису: {duration}",
    "audio_devices_section": "Аудіопристрої",
    "select_device_placeholder": "-- Оберіть {label} --",
    "volume_controls_section": "Гучність",
    "mic_label": "Мікрофон",
    "loopback_label": "Системний звук",
    "mic_volume_label": "Гучність мікрофона",
    "loopback_volume_label": "Гучність системного звуку",
    "recording_controls": "Керування записом",
    "btn_start": "Старт",
    "btn_pause": "Пауза",
    "btn_resume": "Продовжити",
    "btn_stop": "Стоп",
    "start_recording_aria": "Почати запис (Alt+S)",
    "start_recording_disabled_aria": "Почати запис (Alt+S). {hint}",
    "pause_recording_aria": "Призупинити запис (Alt+P)",
    "resume_recording_aria": "Відновити запис (Alt+R)",
    "stop_recording_aria": "Зупинити запис (Alt+T)",
    "hint_select_mic": "Оберіть мікрофон",
    "hint_select_loopback": "Оберіть пристрій системного звуку",
    "hint_select_both": "Оберіть мікрофон та пристрій системного звуку",
    "profile_section": "Профіль запису",
    "profile_label": "Профіль",
    "create_profile_aria": "Створити новий профіль",
    "edit_profile_aria": "Редагувати профіль",
    "delete_profile_aria": "Видалити профіль",
    "delete_profile_confirm": "Видалити цей профіль?",
    "profile_dialog_create_title": "Новий профіль",
    "profile_dialog_edit_title": "Редагування профілю",
    "field_name": "Назва",
    "field_description": "Опис",
    "field_output_folder": "Папка для записів",
    "field_output_mode": "Режим виводу",
    "field_sample_rate": "Частота дискретизації",
    "field_mic_volume": "Гучність мікрофона",
    "field_loopback_volume": "Гучність системного звуку",
    "field_mic_filename": "Ім'я файлу мікрофона",
    "field_loopback_filename": "Ім'я файлу системного звуку",
    "field_mix_filename": "Ім'я файлу мікшу",
    "mode_microphone": "Тільки мікрофон",
    "mode_loopback": "Тільки системний звук",
    "mode_mix": "Мікс (Мік + Система)",
    "mode_mix_plus_mic": "Мікс + Мікрофон (2 файли)",
    "mode_mix_plus_loopback": "Мікс + Системний звук (2 файли)",
    "error_profile_name_required": "Назва профілю обов'язкова",
    "error_filename_invalid": "Імена файлів не можуть бути порожніми або містити < > : \" / \\ | ? *",
    "btn_cancel": "Скасувати",
    "btn_save": "Зберегти",
    "btn_create": "Створити",
    "settings_btn_aria": "Відкрити налаштування",
    "settings_dialog_title": "Налаштування",
    "settings_hotkey_label": "Глобальна гаряча клавіша",
    "settings_hotkey_current_aria": "Поточна клавіша: {hotkey}",
    "settings_hotkey_capture_btn": "Захопити нову",
    "settings_hotkey_reset_btn": "Скинути до Pause",
    "settings_hotkey_capturing": "Натисніть клавішу...",
    "settings_sound_label": "Звукові сповіщення",
    "settings_confirm_exit_label": "Підтвердження виходу під час запису",
    "settings_language_label": "Мова",
    "settings_language_en": "English",
    "settings_language_uk": "Українська",
    "settings_language_restart_note": "Зміна набуде дії після перезапуску",
    "btn_close": "Закрити",
    "confirm_exit_title": "Зупинити запис?",
    "confirm_exit_body": "Запис триває. Зупинити та вийти?",
    "btn_stop_and_exit": "Зупинити і вийти",
    "live_recording_started": "Запис розпочато",
    "live_recording_paused": "Запис призупинено",
    "live_recording_stopped": "Запис зупинено",
    "live_status_info": "{state}, {duration}"
  }
  ```

- [ ] **Step 3: Rebuild to verify both files compile without missing-translation warnings**

  ```bash
  pnpm vite:build
  ```

  Expected: `src/paraglide/messages.js` exports all keys as typed functions.

- [ ] **Step 4: Commit**

  ```bash
  git add messages/en.json messages/uk.json
  git commit -m "feat: add en/uk message files for Paraglide i18n"
  ```

---

## Task 6: TypeScript types + invoke.ts

**Files:**
- Modify: `src/lib/types/index.ts`
- Modify: `src/lib/utils/invoke.ts`

- [ ] **Step 1: Add new fields to Settings interface**

  In `src/lib/types/index.ts`, update the `Settings` interface:

  ```ts
  export interface Settings {
    version: number;
    selectedMic: string | null;
    selectedLoopback: string | null;
    hotkey: string;
    soundEnabled: boolean;
    profiles: RecordingProfile[];
    activeProfileId: string;
    language: "en" | "uk";                  // NEW
    confirmExitDuringRecording: boolean;    // NEW
  }
  ```

- [ ] **Step 2: Add unregisterHotkey to invoke.ts**

  In `src/lib/utils/invoke.ts`, after `setHotkey`:

  ```ts
  export async function unregisterHotkey(): Promise<void> {
    return invoke("unregister_hotkey");
  }
  ```

- [ ] **Step 3: Run type check**

  ```bash
  pnpm check
  ```

  Expected: type errors in `recording.svelte.ts` and `App.svelte` where `hotkey`/`soundEnabled` are used from recording store — these will be fixed in Tasks 7–11. Any other errors are unexpected.

- [ ] **Step 4: Commit**

  ```bash
  git add src/lib/types/index.ts src/lib/utils/invoke.ts
  git commit -m "feat: add language + confirmExitDuringRecording to Settings types and unregisterHotkey IPC wrapper"
  ```

---

## Task 7: settings.svelte.ts store + recording store cleanup

**Files:**
- Create: `src/lib/stores/settings.svelte.ts`
- Modify: `src/lib/stores/recording.svelte.ts`

- [ ] **Step 1: Create src/lib/stores/settings.svelte.ts**

  ```ts
  import type { Settings } from "../types";
  import * as api from "../utils/invoke";

  let language = $state<"en" | "uk">("en");
  let confirmExitDuringRecording = $state(true);
  let hotkey = $state("Pause");
  let soundEnabled = $state(true);
  let settingsVersion = $state(3);

  export function getSettings() {
    return {
      get language() { return language; },
      get confirmExitDuringRecording() { return confirmExitDuringRecording; },
      get hotkey() { return hotkey; },
      get soundEnabled() { return soundEnabled; },
      get version() { return settingsVersion; },
    };
  }

  export function loadSettingsFields(s: Settings) {
    language = (s.language as "en" | "uk") ?? "en";
    confirmExitDuringRecording = s.confirmExitDuringRecording ?? true;
    hotkey = s.hotkey;
    soundEnabled = s.soundEnabled;
    settingsVersion = s.version;
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
  ```

- [ ] **Step 2: Remove hotkey and soundEnabled from recording.svelte.ts**

  In `src/lib/stores/recording.svelte.ts`:

  1. Remove `let soundEnabled = $state(true);`
  2. Remove `let hotkey = $state("Pause");`
  3. Remove from `getRecording()` return object: `soundEnabled` getter/setter and `hotkey` getter/setter
  4. Remove `export async function updateHotkey(...)` function
  5. Remove `export async function updateSoundEnabled(...)` function
  6. In `loadSettingsIntoStore()`, remove the lines `hotkey = s.hotkey;` and `soundEnabled = s.soundEnabled;` — these are now loaded by `loadSettingsFields()` in the settings store

  The updated `loadSettingsIntoStore()`:
  ```ts
  export async function loadSettingsIntoStore() {
    const s = await api.loadSettings();
    selectedMic = s.selectedMic;
    selectedLoopback = s.selectedLoopback;
    // hotkey and soundEnabled now loaded by settings store's loadSettingsFields()
  }
  ```

- [ ] **Step 3: Run type check**

  ```bash
  pnpm check
  ```

  Expected: errors in `App.svelte` for removed `recording.hotkey`, `recording.soundEnabled`, `updateHotkey`, `updateSoundEnabled` — these are fixed in Task 11.

- [ ] **Step 4: Commit**

  ```bash
  git add src/lib/stores/settings.svelte.ts src/lib/stores/recording.svelte.ts
  git commit -m "feat: settings store with language/hotkey/sound/confirm-exit; remove from recording store"
  ```

---

## Task 8: i18n.ts + initialized flag in App.svelte

**Files:**
- Create: `src/lib/i18n.ts`
- Modify: `src/App.svelte` (partial — initialized flag only)

- [ ] **Step 1: Create src/lib/i18n.ts**

  ```ts
  import { setLanguageTag } from "../paraglide/runtime";

  /**
   * Initialize the app language. Call EXACTLY ONCE at startup — never again.
   * Paraglide's setLanguageTag() is reactive; calling it again would immediately
   * re-render all strings without a restart, violating FR7.4.
   */
  export function initLanguage(lang: "en" | "uk") {
    setLanguageTag(lang);
  }

  /**
   * Detect the system language for first-run initialization.
   * Maps 'ru' to 'uk' per FR7.7.
   */
  export function detectLanguage(): "en" | "uk" {
    const lang = navigator.language.toLowerCase();
    if (lang.startsWith("uk") || lang.startsWith("ru")) return "uk";
    return "en";
  }
  ```

- [ ] **Step 2: Add initialized guard to App.svelte**

  At the top of `App.svelte`'s `<script>`, add:

  ```ts
  import { initLanguage, detectLanguage } from "./lib/i18n";
  import { getSettings, loadSettingsFields } from "./lib/stores/settings.svelte";

  let initialized = $state(false);
  const appSettings = getSettings();
  ```

  In `onMount`, update `loadSettingsIntoStore` call to also load settings fields and init language:

  ```ts
  onMount(() => {
    (async () => {
      const s = await api.loadSettings(); // load raw settings object
      loadSettingsFields(s);              // populate settings store
      // Detect language on first run (language field is "en" default on v3 default)
      // If the stored language equals the default and no prior run, detect from system
      initLanguage(appSettings.language === "en" && !s.language ? detectLanguage() : appSettings.language);
      await loadSettingsIntoStore();      // loads mic/loopback into recording store
      await loadProfiles();
      const active = profileStore.active;
      if (active) applyProfile(active);
      await refreshDevices();
      await initRecordingListener();
      await initDeviceListener();
      initialized = true;
    })();
    window.addEventListener("keydown", handleMnemonic);
    return () => window.removeEventListener("keydown", handleMnemonic);
  });
  ```

  Wrap the entire `<main>` template content with `{#if initialized}...{/if}`:

  ```html
  {#if initialized}
    <main role="application" aria-label="AudioCaptor">
      <!-- ... all existing content ... -->
    </main>
  {/if}
  ```

  > **Note on first-run language detection:** The settings file on a fresh install will have `language: "en"` (serde default). To distinguish "user explicitly chose English" from "first run, detect from system", check `s.language` from the raw settings — if the JSON had no `language` field, `s.language` will be `"en"` via serde default, indistinguishable. Simplest correct approach: if `settingsVersion < 3` on load (i.e., this is a migration), run detection; otherwise use stored value. Adjust the logic as follows:

  ```ts
  const rawSettings = await api.loadSettings();
  // Detect language only when migrating from pre-v3 (first time language field appears)
  const isFirstLanguageRun = rawSettings.version < 3 || rawSettings.language === "";
  loadSettingsFields(rawSettings);
  const lang = isFirstLanguageRun ? detectLanguage() : appSettings.language;
  // Persist detected language if needed
  if (isFirstLanguageRun) {
    setLanguage(lang);
    // Save will happen via scheduleSave() later
  }
  initLanguage(lang);
  ```

  Note: `api.loadSettings()` returns the *migrated* settings (migration runs on the Rust side), so `rawSettings.version` will be 3 even on first run. For simplest behavior: skip detection, default to `"en"`, and let users change via Settings dialog. Detection is a nice-to-have, not blocking.

- [ ] **Step 3: Run type check**

  ```bash
  pnpm check
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add src/lib/i18n.ts src/App.svelte
  git commit -m "feat: i18n init with initialized guard to prevent language flash"
  ```

---

## Task 9: SettingsDialog.svelte

**Files:**
- Create: `src/lib/components/SettingsDialog.svelte`

- [ ] **Step 1: Create the component**

  ```svelte
  <script lang="ts">
    import * as m from "../../paraglide/messages";
    import { updateHotkey, updateSoundEnabled, setLanguage, setConfirmExitDuringRecording } from "../stores/settings.svelte";
    import * as api from "../utils/invoke";

    interface Props {
      open: boolean;
      hotkey: string;
      soundEnabled: boolean;
      confirmExitDuringRecording: boolean;
      language: "en" | "uk";
      onclose: () => void;
      onsave: (patch: Partial<import("../types").Settings>) => void;
    }

    let {
      open,
      hotkey,
      soundEnabled,
      confirmExitDuringRecording,
      language,
      onclose,
      onsave,
    }: Props = $props();

    let capturingHotkey = $state(false);
    let localHotkey = $state(hotkey);

    $effect(() => { localHotkey = hotkey; });

    async function startHotkeyCapture() {
      capturingHotkey = true;
      await api.unregisterHotkey();

      async function onKeyDown(e: KeyboardEvent) {
        e.preventDefault();
        e.stopPropagation();

        if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

        const parts: string[] = [];
        if (e.ctrlKey) parts.push("Ctrl");
        if (e.altKey) parts.push("Alt");
        if (e.shiftKey) parts.push("Shift");
        let key = e.key;
        if (key === " ") key = "Space";
        else if (key.length === 1) key = key.toUpperCase();
        else if (key === "Escape") {
          // Cancel — re-register current hotkey, await so dialog doesn't close before IPC completes
          window.removeEventListener("keydown", onKeyDown, true);
          await cancelCapture();
          return;
        }
        parts.push(key);
        const shortcut = parts.join("+");
        localHotkey = shortcut;
        updateHotkey(shortcut);
        onsave({ hotkey: shortcut });
        capturingHotkey = false;
        window.removeEventListener("keydown", onKeyDown, true);
      }

      window.addEventListener("keydown", onKeyDown, true);
    }

    async function cancelCapture() {
      capturingHotkey = false;
      await api.setHotkey(localHotkey); // re-register current hotkey
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
      if (e.key === "Escape" && capturingHotkey) { cancelCapture(); return; }
      if (e.key === "Tab") {
        const dialog = (e.currentTarget as HTMLElement);
        const focusable = dialog.querySelectorAll<HTMLElement>(
          'input, select, button, [tabindex]:not([tabindex="-1"])'
        );
        if (!focusable.length) return;
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
      }
    }

    $effect(() => {
      if (open) requestAnimationFrame(() => document.getElementById("settings-hotkey-display")?.focus());
    });
  </script>

  {#if open}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div class="dialog-backdrop" role="dialog" aria-modal="true" aria-labelledby="settings-dialog-title" onkeydown={handleKeydown}>
      <div class="dialog">
        <h2 id="settings-dialog-title">{m.settings_dialog_title()}</h2>

        <!-- Hotkey section -->
        <div class="field">
          <label for="settings-hotkey-display">{m.settings_hotkey_label()}</label>
          <div class="hotkey-row">
            <input
              id="settings-hotkey-display"
              type="text"
              value={localHotkey}
              readonly
              aria-label={m.settings_hotkey_current_aria({ hotkey: localHotkey })}
              class="hotkey-input"
            />
            <button
              type="button"
              class="btn-small"
              onclick={startHotkeyCapture}
              disabled={capturingHotkey}
            >
              {capturingHotkey ? m.settings_hotkey_capturing() : m.settings_hotkey_capture_btn()}
            </button>
            <button
              type="button"
              class="btn-small btn-secondary"
              onclick={resetHotkey}
              disabled={capturingHotkey}
            >
              {m.settings_hotkey_reset_btn()}
            </button>
          </div>
        </div>

        <!-- Sound toggle -->
        <div class="field">
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={soundEnabled}
              onchange={handleSoundToggle}
            />
            {m.settings_sound_label()}
          </label>
        </div>

        <!-- Confirm exit toggle -->
        <div class="field">
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={confirmExitDuringRecording}
              onchange={handleConfirmExitToggle}
            />
            {m.settings_confirm_exit_label()}
          </label>
        </div>

        <!-- Language selector -->
        <div class="field">
          <label for="settings-language">{m.settings_language_label()}</label>
          <select id="settings-language" value={language} onchange={handleLanguageChange}>
            <option value="en">{m.settings_language_en()}</option>
            <option value="uk">{m.settings_language_uk()}</option>
          </select>
          <p class="hint">{m.settings_language_restart_note()}</p>
        </div>

        <div class="actions">
          <button type="button" class="btn-primary" onclick={async () => { if (capturingHotkey) await cancelCapture(); onclose(); }}>{m.btn_close()}</button>
        </div>
      </div>
    </div>
  {/if}

  <style>
    .dialog-backdrop {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.4);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 100;
    }
    .dialog {
      background: white;
      border-radius: 8px;
      padding: 24px;
      width: 380px;
      box-shadow: 0 4px 24px rgba(0,0,0,0.2);
    }
    h2 { margin: 0 0 16px; font-size: 1.2rem; }
    .field { display: flex; flex-direction: column; gap: 4px; margin-bottom: 16px; }
    .field label { font-weight: 600; font-size: 0.875rem; }
    .field select { padding: 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 0.875rem; }
    .hotkey-row { display: flex; gap: 8px; align-items: center; }
    .hotkey-input { flex: 1; padding: 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 0.875rem; background: #f5f5f5; cursor: default; }
    .checkbox-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 0.875rem; font-weight: normal; }
    .hint { margin: 4px 0 0; font-size: 0.8rem; color: #6b7280; }
    .btn-small { padding: 8px 10px; border: none; border-radius: 4px; font-size: 0.8rem; font-weight: 600; cursor: pointer; background: #6b7280; color: white; white-space: nowrap; }
    .btn-small:hover:not(:disabled) { background: #4b5563; }
    .btn-small:disabled { opacity: 0.6; cursor: not-allowed; }
    .btn-small.btn-secondary { background: #e5e7eb; color: #374151; }
    .btn-small.btn-secondary:hover:not(:disabled) { background: #d1d5db; }
    .actions { display: flex; justify-content: flex-end; margin-top: 8px; }
    .btn-primary { padding: 8px 16px; border: none; border-radius: 4px; font-size: 0.875rem; font-weight: 600; cursor: pointer; background: #2563eb; color: white; }
    .btn-primary:hover { background: #1d4ed8; }
  </style>
  ```

- [ ] **Step 2: Run type check**

  ```bash
  pnpm check
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src/lib/components/SettingsDialog.svelte
  git commit -m "feat: SettingsDialog modal with hotkey capture, sound, confirm-exit, language"
  ```

---

## Task 10: ConfirmExitDialog.svelte

**Files:**
- Create: `src/lib/components/ConfirmExitDialog.svelte`

- [ ] **Step 1: Create the component**

  ```svelte
  <script lang="ts">
    import * as m from "../../paraglide/messages";

    interface Props {
      open: boolean;
      onstopandexit: () => void;
      oncancel: () => void;
    }

    let { open, onstopandexit, oncancel }: Props = $props();

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === "Escape") { oncancel(); return; }
      if (e.key === "Tab") {
        const dialog = e.currentTarget as HTMLElement;
        const focusable = dialog.querySelectorAll<HTMLElement>("button");
        if (!focusable.length) return;
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
      }
    }

    $effect(() => {
      if (open) requestAnimationFrame(() => document.getElementById("confirm-exit-cancel")?.focus());
    });
  </script>

  {#if open}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="dialog-backdrop"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirm-exit-title"
      aria-describedby="confirm-exit-body"
      onkeydown={handleKeydown}
    >
      <div class="dialog">
        <h2 id="confirm-exit-title">{m.confirm_exit_title()}</h2>
        <p id="confirm-exit-body">{m.confirm_exit_body()}</p>
        <div class="actions">
          <button id="confirm-exit-cancel" type="button" class="btn-secondary" onclick={oncancel}>
            {m.btn_cancel()}
          </button>
          <button type="button" class="btn-danger" onclick={onstopandexit}>
            {m.btn_stop_and_exit()}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <style>
    .dialog-backdrop {
      position: fixed;
      inset: 0;
      background: rgba(0,0,0,0.4);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 200;
    }
    .dialog {
      background: white;
      border-radius: 8px;
      padding: 24px;
      width: 340px;
      box-shadow: 0 4px 24px rgba(0,0,0,0.2);
    }
    h2 { margin: 0 0 8px; font-size: 1.1rem; }
    p { margin: 0 0 20px; font-size: 0.9rem; color: #374151; }
    .actions { display: flex; gap: 8px; justify-content: flex-end; }
    .btn-secondary { padding: 8px 16px; border: none; border-radius: 4px; font-size: 0.875rem; font-weight: 600; cursor: pointer; background: #e5e7eb; color: #374151; }
    .btn-secondary:hover { background: #d1d5db; }
    .btn-danger { padding: 8px 16px; border: none; border-radius: 4px; font-size: 0.875rem; font-weight: 600; cursor: pointer; background: #ef4444; color: white; }
    .btn-danger:hover { background: #dc2626; }
  </style>
  ```

- [ ] **Step 2: Run type check**

  ```bash
  pnpm check
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src/lib/components/ConfirmExitDialog.svelte
  git commit -m "feat: ConfirmExitDialog for close-during-recording confirmation"
  ```

---

## Task 11: App.svelte full refactor

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Update imports**

  Replace/add at top of `<script>`:

  ```ts
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import * as api from "./lib/utils/invoke";
  import * as m from "./paraglide/messages";
  import { initLanguage, detectLanguage } from "./lib/i18n";
  import {
    getSettings, loadSettingsFields, updateHotkey,
    updateSoundEnabled, setLanguage, setConfirmExitDuringRecording
  } from "./lib/stores/settings.svelte";
  import SettingsDialog from "./lib/components/SettingsDialog.svelte";
  import ConfirmExitDialog from "./lib/components/ConfirmExitDialog.svelte";

  // Remove imports of: updateHotkey, updateSoundEnabled from recording store
  // Remove: saveSettings import (use api.saveSettings directly)
  ```

- [ ] **Step 2: Replace state declarations**

  Remove `let capturingHotkey = $state(false);` (moved to SettingsDialog).

  Add:
  ```ts
  let settingsOpen = $state(false);
  let confirmExitOpen = $state(false);
  const appSettings = getSettings();
  const appWindow = getCurrentWindow();
  ```

- [ ] **Step 3: Fix scheduleSave — read version from settings store**

  Replace hardcoded `version: 2` with `version: appSettings.version`:

  ```ts
  function scheduleSave() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      await api.saveSettings({
        version: appSettings.version,          // was hardcoded 2
        selectedMic: recording.selectedMic,
        selectedLoopback: recording.selectedLoopback,
        hotkey: appSettings.hotkey,            // from settings store
        soundEnabled: appSettings.soundEnabled, // from settings store
        language: appSettings.language,
        confirmExitDuringRecording: appSettings.confirmExitDuringRecording,
        profiles: profileStore.list,
        activeProfileId: profileStore.activeId,
      });
    }, 500);
  }
  ```

- [ ] **Step 4: Update onMount**

  ```ts
  onMount(() => {
    (async () => {
      const rawSettings = await api.loadSettings();
      loadSettingsFields(rawSettings);
      initLanguage(appSettings.language);
      await loadSettingsIntoStore();  // loads mic/loopback
      await loadProfiles();
      const active = profileStore.active;
      if (active) applyProfile(active);
      await refreshDevices();
      await initRecordingListener();
      await initDeviceListener();

      // Register close handler
      await appWindow.onCloseRequested(async (event) => {
        if (appSettings.confirmExitDuringRecording && isRecording) {
          event.preventDefault();
          confirmExitOpen = true;
        }
      });

      initialized = true;
    })();
    window.addEventListener("keydown", handleMnemonic);
    return () => window.removeEventListener("keydown", handleMnemonic);
  });
  ```

- [ ] **Step 5: Add handleSettingsSave and handleConfirmExit**

  ```ts
  function handleSettingsSave(patch: Partial<import("./lib/types").Settings>) {
    // Settings store already updated by SettingsDialog before calling this
    scheduleSave();
  }

  async function handleStopAndExit() {
    confirmExitOpen = false;
    try { await stopRecording(); } catch { /* already stopped */ }
    await appWindow.close();
  }
  ```

- [ ] **Step 6: Update liveRegionText effect to use Paraglide**

  ```ts
  $effect(() => {
    if (!initialized) return;
    const state = recording.state;
    if (state === "Recording") liveRegionText = m.live_recording_started();
    else if (state === "Paused") liveRegionText = m.live_recording_paused();
    else if (state === "Idle") liveRegionText = m.live_recording_stopped();
  });
  ```

  Update Alt+I handler:
  ```ts
  case "i":
    e.preventDefault();
    if (recording.state !== "Idle") {
      liveRegionText = m.live_status_info({
        state: recording.state === "Paused" ? m.status_paused() : m.status_recording(),
        duration: formatDuration(recording.durationMs),
      });
    } else {
      liveRegionText = m.status_ready();
    }
    break;
  ```

- [ ] **Step 7: Update template**

  Replace the `<main>` content. Key changes:

  1. Add ⚙ button near h1:
  ```html
  <div class="header-row">
    <h1>{m.app_title()}</h1>
    <button
      type="button"
      class="btn-settings"
      aria-label={m.settings_btn_aria()}
      onclick={() => settingsOpen = true}
    >⚙</button>
  </div>
  ```

  2. Pass localized labels to DeviceSelect:
  ```html
  <DeviceSelect
    label={m.mic_label()}
    ...
  />
  <DeviceSelect
    label={m.loopback_label()}
    ...
  />
  ```

  3. Pass localized labels to VolumeSlider:
  ```html
  <VolumeSlider label={m.mic_volume_label()} ... />
  <VolumeSlider label={m.loopback_volume_label()} ... />
  ```

  4. Remove the inline `<section aria-label="Settings">` block (hotkey + sound checkboxes) entirely.

  5. Add SettingsDialog and ConfirmExitDialog:
  ```html
  <SettingsDialog
    open={settingsOpen}
    hotkey={appSettings.hotkey}
    soundEnabled={appSettings.soundEnabled}
    confirmExitDuringRecording={appSettings.confirmExitDuringRecording}
    language={appSettings.language}
    onclose={() => settingsOpen = false}
    onsave={handleSettingsSave}
  />

  <ConfirmExitDialog
    open={confirmExitOpen}
    onstopandexit={handleStopAndExit}
    oncancel={() => confirmExitOpen = false}
  />
  ```

  6. Update aria-label on sections:
  ```html
  <section aria-label={m.audio_devices_section()}>
  <section aria-label={m.volume_controls_section()}>
  ```

  7. Update aria-live region:
  ```html
  <div role="status" aria-atomic="true" class="visually-hidden">{liveRegionText}</div>
  ```

  8. Add `.header-row` and `.btn-settings` styles:
  ```css
  .header-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin: 0 0 16px;
  }
  .header-row h1 { margin: 0; }
  .btn-settings {
    background: none;
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    font-size: 1rem;
  }
  .btn-settings:hover { background: #f0f0f0; }
  ```

- [ ] **Step 8: Run type check**

  ```bash
  pnpm check
  ```

  Expected: zero errors.

- [ ] **Step 9: Commit**

  ```bash
  git add src/App.svelte
  git commit -m "feat: wire SettingsDialog, ConfirmExitDialog, onCloseRequested, i18n in App.svelte"
  ```

---

## Task 12: Localize all components

**Files:**
- Modify: `src/lib/components/StatusIndicator.svelte`
- Modify: `src/lib/components/RecordControls.svelte`
- Modify: `src/lib/components/ProfileSelector.svelte`
- Modify: `src/lib/components/ProfileDialog.svelte`
- Modify: `src/lib/components/DeviceSelect.svelte`

- [ ] **Step 1: Localize StatusIndicator.svelte**

  Add import: `import * as m from "../../paraglide/messages";`

  Replace:
  ```ts
  // OLD:
  let stateLabel = $derived(
    state === "Idle" ? "Ready" : state === "Recording" ? "Recording" : "Paused"
  );
  ```

  With:
  ```ts
  let stateLabel = $derived(
    state === "Idle" ? m.status_ready() : state === "Recording" ? m.status_recording() : m.status_paused()
  );
  ```

  Update aria-labels:
  ```html
  <!-- OLD: aria-label="Recording status" -->
  <div class="status-indicator" aria-label={m.recording_status()}>

  <!-- OLD: aria-label="Recording duration: {formattedDuration}" -->
  <span class="duration" role="timer" aria-label={m.recording_duration({ duration: formattedDuration })}>
  ```

- [ ] **Step 2: Localize RecordControls.svelte**

  Add import: `import * as m from "../../paraglide/messages";`

  Remove `altPressed` state and the Alt key listeners (they are no longer needed once underlines are removed). Replace the entire template button text with localized strings. The keyboard shortcuts remain functional via mnemonic handler in `App.svelte` — the visual underlines are removed per YAGNI (aria-labels document the shortcuts):

  ```html
  <div class="record-controls" role="group" aria-label={m.recording_controls()}>
    {#if recordingState === "Idle"}
      <button
        type="button"
        onclick={canRecord ? onstart : undefined}
        aria-disabled={!canRecord || undefined}
        aria-label={canRecord
          ? m.start_recording_aria()
          : m.start_recording_disabled_aria({ hint: readinessHint })}
        class="btn btn-start"
        class:disabled={!canRecord}
      >
        {m.btn_start()}
      </button>
      {#if !canRecord}
        <p id="start-hint" class="hint" role="note">{readinessHint}</p>
      {/if}
    {:else if recordingState === "Recording"}
      <button type="button" onclick={onpause} aria-label={m.pause_recording_aria()} class="btn btn-pause">
        {m.btn_pause()}
      </button>
      <button type="button" onclick={onstop} aria-label={m.stop_recording_aria()} class="btn btn-stop">
        {m.btn_stop()}
      </button>
    {:else if recordingState === "Paused"}
      <button type="button" onclick={onresume} aria-label={m.resume_recording_aria()} class="btn btn-resume">
        {m.btn_resume()}
      </button>
      <button type="button" onclick={onstop} aria-label={m.stop_recording_aria()} class="btn btn-stop">
        {m.btn_stop()}
      </button>
    {/if}
  </div>
  ```

  Also update `readinessHint` — it's a string generated in `recording.svelte.ts`. Update `get readinessHint()` in `recording.svelte.ts` to return message keys and translate them in RecordControls, OR (simpler) import `m` in RecordControls and derive the hint there. Simplest: pass hint key as a type from the store. 

  **Simplest approach**: update `readinessHint` getter in `recording.svelte.ts` to return an enum value, and translate in RecordControls. Or just keep it as a plain English string for now and note it as a follow-up. For MVP, the hint text can remain in English since it's a screen-reader-only hint in the aria-label.

  **Pragmatic approach**: Remove `readinessHint` string from recording store and compute it locally in RecordControls using `m.*()`. Update the `Props` interface to pass `canRecordMic: boolean` and `canRecordLoopback: boolean` booleans instead of a pre-built string:

  In `RecordControls.svelte` Props:
  ```ts
  interface Props {
    recordingState: RecordingState;
    canRecord: boolean;
    needsMic: boolean;        // replaces readinessHint
    needsLoopback: boolean;   // replaces readinessHint
    onstart: () => void;
    onpause: () => void;
    onresume: () => void;
    onstop: () => void;
  }

  let { recordingState, canRecord, needsMic, needsLoopback, onstart, onpause, onresume, onstop }: Props = $props();

  let readinessHint = $derived(
    needsMic && needsLoopback ? m.hint_select_both() :
    needsMic ? m.hint_select_mic() :
    needsLoopback ? m.hint_select_loopback() : ""
  );
  ```

  Update `recording.svelte.ts` `getRecording()` to expose `needsMic` and `needsLoopback` booleans (already computed for `canRecord` and `readinessHint` — just expose them separately). Remove `readinessHint` from the store.

  Update `App.svelte` RecordControls usage:
  ```html
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
  ```

- [ ] **Step 3: Localize ProfileSelector.svelte**

  Add import: `import * as m from "../../paraglide/messages";`

  ```html
  <section aria-label={m.profile_section()}>
    <div class="profile-row">
      <label for="profile-select">{m.profile_label()}</label>
      ...
      <button aria-label={m.create_profile_aria()} ...>+</button>
      <button aria-label={m.edit_profile_aria()} ...>✎</button>
      <button aria-label={m.delete_profile_aria()} ...>✕</button>
    </div>
  </section>
  ```

  The `confirm("Delete this profile?")` call in `App.svelte` `handleProfileDelete` — replace it:
  ```ts
  // In App.svelte handleProfileDelete:
  if (!confirm(m.delete_profile_confirm())) return;
  ```

- [ ] **Step 4: Localize DeviceSelect.svelte**

  Add import: `import * as m from "../../paraglide/messages";`

  Replace hardcoded placeholder option:
  ```html
  <!-- OLD: -->
  <option value="">-- Select {label} --</option>

  <!-- NEW: -->
  <option value="">{m.select_device_placeholder({ label })}</option>
  ```

- [ ] **Step 5: Localize ProfileDialog.svelte**

  Add import: `import * as m from "../../paraglide/messages";`

  Replace all hardcoded strings:

  ```ts
  let title = $derived(isEdit ? m.profile_dialog_edit_title() : m.profile_dialog_create_title());
  ```

  Output mode labels:
  ```ts
  const outputModes: { value: OutputMode; label: string }[] = [
    { value: "Microphone", label: m.mode_microphone() },
    { value: "Loopback", label: m.mode_loopback() },
    { value: "Mix", label: m.mode_mix() },
    { value: "MixPlusMicrophone", label: m.mode_mix_plus_mic() },
    { value: "MixPlusLoopback", label: m.mode_mix_plus_loopback() },
  ];
  ```

  Validation errors:
  ```ts
  if (!name.trim()) { error = m.error_profile_name_required(); return; }
  // ...
  error = m.error_filename_invalid();
  ```

  Field labels in template — replace each `<label>` text with the corresponding `m.*()` call:
  - "Name" → `{m.field_name()}`
  - "Description" → `{m.field_description()}`
  - "Output Folder" → `{m.field_output_folder()}`
  - "Output Mode" → `{m.field_output_mode()}`
  - "Sample Rate" → `{m.field_sample_rate()}`
  - "Mic Volume" → `{m.field_mic_volume()}`
  - "Loopback Volume" → `{m.field_loopback_volume()}`
  - "Mic Filename" → `{m.field_mic_filename()}`
  - "Loopback Filename" → `{m.field_loopback_filename()}`
  - "Mix Filename" → `{m.field_mix_filename()}`
  - "Cancel" → `{m.btn_cancel()}`
  - `isEdit ? "Save" : "Create"` → `isEdit ? m.btn_save() : m.btn_create()`

  Sample rate option: `{rate} Hz` → `{rate} ${m.hz() || 'Hz'}` — simpler: just keep `{rate} Hz` since "Hz" is the same in all languages. Or add `"hz": "Hz"` to both message files.

- [ ] **Step 6: Run type check and fix any remaining errors**

  ```bash
  pnpm check
  ```

- [ ] **Step 7: Commit**

  ```bash
  git add src/lib/components/
  git commit -m "feat: localize all UI components with Paraglide message functions"
  ```

---

## Task 13: Scoop manifest

**Files:**
- Create: `bucket/audiocaptor.json`

- [ ] **Step 1: Create bucket/audiocaptor.json**

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

  > Replace `USERNAME` with the actual GitHub username/org when the repo is public. Fill `hash` with the actual SHA256 from `scoop hash AudioCaptor.exe` at release time. When a `.sha256` sidecar file is available on releases, add to `autoupdate`: `"hash": { "url": "$url.sha256" }`.

  > The `persist` paths must match exactly what `portable::ensure_dirs()` creates: `logs` (lowercase), `Recordings` (capital R).

- [ ] **Step 2: Commit**

  ```bash
  git add bucket/audiocaptor.json
  git commit -m "feat: add Scoop manifest with persist for settings, logs, Recordings"
  ```

---

## Final verification

- [ ] Run full type check: `pnpm check` — zero errors
- [ ] Run Rust tests: `cd src-tauri && cargo test` — all pass
- [ ] Build release: `pnpm build` — succeeds

### Manual smoke tests

| Test | Expected |
|------|---------|
| Launch app | UI shows in English (or Ukrainian if system lang is uk/ru) |
| Open Settings (⚙) | Modal opens with hotkey, sound, confirm-exit, language fields |
| Click "Capture New" | Global hotkey deregisters, "Press a key..." shown. Press F9 → new hotkey registered |
| Press Escape during capture | Capture cancelled, original hotkey re-registered |
| Click "Reset to Pause" | Hotkey reverts to Pause |
| Toggle "Sound notifications" | Setting saves, sound plays/doesn't on recording actions |
| Change language to Українська | Note shown. After restart UI is in Ukrainian |
| Start recording, attempt to close | ConfirmExitDialog appears |
| ConfirmExitDialog → "Stop and exit" | Recording stops, app closes |
| ConfirmExitDialog → "Cancel" | Dialog dismisses, recording continues |
| Close app when not recording | App closes immediately (no dialog) |
| Screen reader (NVDA/JAWS) | Announces "Recording started/paused/stopped" on state change |
