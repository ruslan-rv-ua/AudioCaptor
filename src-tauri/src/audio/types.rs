#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum OutputMode {
    Microphone,
    Loopback,
    Mix,
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

#[derive(Debug)]
pub enum AudioCommand {
    Pause,
    Resume,
    Stop,
    SetMicVolume(f32),
    SetLoopbackVolume(f32),
}
