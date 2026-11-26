use crate::audio::types::{OutputFormat, OutputMode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordingProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub output_folder: PathBuf,
    pub output_mode: OutputMode,
    pub sample_rate: u32,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub mic_filename: String,
    pub loopback_filename: String,
    pub mix_filename: String,
    /// The id of an `OutputFormat`, NEW in settings v6. Kept as text, not as the
    /// enum: a format added by a newer version must load without an error and
    /// stay in the file (see `format`). Profiles from v5 have no such field.
    #[serde(default = "default_output_format")]
    pub output_format: String,
}

fn default_output_format() -> String {
    OutputFormat::Wav.id().to_string()
}

impl RecordingProfile {
    /// The format to record in. A format this version does not know records WAV.
    pub fn format(&self) -> OutputFormat {
        OutputFormat::from_id(&self.output_format).unwrap_or_default()
    }
}

const FORBIDDEN_FILENAME_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

impl Default for RecordingProfile {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            description: String::new(),
            output_folder: PathBuf::from("Recordings"),
            output_mode: OutputMode::Mix,
            sample_rate: 48000,
            mic_volume: 1.0,
            loopback_volume: 0.5,
            mic_filename: "mic".to_string(),
            loopback_filename: "loopback".to_string(),
            mix_filename: "mix".to_string(),
            output_format: default_output_format(),
        }
    }
}

pub fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Filename cannot be empty".into());
    }
    if let Some(c) = name.chars().find(|c| FORBIDDEN_FILENAME_CHARS.contains(c)) {
        return Err(format!("Filename contains forbidden character: '{c}'"));
    }
    Ok(())
}

pub fn validate_profile_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Profile name cannot be empty".into());
    }
    Ok(())
}

pub fn save_profile(
    profiles: &mut Vec<RecordingProfile>,
    profile: RecordingProfile,
) -> Result<(), String> {
    validate_profile_name(&profile.name)?;
    validate_filename(&profile.mic_filename)?;
    validate_filename(&profile.loopback_filename)?;
    validate_filename(&profile.mix_filename)?;

    // Check for duplicate name (different id)
    let has_dup = profiles
        .iter()
        .any(|p| p.id != profile.id && p.name == profile.name);
    if has_dup {
        return Err(format!("Profile name '{}' is a duplicate", profile.name));
    }

    // Update existing or insert new
    if let Some(existing) = profiles.iter_mut().find(|p| p.id == profile.id) {
        *existing = profile;
    } else {
        profiles.push(profile);
    }

    Ok(())
}

pub fn delete_profile(profiles: &mut Vec<RecordingProfile>, id: &str) -> Result<(), String> {
    if profiles.len() <= 1 {
        return Err("Cannot delete the last profile".into());
    }
    let idx = profiles
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| format!("Profile not found: {id}"))?;
    profiles.remove(idx);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_has_expected_values() {
        let p = RecordingProfile::default();
        assert_eq!(p.id, "default");
        assert_eq!(p.name, "Default");
        assert_eq!(p.output_mode, OutputMode::Mix);
        assert_eq!(p.sample_rate, 48000);
        assert_eq!(p.mic_volume, 1.0);
        assert_eq!(p.loopback_volume, 0.5);
        assert_eq!(p.mic_filename, "mic");
        assert_eq!(p.loopback_filename, "loopback");
        assert_eq!(p.mix_filename, "mix");
        assert_eq!(p.output_format, "Wav");
        assert_eq!(p.format(), OutputFormat::Wav);
    }

    #[test]
    fn profile_serialization_roundtrip() {
        let p = RecordingProfile::default();
        let json = serde_json::to_string_pretty(&p).unwrap();
        let deserialized: RecordingProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, p);
    }

    #[test]
    fn profile_format_is_stored_by_id() {
        let p = RecordingProfile {
            output_format: "OpusSpeech".into(),
            ..RecordingProfile::default()
        };
        assert_eq!(p.format(), OutputFormat::OpusSpeech);
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains(r#""outputFormat":"OpusSpeech""#), "{json}");
        let music: RecordingProfile =
            serde_json::from_str(&json.replace("OpusSpeech", "OpusMusic")).unwrap();
        assert_eq!(music.format(), OutputFormat::OpusMusic);
    }

    #[test]
    fn profile_without_format_records_wav() {
        // A profile as settings v5 wrote it: no `outputFormat` field.
        let json = r#"{
            "id": "default", "name": "Default", "description": "",
            "outputFolder": "Recordings", "outputMode": "Mix", "sampleRate": 48000,
            "micVolume": 1.0, "loopbackVolume": 0.5,
            "micFilename": "mic", "loopbackFilename": "loopback", "mixFilename": "mix"
        }"#;
        let p: RecordingProfile = serde_json::from_str(json).unwrap();
        assert_eq!(p.output_format, "Wav");
        assert_eq!(p.format(), OutputFormat::Wav);
    }

    #[test]
    fn profile_with_unknown_format_loads_and_records_wav() {
        // A format from a newer version: the profile loads, records WAV, and the
        // unknown value is written back unchanged.
        let json = r#"{
            "id": "p", "name": "Podcast", "description": "",
            "outputFolder": "Recordings", "outputMode": "Mix", "sampleRate": 48000,
            "micVolume": 1.0, "loopbackVolume": 0.5,
            "micFilename": "mic", "loopbackFilename": "loopback", "mixFilename": "mix",
            "outputFormat": "Mp3Standard"
        }"#;
        let p: RecordingProfile = serde_json::from_str(json).unwrap();
        assert_eq!(p.format(), OutputFormat::Wav);
        assert_eq!(p.output_format, "Mp3Standard");
        let written = serde_json::to_string(&p).unwrap();
        assert!(
            written.contains(r#""outputFormat":"Mp3Standard""#),
            "{written}"
        );
    }

    #[test]
    fn validate_filename_rejects_forbidden_chars() {
        assert!(validate_filename("good_name").is_ok());
        assert!(validate_filename("my-recording").is_ok());
        assert!(validate_filename("test 123").is_ok());
        assert!(validate_filename("bad<name").is_err());
        assert!(validate_filename("bad>name").is_err());
        assert!(validate_filename("bad:name").is_err());
        assert!(validate_filename("bad\"name").is_err());
        assert!(validate_filename("bad/name").is_err());
        assert!(validate_filename("bad\\name").is_err());
        assert!(validate_filename("bad|name").is_err());
        assert!(validate_filename("bad?name").is_err());
        assert!(validate_filename("bad*name").is_err());
    }

    #[test]
    fn validate_filename_rejects_empty() {
        assert!(validate_filename("").is_err());
    }

    #[test]
    fn validate_profile_name_rejects_empty() {
        assert!(validate_profile_name("").is_err());
    }

    #[test]
    fn validate_profile_name_accepts_valid() {
        assert!(validate_profile_name("My Profile").is_ok());
        assert!(validate_profile_name("Podcast Recording").is_ok());
    }

    #[test]
    fn save_profile_new_adds_to_list() {
        let mut profiles = vec![RecordingProfile::default()];
        let new_profile = RecordingProfile {
            id: "new-id".into(),
            name: "Podcast".into(),
            ..RecordingProfile::default()
        };
        let result = save_profile(&mut profiles, new_profile);
        assert!(result.is_ok());
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[1].name, "Podcast");
    }

    #[test]
    fn save_profile_existing_updates_in_place() {
        let mut profiles = vec![RecordingProfile::default()];
        let mut updated = profiles[0].clone();
        updated.name = "Updated Default".into();
        updated.mic_volume = 2.0;
        let result = save_profile(&mut profiles, updated);
        assert!(result.is_ok());
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Updated Default");
        assert_eq!(profiles[0].mic_volume, 2.0);
    }

    #[test]
    fn save_profile_rejects_duplicate_name() {
        let mut profiles = vec![RecordingProfile::default()];
        let dup = RecordingProfile {
            id: "other-id".into(),
            name: "Default".into(),
            ..RecordingProfile::default()
        };
        let result = save_profile(&mut profiles, dup);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("duplicate"));
    }

    #[test]
    fn save_profile_validates_filenames() {
        let mut profiles = vec![RecordingProfile::default()];
        let bad = RecordingProfile {
            id: "bad".into(),
            name: "Bad Profile".into(),
            mic_filename: "bad<file".into(),
            ..RecordingProfile::default()
        };
        let result = save_profile(&mut profiles, bad);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("forbidden"));
    }

    #[test]
    fn delete_profile_removes_by_id() {
        let mut profiles = vec![
            RecordingProfile::default(),
            RecordingProfile {
                id: "second".into(),
                name: "Second".into(),
                ..RecordingProfile::default()
            },
        ];
        let result = delete_profile(&mut profiles, "second");
        assert!(result.is_ok());
        assert_eq!(profiles.len(), 1);
    }

    #[test]
    fn delete_profile_rejects_last_profile() {
        let mut profiles = vec![RecordingProfile::default()];
        let result = delete_profile(&mut profiles, "default");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("last"));
    }

    #[test]
    fn delete_profile_rejects_nonexistent() {
        let mut profiles = vec![RecordingProfile::default()];
        let result = delete_profile(&mut profiles, "nonexistent");
        assert!(result.is_err());
    }
}
