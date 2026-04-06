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
