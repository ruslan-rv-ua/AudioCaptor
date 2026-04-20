import type { Settings, Theme } from "../types";
import * as api from "../utils/invoke";

let language = $state<"en" | "uk">("en");
let confirmExitDuringRecording = $state(true);
let hotkey = $state<string | null>("Pause");
let soundEnabled = $state(true);
let settingsVersion = $state(5);
let theme = $state<Theme>("auto");
let minimizeToTrayOnFocusLoss = $state(false);

export function getSettings() {
  return {
    get language() { return language; },
    get confirmExitDuringRecording() { return confirmExitDuringRecording; },
    get hotkey() { return hotkey; },
    get soundEnabled() { return soundEnabled; },
    get version() { return settingsVersion; },
    get theme() { return theme; },
    get minimizeToTrayOnFocusLoss() { return minimizeToTrayOnFocusLoss; },
  };
}

export function loadSettingsFields(s: Settings) {
  language = (s.language as "en" | "uk") ?? "en";
  confirmExitDuringRecording = s.confirmExitDuringRecording ?? true;
  hotkey = s.hotkey;
  soundEnabled = s.soundEnabled;
  settingsVersion = s.version;
  theme = s.theme ?? "auto";
  minimizeToTrayOnFocusLoss = s.minimizeToTrayOnFocusLoss ?? false;
}

export async function updateHotkey(shortcut: string | null) {
  hotkey = shortcut;
  if (shortcut === null) {
    await api.unregisterHotkey();
  } else {
    await api.setHotkey(shortcut);
  }
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

export function setMinimizeToTrayOnFocusLoss(value: boolean) {
  minimizeToTrayOnFocusLoss = value;
}
