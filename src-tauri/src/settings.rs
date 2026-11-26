use crate::audio::types::{OutputFormat, OutputMode};
use crate::profiles::RecordingProfile;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
    pub language: String,                    // NEW in v3
    pub confirm_exit_during_recording: bool, // NEW in v3
    #[serde(default = "default_theme")]
    pub theme: String, // NEW in v4 — "auto" | "light" | "dark"
    #[serde(default)]
    pub minimize_to_tray_on_focus_loss: bool, // NEW in v5
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
            version: 6,
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
    Ok(crate::portable::data_dir()?.join("settings.json"))
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
                    output_format: OutputFormat::Wav.id().to_string(),
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
                // outputFormat added to each profile in v6; serde fills "Wav"
                // for profiles written by older versions
                settings.version = 6;
            }
            6 => {
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

/// What the app found in `settings.json` at startup.
pub struct StartupSettings {
    pub settings: Settings,
    /// Set when the file existed but could not be read. Holds where the old file
    /// is kept: the copy next to it, or the file itself when the copy failed.
    pub unreadable_kept_at: Option<PathBuf>,
    /// False when the old file must stay untouched (the copy failed).
    pub write_back: bool,
}

/// Reads the settings at startup. An unreadable file (corrupt JSON, an empty
/// file, a field of an unexpected type) is copied to `settings.unreadable.json`
/// before the caller writes the defaults over it. An existing copy is never
/// overwritten: the next ones are `settings.unreadable-2.json`, `-3` and so on.
pub fn read_settings_at_startup() -> StartupSettings {
    match crate::portable::data_dir() {
        Ok(dir) => read_settings_from(&dir),
        Err(_) => StartupSettings {
            settings: Settings::default(),
            unreadable_kept_at: None,
            write_back: false,
        },
    }
}

fn read_settings_from(dir: &Path) -> StartupSettings {
    let path = dir.join("settings.json");
    let parsed = match fs::read(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return StartupSettings {
                settings: Settings::default(),
                unreadable_kept_at: None,
                write_back: true,
            };
        }
        Err(e) => Err(e.to_string()),
        Ok(bytes) => String::from_utf8(bytes)
            .map_err(|e| e.to_string())
            .and_then(|text| serde_json::from_str::<Settings>(&text).map_err(|e| e.to_string())),
    };
    match parsed {
        Ok(settings) => StartupSettings {
            settings: migrate_settings(settings),
            unreadable_kept_at: None,
            write_back: true,
        },
        Err(reason) => {
            log::error!("Could not read settings.json: {reason}");
            let settings = Settings {
                language: crate::portable::detect_language().to_string(),
                ..Settings::default()
            };
            match keep_unreadable_copy(dir, &path) {
                Ok(copy) => {
                    log::warn!("Kept the unreadable settings as {}", copy.display());
                    StartupSettings {
                        settings,
                        unreadable_kept_at: Some(copy),
                        write_back: true,
                    }
                }
                Err(e) => {
                    log::error!("Could not copy the unreadable settings.json: {e}");
                    StartupSettings {
                        settings,
                        unreadable_kept_at: Some(path),
                        write_back: false,
                    }
                }
            }
        }
    }
}

fn keep_unreadable_copy(dir: &Path, path: &Path) -> std::io::Result<PathBuf> {
    let mut n = 1;
    let target = loop {
        let name = if n == 1 {
            "settings.unreadable.json".to_string()
        } else {
            format!("settings.unreadable-{n}.json")
        };
        let candidate = dir.join(name);
        if !candidate.exists() {
            break candidate;
        }
        n += 1;
    };
    fs::copy(path, &target)?;
    Ok(target)
}

/// Writes `contents` to a temporary file next to `path` and replaces `path`
/// with it, so an interrupted write never leaves a half-written file.
pub(crate) fn write_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

pub fn write_settings(settings: &Settings) -> anyhow::Result<()> {
    let path = settings_path()?;
    let json = serde_json::to_string_pretty(settings)?;
    write_atomic(&path, &json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::RecordingProfile;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "audiocaptor-settings-test-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn assert_unreadable_is_kept(name: &str, contents: &str) {
        let dir = test_dir(name);
        fs::write(dir.join("settings.json"), contents).unwrap();
        let loaded = read_settings_from(&dir);
        let kept = loaded.unreadable_kept_at.expect("file reported unreadable");
        assert_eq!(kept, dir.join("settings.unreadable.json"));
        assert_eq!(fs::read_to_string(&kept).unwrap(), contents);
        assert!(loaded.write_back);
        assert_eq!(loaded.settings.profiles.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_json_is_kept_before_reset() {
        assert_unreadable_is_kept("corrupt", "{ \"version\": 5, \"profiles\": [");
    }

    #[test]
    fn empty_file_is_kept_before_reset() {
        assert_unreadable_is_kept("empty", "");
    }

    #[test]
    fn field_with_unexpected_type_is_kept_before_reset() {
        assert_unreadable_is_kept("type", r#"{"version": 5, "soundEnabled": "yes"}"#);
    }

    #[test]
    fn existing_copy_is_never_overwritten() {
        let dir = test_dir("second");
        fs::write(dir.join("settings.unreadable.json"), "first").unwrap();
        fs::write(dir.join("settings.json"), "broken").unwrap();
        let loaded = read_settings_from(&dir);
        assert_eq!(
            loaded.unreadable_kept_at,
            Some(dir.join("settings.unreadable-2.json"))
        );
        assert_eq!(
            fs::read_to_string(dir.join("settings.unreadable.json")).unwrap(),
            "first"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn readable_and_missing_files_make_no_copy() {
        let dir = test_dir("fine");
        assert!(read_settings_from(&dir).unreadable_kept_at.is_none());
        let json = serde_json::to_string(&Settings::default()).unwrap();
        fs::write(dir.join("settings.json"), json).unwrap();
        let loaded = read_settings_from(&dir);
        assert!(loaded.unreadable_kept_at.is_none());
        assert!(!dir.join("settings.unreadable.json").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_atomic_replaces_file_and_leaves_no_temp_file() {
        let dir = test_dir("atomic");
        let path = dir.join("settings.json");
        fs::write(&path, "old").unwrap();
        write_atomic(&path, "new").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
        assert!(!dir.join("settings.json.tmp").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_serialization_roundtrip() {
        let settings = Settings {
            version: 2,
            ..Settings::default()
        };
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
        // Since Default::version is now 6, the deserialized value is 6.
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
        assert_eq!(settings.version, 6);
    }

    #[test]
    fn migrate_settings_upgrades_v0_to_v6() {
        let settings = Settings {
            version: 0,
            ..Settings::default()
        };
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 6);
        assert_eq!(migrated.profiles.len(), 1);
    }

    #[test]
    fn migrate_settings_upgrades_v1_to_v6() {
        let settings = Settings {
            version: 1,
            ..Settings::default()
        };
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 6);
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
        assert_eq!(settings.version, 6);
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
        assert_eq!(migrated.version, 6);
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
    fn migrate_v0_to_v6_goes_through_all_steps() {
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
        assert_eq!(migrated.version, 6);
        assert_eq!(migrated.profiles.len(), 1);
    }

    #[test]
    fn settings_v2_roundtrip_preserves_profiles() {
        let settings = Settings {
            version: 2,
            profiles: vec![
                RecordingProfile::default(),
                RecordingProfile {
                    id: "custom".into(),
                    name: "Podcast".into(),
                    ..RecordingProfile::default()
                },
            ],
            active_profile_id: "custom".into(),
            ..Settings::default()
        };
        let json = serde_json::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.profiles.len(), 2);
        assert_eq!(deserialized.active_profile_id, "custom");
        assert_eq!(deserialized.profiles[1].name, "Podcast");
    }

    #[test]
    fn migrate_v2_to_v6_adds_language_and_confirm_exit() {
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
        assert_eq!(migrated.version, 6);
        assert_eq!(migrated.language, "en");
        assert!(migrated.confirm_exit_during_recording);
    }

    #[test]
    fn migrate_v3_to_v6_adds_theme() {
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
        assert_eq!(migrated.version, 6);
        assert_eq!(migrated.theme, "auto");
    }

    #[test]
    fn settings_default_has_version_6() {
        let s = Settings::default();
        assert_eq!(s.version, 6);
        assert!(!s.minimize_to_tray_on_focus_loss);
    }

    #[test]
    fn migrate_v4_to_v6_adds_minimize_to_tray_on_focus_loss() {
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
        assert_eq!(migrated.version, 6);
        assert!(!migrated.minimize_to_tray_on_focus_loss);
    }

    /// `settings.json` as version 0.1.2 writes it (settings v5), with two profiles.
    const SETTINGS_V5: &str = r#"{
        "version": 5,
        "selectedMic": "{0.0.1.00000000}.{mic}",
        "selectedLoopback": "{0.0.0.00000000}.{speakers}",
        "hotkey": "F9",
        "soundEnabled": false,
        "profiles": [
            {
                "id": "default",
                "name": "Default",
                "description": "",
                "outputFolder": "Recordings",
                "outputMode": "Mix",
                "sampleRate": 48000,
                "micVolume": 1.0,
                "loopbackVolume": 0.5,
                "micFilename": "mic",
                "loopbackFilename": "loopback",
                "mixFilename": "mix"
            },
            {
                "id": "3f2c7a0e-5b1d-4c58-9a57-0d6f3f1f2a11",
                "name": "Лекції",
                "description": "Мікрофон і звук окремо",
                "outputFolder": "D:\\Записи\\Лекції",
                "outputMode": "SeparateFiles",
                "sampleRate": 16000,
                "micVolume": 2.5,
                "loopbackVolume": 0.8,
                "micFilename": "голос",
                "loopbackFilename": "звук",
                "mixFilename": "mix"
            }
        ],
        "activeProfileId": "3f2c7a0e-5b1d-4c58-9a57-0d6f3f1f2a11",
        "language": "uk",
        "confirmExitDuringRecording": false,
        "theme": "dark",
        "minimizeToTrayOnFocusLoss": true
    }"#;

    #[test]
    fn migrate_v5_to_v6_keeps_profiles_and_records_wav() {
        let dir = test_dir("v5");
        fs::write(dir.join("settings.json"), SETTINGS_V5).unwrap();
        let loaded = read_settings_from(&dir);
        assert!(loaded.unreadable_kept_at.is_none());
        assert!(loaded.write_back);
        let s = loaded.settings;

        assert_eq!(s.version, 6);
        assert_eq!(s.selected_mic.as_deref(), Some("{0.0.1.00000000}.{mic}"));
        assert_eq!(
            s.selected_loopback.as_deref(),
            Some("{0.0.0.00000000}.{speakers}")
        );
        assert_eq!(s.hotkey.as_deref(), Some("F9"));
        assert!(!s.sound_enabled);
        assert_eq!(s.active_profile_id, "3f2c7a0e-5b1d-4c58-9a57-0d6f3f1f2a11");
        assert_eq!(s.language, "uk");
        assert!(!s.confirm_exit_during_recording);
        assert_eq!(s.theme, "dark");
        assert!(s.minimize_to_tray_on_focus_loss);

        // Both profiles are there, unchanged, and record WAV as before.
        assert_eq!(s.profiles.len(), 2);
        assert_eq!(s.profiles[0], RecordingProfile::default());
        assert_eq!(
            s.profiles[1],
            RecordingProfile {
                id: "3f2c7a0e-5b1d-4c58-9a57-0d6f3f1f2a11".into(),
                name: "Лекції".into(),
                description: "Мікрофон і звук окремо".into(),
                output_folder: PathBuf::from("D:\\Записи\\Лекції"),
                output_mode: OutputMode::SeparateFiles,
                sample_rate: 16000,
                mic_volume: 2.5,
                loopback_volume: 0.8,
                mic_filename: "голос".into(),
                loopback_filename: "звук".into(),
                mix_filename: "mix".into(),
                output_format: "Wav".into(),
            }
        );
        assert!(s.profiles.iter().all(|p| p.format() == OutputFormat::Wav));

        // What the app writes back is version 6 with the format in each profile.
        let written: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(written["version"], 6);
        assert_eq!(written["profiles"][0]["outputFormat"], "Wav");
        assert_eq!(written["profiles"][1]["outputFormat"], "Wav");
        assert_eq!(written["profiles"][1]["name"], "Лекції");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_format_from_newer_version_is_read_and_kept() {
        // A file from a newer version: a higher settings version and a format this
        // version does not know. It is read without an error, no "unreadable"
        // copy is made, the profile records WAV, and the unknown value stays.
        let json = SETTINGS_V5
            .replace(r#""version": 5"#, r#""version": 7"#)
            .replace(
                r#""mixFilename": "mix"
            },"#,
                r#""mixFilename": "mix",
                "outputFormat": "Mp3Standard"
            },"#,
            );
        let dir = test_dir("newer");
        fs::write(dir.join("settings.json"), &json).unwrap();
        let loaded = read_settings_from(&dir);
        assert!(loaded.unreadable_kept_at.is_none());
        let s = loaded.settings;
        assert_eq!(s.version, 7);
        assert_eq!(s.profiles.len(), 2);
        assert_eq!(s.profiles[0].output_format, "Mp3Standard");
        assert_eq!(s.profiles[0].format(), OutputFormat::Wav);
        assert_eq!(s.profiles[1].format(), OutputFormat::Wav);

        let written = serde_json::to_string(&s).unwrap();
        assert!(written.contains(r#""outputFormat":"Mp3Standard""#));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrate_v4_uses_serde_default_when_field_absent() {
        // Field not present in JSON → serde fills false (the default)
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
        assert_eq!(migrated.version, 6);
        assert!(!migrated.minimize_to_tray_on_focus_loss);
    }

    #[test]
    fn migrate_v4_preserves_existing_minimize_to_tray_true() {
        // Field present with non-default value → serde must preserve it
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
            "minimizeToTrayOnFocusLoss": true
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        let migrated = migrate_settings(settings);
        assert_eq!(migrated.version, 6);
        assert!(migrated.minimize_to_tray_on_focus_loss);
    }
}
