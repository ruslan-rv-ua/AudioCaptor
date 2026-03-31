use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub selected_mic: Option<String>,
    pub selected_loopback: Option<String>,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub output_mode: String,
    pub sample_rate: u32,
    pub hotkey: String,
    pub sound_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_mic: None,
            selected_loopback: None,
            mic_volume: 1.0,
            loopback_volume: 0.5,
            output_mode: "mix".to_string(),
            sample_rate: 48000,
            hotkey: "Pause".to_string(),
            sound_enabled: true,
        }
    }
}

fn settings_path() -> anyhow::Result<PathBuf> {
    Ok(crate::portable::exe_dir()?.join("settings.json"))
}

pub fn read_settings() -> Settings {
    let path = match settings_path() {
        Ok(p) => p,
        Err(_) => return Settings::default(),
    };
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn write_settings(settings: &Settings) -> anyhow::Result<()> {
    let path = settings_path()?;
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json)?;
    Ok(())
}
