use crate::audio::types::OutputMode;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub selected_mic: Option<String>,
    pub selected_loopback: Option<String>,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub output_mode: OutputMode,
    pub sample_rate: u32,
    pub hotkey: String,
    pub sound_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 0,
            selected_mic: None,
            selected_loopback: None,
            mic_volume: 1.0,
            loopback_volume: 0.5,
            output_mode: OutputMode::Mix,
            sample_rate: 48000,
            hotkey: "Pause".to_string(),
            sound_enabled: true,
        }
    }
}

fn settings_path() -> anyhow::Result<PathBuf> {
    Ok(crate::portable::exe_dir()?.join("settings.json"))
}

fn migrate_settings(mut settings: Settings) -> Settings {
    match settings.version {
        0 => {
            // Old settings file without version field — upgrade to v1
            settings.version = 1;
            log::info!("Migrated settings from v0 to v1");
        }
        1 => { /* current version */ }
        v => log::warn!("Unknown settings version: {v}"),
    }
    settings
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
    let settings: Settings = serde_json::from_str(&contents).unwrap_or_default();
    migrate_settings(settings)
}

pub fn write_settings(settings: &Settings) -> anyhow::Result<()> {
    let path = settings_path()?;
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_serialization_roundtrip() {
        let mut settings = Settings::default();
        settings.version = 1; // simulate post-migration state
        let json = serde_json::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.mic_volume, 1.0);
        assert_eq!(deserialized.loopback_volume, 0.5);
        assert_eq!(deserialized.output_mode, OutputMode::Mix);
        assert_eq!(deserialized.sample_rate, 48000);
        assert_eq!(deserialized.hotkey, "Pause");
        assert!(deserialized.sound_enabled);
    }

    #[test]
    fn settings_missing_version_defaults_to_zero() {
        let json = r#"{
            "selectedMic": null,
            "selectedLoopback": null,
            "micVolume": 1.0,
            "loopbackVolume": 0.5,
            "outputMode": "Mix",
            "sampleRate": 48000,
            "hotkey": "Pause",
            "soundEnabled": true
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.version, 0);
    }

    #[test]
    fn migrate_settings_upgrades_v0_to_v1() {
        let mut settings = Settings::default();
        settings.version = 0;
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 1);
    }

    #[test]
    fn migrate_settings_keeps_v1_unchanged() {
        let mut settings = Settings::default();
        settings.version = 1;
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 1);
    }

    #[test]
    fn settings_output_mode_serializes_as_pascal_case() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"Mix\""), "outputMode should serialize as PascalCase");
    }
}
