#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum OutputMode {
    Microphone,
    Loopback,
    #[default]
    Mix,
    MixPlusMicrophone,
    MixPlusLoopback,
    SeparateFiles,
}

impl OutputMode {
    pub fn needs_mic(&self) -> bool {
        matches!(
            self,
            OutputMode::Microphone
                | OutputMode::Mix
                | OutputMode::MixPlusMicrophone
                | OutputMode::MixPlusLoopback
                | OutputMode::SeparateFiles
        )
    }

    pub fn needs_loopback(&self) -> bool {
        matches!(
            self,
            OutputMode::Loopback
                | OutputMode::Mix
                | OutputMode::MixPlusMicrophone
                | OutputMode::MixPlusLoopback
                | OutputMode::SeparateFiles
        )
    }
}

/// The file format a profile records to: one entry of the format list in the
/// profile dialog (decision-6). Each variant stands for fixed encoder settings.
///
/// A profile stores the id as plain text (`RecordingProfile::output_format`),
/// so a value written by a newer version survives in `settings.json`; this
/// version then records WAV.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Wav,
    OpusSpeech,
    OpusMusic,
}

impl OutputFormat {
    /// The id stored in `settings.json`.
    pub fn id(self) -> &'static str {
        match self {
            OutputFormat::Wav => "Wav",
            OutputFormat::OpusSpeech => "OpusSpeech",
            OutputFormat::OpusMusic => "OpusMusic",
        }
    }

    /// The format with this id, or `None` for an id this version does not know.
    pub fn from_id(id: &str) -> Option<Self> {
        [
            OutputFormat::Wav,
            OutputFormat::OpusSpeech,
            OutputFormat::OpusMusic,
        ]
        .into_iter()
        .find(|format| format.id() == id)
    }

    /// The name of the format in messages, e.g. "Converting to Opus".
    pub fn family(self) -> &'static str {
        match self {
            OutputFormat::Wav => "WAV",
            OutputFormat::OpusSpeech | OutputFormat::OpusMusic => "Opus",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool, // true = mic, false = output (for loopback)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_mode_mix_plus_microphone_serializes() {
        let json = serde_json::to_string(&OutputMode::MixPlusMicrophone).unwrap();
        assert_eq!(json, "\"MixPlusMicrophone\"");
    }

    #[test]
    fn output_mode_mix_plus_loopback_serializes() {
        let json = serde_json::to_string(&OutputMode::MixPlusLoopback).unwrap();
        assert_eq!(json, "\"MixPlusLoopback\"");
    }

    #[test]
    fn output_mode_mix_plus_microphone_roundtrip() {
        let mode = OutputMode::MixPlusMicrophone;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: OutputMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, OutputMode::MixPlusMicrophone);
    }

    #[test]
    fn output_mode_mix_plus_loopback_roundtrip() {
        let mode = OutputMode::MixPlusLoopback;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: OutputMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, OutputMode::MixPlusLoopback);
    }

    #[test]
    fn output_mode_separate_files_serializes() {
        let json = serde_json::to_string(&OutputMode::SeparateFiles).unwrap();
        assert_eq!(json, "\"SeparateFiles\"");
    }

    #[test]
    fn output_mode_separate_files_roundtrip() {
        let mode = OutputMode::SeparateFiles;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: OutputMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, OutputMode::SeparateFiles);
    }
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Pause,
    Resume,
    Stop,
    SetMicVolume(f32),
    SetLoopbackVolume(f32),
}
