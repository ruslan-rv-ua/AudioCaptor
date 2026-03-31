import { listen } from "@tauri-apps/api/event";
import type { RecordingState, RecordingStateEvent, OutputMode } from "../types";
import * as api from "../utils/invoke";

let recordingState = $state<RecordingState>("Idle");
let durationMs = $state(0);
let selectedMic = $state<string | null>(null);
let selectedLoopback = $state<string | null>(null);
let micVolume = $state(1.0);
let loopbackVolume = $state(0.5);
let outputMode = $state<OutputMode>("Mix");
let sampleRate = $state(48000);
let recordingError = $state<string | null>(null);
let soundEnabled = $state(true);
let hotkey = $state("Pause");

export function getRecording() {
  return {
    get state() { return recordingState; },
    get durationMs() { return durationMs; },
    get selectedMic() { return selectedMic; },
    set selectedMic(v: string | null) { selectedMic = v; },
    get selectedLoopback() { return selectedLoopback; },
    set selectedLoopback(v: string | null) { selectedLoopback = v; },
    get micVolume() { return micVolume; },
    set micVolume(v: number) { micVolume = v; },
    get loopbackVolume() { return loopbackVolume; },
    set loopbackVolume(v: number) { loopbackVolume = v; },
    get outputMode() { return outputMode; },
    set outputMode(v: OutputMode) { outputMode = v; },
    get sampleRate() { return sampleRate; },
    set sampleRate(v: number) { sampleRate = v; },
    get error() { return recordingError; },
    get soundEnabled() { return soundEnabled; },
    set soundEnabled(v: boolean) { soundEnabled = v; },
    get hotkey() { return hotkey; },
    set hotkey(v: string) { hotkey = v; },
  };
}

export async function initRecordingListener() {
  await listen<RecordingStateEvent>("recording-state-changed", (event) => {
    recordingState = event.payload.state;
    durationMs = event.payload.durationMs;
  });

  // Listen for errors from hotkey-triggered actions
  await listen<string>("recording-error", (event) => {
    recordingError = event.payload;
  });
}

export async function startRecording() {
  recordingError = null;
  try {
    await api.startRecording(selectedMic, selectedLoopback, outputMode, sampleRate);
    recordingState = "Recording";
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function pauseRecording() {
  try {
    await api.pauseRecording();
    recordingState = "Paused";
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function resumeRecording() {
  try {
    await api.resumeRecording();
    recordingState = "Recording";
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function stopRecording() {
  try {
    await api.stopRecording();
    recordingState = "Idle";
    durationMs = 0;
  } catch (e) {
    recordingError = e instanceof Error ? e.message : String(e);
  }
}

export async function updateMicVolume(volume: number) {
  micVolume = volume;
  await api.setMicVolume(volume);
}

export async function updateLoopbackVolume(volume: number) {
  loopbackVolume = volume;
  await api.setLoopbackVolume(volume);
}

export async function updateSoundEnabled(enabled: boolean) {
  soundEnabled = enabled;
  await api.setSoundEnabled(enabled);
}

export async function updateHotkey(shortcut: string) {
  hotkey = shortcut;
  await api.setHotkey(shortcut);
}

export async function loadSettingsIntoStore() {
  const s = await api.loadSettings();
  selectedMic = s.selectedMic;
  selectedLoopback = s.selectedLoopback;
  micVolume = s.micVolume;
  loopbackVolume = s.loopbackVolume;
  outputMode = s.outputMode;
  sampleRate = s.sampleRate;
  hotkey = s.hotkey;
  soundEnabled = s.soundEnabled;
}
