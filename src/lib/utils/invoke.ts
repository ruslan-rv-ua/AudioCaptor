import { invoke } from "@tauri-apps/api/core";
import type { AudioDevice, OutputMode, RecordingProfile, Settings } from "../types";

export async function getAudioDevices(): Promise<AudioDevice[]> {
  return invoke<AudioDevice[]>("get_audio_devices");
}

export async function startRecording(
  micId: string | null,
  loopbackId: string | null,
  mode: OutputMode,
  sampleRate: number
): Promise<void> {
  return invoke("start_recording", {
    micId,
    loopbackId,
    mode,
    sampleRate,
  });
}

export async function pauseRecording(): Promise<void> {
  return invoke("pause_recording");
}

export async function resumeRecording(): Promise<void> {
  return invoke("resume_recording");
}

export async function stopRecording(): Promise<void> {
  return invoke("stop_recording");
}

export async function setMicVolume(volume: number): Promise<void> {
  return invoke("set_mic_volume", { volume });
}

export async function setLoopbackVolume(volume: number): Promise<void> {
  return invoke("set_loopback_volume", { volume });
}

export async function loadSettings(): Promise<Settings> {
  return invoke<Settings>("load_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function setSoundEnabled(enabled: boolean): Promise<void> {
  return invoke("set_sound_enabled", { enabled });
}

export async function setHotkey(shortcut: string): Promise<void> {
  return invoke("set_hotkey", { shortcut });
}

export async function unregisterHotkey(): Promise<void> {
  return invoke("unregister_hotkey");
}

export async function cmdListProfiles(): Promise<RecordingProfile[]> {
  return invoke<RecordingProfile[]>("cmd_list_profiles");
}

export async function cmdSaveProfile(profile: RecordingProfile): Promise<void> {
  return invoke("cmd_save_profile", { profile });
}

export async function cmdDeleteProfile(id: string): Promise<string> {
  return invoke<string>("cmd_delete_profile", { id });
}

export async function cmdSelectProfile(id: string): Promise<void> {
  return invoke("cmd_select_profile", { id });
}

export async function cmdGetActiveProfile(): Promise<RecordingProfile> {
  return invoke<RecordingProfile>("cmd_get_active_profile");
}

export async function getRecordingsDir(): Promise<string> {
  return invoke<string>("get_recordings_dir");
}

export async function refreshDevices(): Promise<AudioDevice[]> {
  return invoke<AudioDevice[]>("refresh_devices");
}

export async function quitApp(): Promise<void> {
  return invoke("quit_app");
}
