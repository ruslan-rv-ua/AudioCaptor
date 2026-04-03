export interface AudioDevice {
  id: string;
  name: string;
  isInput: boolean;
}

export type RecordingState = "Idle" | "Recording" | "Paused";

export type OutputMode = "Microphone" | "Loopback" | "Mix" | "MixPlusMicrophone" | "MixPlusLoopback";

export interface RecordingProfile {
  id: string;
  name: string;
  description: string;
  outputFolder: string;
  outputMode: OutputMode;
  sampleRate: number;
  micVolume: number;
  loopbackVolume: number;
  micFilename: string;
  loopbackFilename: string;
  mixFilename: string;
}

export interface Settings {
  version: number;
  selectedMic: string | null;
  selectedLoopback: string | null;
  hotkey: string;
  soundEnabled: boolean;
  profiles: RecordingProfile[];
  activeProfileId: string;
}

export interface RecordingStateEvent {
  state: RecordingState;
  durationMs: number;
}
