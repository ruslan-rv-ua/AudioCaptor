import { listen } from "@tauri-apps/api/event";
import type {
  RecordingState,
  RecordingStateEvent,
  OutputMode,
  QueuedConversion,
  RecordingProfile,
} from "../types";
import * as api from "../utils/invoke";

let recordingState = $state<RecordingState>("Idle");
let durationMs = $state(0);
// What the recording that stopped last is converted to: nothing for WAV. It is
// set just before the state turns Idle, so the announcement of the stop can
// say it.
let stoppedConversion = $state<QueuedConversion>({ files: 0, ahead: 0, format: null });
let selectedMic = $state<string | null>(null);
let selectedLoopback = $state<string | null>(null);
let micVolume = $state(1.0);
let loopbackVolume = $state(0.5);
let outputMode = $state<OutputMode>("Mix");
let sampleRate = $state(48000);
let recordingError = $state<string | null>(null);

export function getRecording() {
  return {
    get state() { return recordingState; },
    get durationMs() { return durationMs; },
    get stoppedConversion() { return stoppedConversion; },
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
    get canRecord() {
      const mode = outputMode;
      const needsMic = mode === "Microphone" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback" || mode === "SeparateFiles";
      const needsLoopback = mode === "Loopback" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback" || mode === "SeparateFiles";
      if (needsMic && !selectedMic) return false;
      if (needsLoopback && !selectedLoopback) return false;
      return true;
    },
    get needsMic() {
      return (outputMode === "Microphone" || outputMode === "Mix" || outputMode === "MixPlusMicrophone" || outputMode === "MixPlusLoopback" || outputMode === "SeparateFiles")
        && !selectedMic;
    },
    get needsLoopback() {
      return (outputMode === "Loopback" || outputMode === "Mix" || outputMode === "MixPlusMicrophone" || outputMode === "MixPlusLoopback" || outputMode === "SeparateFiles")
        && !selectedLoopback;
    },
    get readinessHint() {
      const mode = outputMode;
      const needsMic = (mode === "Microphone" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback" || mode === "SeparateFiles") && !selectedMic;
      const needsLoopback = (mode === "Loopback" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback" || mode === "SeparateFiles") && !selectedLoopback;
      if (needsMic && needsLoopback) return "Select microphone and system audio device";
      if (needsMic) return "Select a microphone";
      if (needsLoopback) return "Select a system audio device";
      return "";
    },
  };
}

export async function initRecordingListener() {
  await listen<RecordingStateEvent>("recording-state-changed", (event) => {
    // A stop by the global hotkey or the tray menu is known only from here.
    if (event.payload.conversion) stoppedConversion = event.payload.conversion;
    recordingState = event.payload.state;
    durationMs = event.payload.durationMs;
  });

  // Listen for errors from hotkey-triggered actions
  await listen<string>("recording-error", (event) => {
    recordingError = event.payload;
  });
}

// Shows a message in the error block, which is a live region that NVDA reads.
export function showError(message: string) {
  recordingError = message;
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
    stoppedConversion = await api.stopRecording();
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

export function applyProfile(profile: RecordingProfile) {
  micVolume = profile.micVolume;
  loopbackVolume = profile.loopbackVolume;
  outputMode = profile.outputMode;
  sampleRate = profile.sampleRate;
  // Send volume updates to backend
  api.setMicVolume(profile.micVolume);
  api.setLoopbackVolume(profile.loopbackVolume);
}

export async function loadSettingsIntoStore() {
  const s = await api.loadSettings();
  selectedMic = s.selectedMic;
  selectedLoopback = s.selectedLoopback;
  // Profile-specific fields (micVolume, loopbackVolume, outputMode, sampleRate)
  // are loaded via applyProfile() after loadProfiles() in App.svelte onMount
}
