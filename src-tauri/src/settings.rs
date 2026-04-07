use crate::audio::types::OutputMode;
use crate::profiles::RecordingProfile;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub selected_mic: Option<String>,
    pub selected_loopback: Option<String>,
    pub hotkey: Option<String>,
    pub sound_enabled: bool,
    pub profiles: Vec<RecordingProfile>,
    pub active_profile_id: String,
    pub language: String,                        // NEW in v3
    pub confirm_exit_during_recording: bool,     // NEW in v3
    #[serde(default = "default_theme")]
    pub theme: String,              // NEW in v4 — "auto" | "light" | "dark"
    #[serde(default)]
    pub minimize_to_tray_on_focus_loss: bool,  // NEW in v5
    // Legacy fields — used only during migration from v1
    #[serde(default, skip_serializing)]
    mic_volume: f32,
    #[serde(default, skip_serializing)]
    loopback_volume: f32,
    #[serde(default, skip_serializing)]
    output_mode: OutputMode,
    #[serde(default = "default_sample_rate", skip_serializing)]
    sample_rate: u32,
}

fn default_sample_rate() -> u32 {
    48000
}

fn default_theme() -> String {
    "auto".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 5,
            selected_mic: None,
            selected_loopback: None,
            hotkey: Some("Pause".to_string()),
            sound_enabled: true,
            profiles: vec![RecordingProfile::default()],
            active_profile_id: "default".to_string(),
            language: "en".to_string(),
            confirm_exit_during_recording: true,
            theme: "auto".to_string(),
            minimize_to_tray_on_focus_loss: false,
            mic_volume: 1.0,
            loopback_volume: 0.5,
            output_mode: OutputMode::Mix,
            sample_rate: 48000,
        }
    }
}

fn settings_path() -> anyhow::Result<PathBuf> {
    Ok(crate::portable::exe_dir()?.join("settings.json"))
}

pub(crate) fn migrate_settings(mut settings: Settings) -> Settings {
    loop {
        match settings.version {
            0 => {
                settings.version = 1;
                log::info!("Migrated settings from v0 to v1");
            }
            1 => {
                // Migrate flat fields into a default profile
                let profile = RecordingProfile {
                    id: "default".to_string(),
                    name: "Default".to_string(),
                    description: String::new(),
                    output_folder: PathBuf::from("Recordings"),
                    output_mode: settings.output_mode.clone(),
                    sample_rate: settings.sample_rate,
                    mic_volume: settings.mic_volume,
                    loopback_volume: settings.loopback_volume,
                    mic_filename: "mic".to_string(),
                    loopback_filename: "loopback".to_string(),
                    mix_filename: "mix".to_string(),
                };
                settings.profiles = vec![profile];
                settings.active_profile_id = "default".to_string();
                settings.version = 2;
                log::info!("Migrated settings from v1 to v2 (profiles)");
            }
            2 => {
                // Ensure at least one profile exists
                if settings.profiles.is_empty() {
                    settings.profiles = vec![RecordingProfile::default()];
                    settings.active_profile_id = "default".to_string();
                }
                settings.version = 3;
                // No break — loop continues to v3 arm below
            }
            3 => {
                // theme added in v4; serde Default fills "auto" for existing files
                settings.version = 4;
                // fall through to v4 arm
            }
            4 => {
                settings.version = 5;
                // serde #[serde(default)] fills minimize_to_tray_on_focus_loss = false for old files
            }
            5 => {
                break; // terminal version
            }
            v => {
                log::warn!("Unknown settings version: {v}");
                break;
            }
        }
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
    use crate::profiles::RecordingProfile;

    #[test]
    fn settings_serialization_roundtrip() {
        let mut settings = Settings::default();
        settings.version = 2;
        let json = serde_json::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, 2);
        assert_eq!(deserialized.profiles.len(), 1);
        assert_eq!(deserialized.profiles[0].mic_volume, 1.0);
        assert_eq!(deserialized.profiles[0].loopback_volume, 0.5);
        assert_eq!(deserialized.profiles[0].output_mode, OutputMode::Mix);
        assert_eq!(deserialized.profiles[0].sample_rate, 48000);
        assert_eq!(deserialized.hotkey, Some("Pause".to_string()));
        assert!(deserialized.sound_enabled);
        assert_eq!(deserialized.language, "en");
        assert!(deserialized.confirm_exit_during_recording);
        assert_eq!(deserialized.theme, "auto");
    }

    #[test]
    fn settings_missing_version_defaults_to_struct_default() {
        // When "version" is absent from JSON, serde fills it with Settings::default().version
        // (because #[serde(default)] on the struct uses Default::default() to seed missing fields).
        // Since Default::version is now 5, the deserialized value is 5.
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
        assert_eq!(settings.version, 5);
    }

    #[test]
    fn migrate_settings_upgrades_v0_to_v5() {
        let mut settings = Settings::default();
        settings.version = 0;
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert_eq!(migrated.profiles.len(), 1);
    }

    #[test]
    fn migrate_settings_upgrades_v1_to_v5() {
        let mut settings = Settings::default();
        settings.version = 1;
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert_eq!(migrated.profiles.len(), 1);
        assert_eq!(migrated.active_profile_id, "default");
    }

    #[test]
    fn settings_output_mode_serializes_via_profile() {
        let settings = Settings::default();
        assert_eq!(settings.profiles[0].output_mode, OutputMode::Mix);
        let json = serde_json::to_string(&settings.profiles[0]).unwrap();
        assert!(
            json.contains("\"Mix\""),
            "outputMode should serialize as PascalCase"
        );
    }

    #[test]
    fn settings_v2_has_profiles_and_active_id() {
        let settings = Settings::default();
        assert_eq!(settings.version, 5);
        assert_eq!(settings.profiles.len(), 1);
        assert_eq!(settings.active_profile_id, "default");
        assert_eq!(settings.profiles[0].name, "Default");
    }

    #[test]
    fn migrate_v1_to_v2_creates_default_profile_from_flat_fields() {
        let json = r#"{
            "version": 1,
            "selectedMic": "mic-123",
            "selectedLoopback": "loop-456",
            "micVolume": 0.8,
            "loopbackVolume": 0.3,
            "outputMode": "Loopback",
            "sampleRate": 44100,
            "hotkey": "F9",
            "soundEnabled": false
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert_eq!(migrated.profiles.len(), 1);
        let p = &migrated.profiles[0];
        assert_eq!(p.id, "default");
        assert_eq!(p.mic_volume, 0.8);
        assert_eq!(p.loopback_volume, 0.3);
        assert_eq!(p.output_mode, OutputMode::Loopback);
        assert_eq!(p.sample_rate, 44100);
        assert_eq!(migrated.active_profile_id, "default");
        assert_eq!(migrated.selected_mic, Some("mic-123".into()));
        assert_eq!(migrated.hotkey, Some("F9".into()));
        assert!(!migrated.sound_enabled);
    }

    #[test]
    fn migrate_v0_to_v5_goes_through_all_steps() {
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
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert_eq!(migrated.profiles.len(), 1);
    }

    #[test]
    fn settings_v2_roundtrip_preserves_profiles() {
        let mut settings = Settings::default();
        settings.version = 2;
        settings.profiles = vec![
            RecordingProfile::default(),
            RecordingProfile {
                id: "custom".into(),
                name: "Podcast".into(),
                ..RecordingProfile::default()
            },
        ];
        settings.active_profile_id = "custom".into();
        let json = serde_json::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.profiles.len(), 2);
        assert_eq!(deserialized.active_profile_id, "custom");
        assert_eq!(deserialized.profiles[1].name, "Podcast");
    }

    #[test]
    fn migrate_v2_to_v5_adds_language_and_confirm_exit() {
        let json = r#"{
            "version": 2,
            "selectedMic": null,
            "selectedLoopback": null,
            "hotkey": "Pause",
            "soundEnabled": true,
            "profiles": [],
            "activeProfileId": "default"
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert_eq!(migrated.language, "en");
        assert!(migrated.confirm_exit_during_recording);
    }

    #[test]
    fn migrate_v3_to_v5_adds_theme() {
        let json = r#"{
            "version": 3,
            "selectedMic": null,
            "selectedLoopback": null,
            "hotkey": "Pause",
            "soundEnabled": true,
            "profiles": [],
            "activeProfileId": "default",
            "language": "en",
            "confirmExitDuringRecording": true
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert_eq!(migrated.theme, "auto");
    }

    #[test]
    fn settings_default_has_version_5() {
        let s = Settings::default();
        assert_eq!(s.version, 5);
        assert!(!s.minimize_to_tray_on_focus_loss);
    }

    #[test]
    fn migrate_v4_to_v5_adds_minimize_to_tray_on_focus_loss() {
        let json = r#"{
            "version": 4,
            "selectedMic": null,
            "selectedLoopback": null,
            "hotkey": "Pause",
            "soundEnabled": true,
            "profiles": [],
            "activeProfileId": "default",
            "language": "en",
            "confirmExitDuringRecording": true,
            "theme": "auto"
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert!(!migrated.minimize_to_tray_on_focus_loss);
    }

    #[test]
    fn migrate_v4_preserves_existing_minimize_to_tray_false() {
        // If somehow the field is already present, serde preserves it
        let json = r#"{
            "version": 4,
            "selectedMic": null,
            "selectedLoopback": null,
            "hotkey": "Pause",
            "soundEnabled": true,
            "profiles": [],
            "activeProfileId": "default",
            "language": "en",
            "confirmExitDuringRecording": true,
            "theme": "auto",
            "minimizeToTrayOnFocusLoss": false
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 5);
        assert!(!migrated.minimize_to_tray_on_focus_loss);
    }
}
