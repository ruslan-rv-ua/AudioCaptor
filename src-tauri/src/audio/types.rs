#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum OutputMode {
    Microphone,
    Loopback,
    #[default]
    Mix,
    MixPlusMicrophone,
    MixPlusLoopback,
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
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Pause,
    Resume,
    Stop,
    SetMicVolume(f32),
    SetLoopbackVolume(f32),
}
