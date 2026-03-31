import { invoke } from "@tauri-apps/api/core";
import type { AudioDevice, OutputMode, Settings } from "../types";

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
