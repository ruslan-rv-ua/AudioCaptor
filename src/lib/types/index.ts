export interface AudioDevice {
  id: string;
  name: string;
  isInput: boolean;
}

export type RecordingState = "Idle" | "Recording" | "Paused";

export type OutputMode = "Microphone" | "Loopback" | "Mix" | "MixPlusMicrophone" | "MixPlusLoopback" | "SeparateFiles";

/** The formats this version records in: the entries of the profile's format list. */
export type OutputFormat = "Wav" | "OpusSpeech" | "OpusMusic" | "Mp3Speech" | "Mp3Standard" | "Mp3High" | "Flac";

export type Theme = "auto" | "light" | "dark";

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
  /**
   * An `OutputFormat`, or a format written by a newer version. A format this
   * version does not know stays in the profile and records WAV.
   */
  outputFormat: string;
}

export interface Settings {
  version: number;
  selectedMic: string | null;
  selectedLoopback: string | null;
  hotkey: string | null;
  soundEnabled: boolean;
  profiles: RecordingProfile[];
  activeProfileId: string;
  language: "en" | "uk";
  confirmExitDuringRecording: boolean;
  minimizeToTrayOnFocusLoss: boolean;
  theme: Theme;
}

export interface RecordingStateEvent {
  state: RecordingState;
  durationMs: number;
  /** Sent with `Idle`: what happens to the recording that has just stopped. */
  conversion?: QueuedConversion;
}

/** What stopping a recording put into the conversion queue. */
export interface QueuedConversion {
  /** Files of the recording that will be converted; 0 for a WAV profile. */
  files: number;
  /** Files that were already waiting to be converted before them. */
  ahead: number;
  /** The format they are converted to, e.g. "Opus". */
  format: string | null;
}

/** Where the conversion queue stands (the `conversion-status` event). */
export interface ConversionStatus {
  /** Files not yet converted, the one being converted included. */
  files: number;
  /** Files waiting behind the one being converted. */
  waiting: number;
  /** The format the next file is converted to, e.g. "Opus". */
  format: string | null;
  /** How much of the file being converted is done, 0 to 1. */
  progress: number;
  /** Seconds until the queue is empty, once there is something to go by. */
  etaSeconds: number | null;
}

/** A recording that could not be converted (the `conversion-failed` event). */
export interface ConversionFailure {
  /** The WAV file, which is kept. */
  wav: string;
  format: string;
  /** Windows has no encoder for the format: an N edition without the Media Feature Pack. */
  noEncoder: boolean;
}
