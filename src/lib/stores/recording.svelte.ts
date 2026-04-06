import { listen } from "@tauri-apps/api/event";
import type { RecordingState, RecordingStateEvent, OutputMode, RecordingProfile } from "../types";
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
    get canRecord() {
      const mode = outputMode;
      const needsMic = mode === "Microphone" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback";
      const needsLoopback = mode === "Loopback" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback";
      if (needsMic && !selectedMic) return false;
      if (needsLoopback && !selectedLoopback) return false;
      return true;
    },
    get needsMic() { return !selectedMic; },
    get needsLoopback() { return !selectedLoopback; },
    get readinessHint() {
      const mode = outputMode;
      const needsMic = (mode === "Microphone" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback") && !selectedMic;
      const needsLoopback = (mode === "Loopback" || mode === "Mix" || mode === "MixPlusMicrophone" || mode === "MixPlusLoopback") && !selectedLoopback;
      if (needsMic && needsLoopback) return "Select microphone and system audio device";
      if (needsMic) return "Select a microphone";
      if (needsLoopback) return "Select a system audio device";
      return "";
    },
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
