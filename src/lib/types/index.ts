export interface AudioDevice {
  id: string;
  name: string;
  isInput: boolean;
}

export type RecordingState = "Idle" | "Recording" | "Paused";

export type OutputMode = "Microphone" | "Loopback" | "Mix";

export interface Settings {
  version: number;
  selectedMic: string | null;
  selectedLoopback: string | null;
  micVolume: number;
  loopbackVolume: number;
  outputMode: OutputMode;
  sampleRate: number;
  hotkey: string;
  soundEnabled: boolean;
}

export interface RecordingStateEvent {
  state: RecordingState;
  durationMs: number;
}
