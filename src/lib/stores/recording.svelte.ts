import { listen } from "@tauri-apps/api/event";
import type {
  RecordingState,
  RecordingStateEvent,
  OutputMode,
  QueuedConversion,
  RecordingProfile,
} from "../types";
import * as api from "../utils/invoke";
import * as m from "../../paraglide/messages";

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

// The modes that record from the microphone, and from system audio.
const MIC_MODES: OutputMode[] = ["Microphone", "Mix", "MixPlusMicrophone", "MixPlusLoopback", "SeparateFiles"];
const LOOPBACK_MODES: OutputMode[] = ["Loopback", "Mix", "MixPlusMicrophone", "MixPlusLoopback", "SeparateFiles"];

function readinessHint(): string {
  const needsMic = MIC_MODES.includes(outputMode) && !selectedMic;
  const needsLoopback = LOOPBACK_MODES.includes(outputMode) && !selectedLoopback;
  if (needsMic && needsLoopback) return m.hint_select_both();
  if (needsMic) return m.hint_select_mic();
  if (needsLoopback) return m.hint_select_loopback();
  return "";
}

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
    get canRecord() { return readinessHint() === ""; },
    // Why a recording cannot start: the device the profile needs and that is
    // not selected. The disabled Start button carries it in its name, and Alt+S
    // announces it. Empty when a recording can start.
    get readinessHint() { return readinessHint(); },
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
