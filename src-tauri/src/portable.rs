use std::fs;
use std::path::PathBuf;

/// Returns the directory containing the running executable.
pub fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Cannot determine exe directory"))
}

/// Returns the portable data directory (`<exe_dir>/AudioCaptor-data/`).
pub fn data_dir() -> anyhow::Result<PathBuf> {
    Ok(exe_dir()?.join("AudioCaptor-data"))
}

/// Returns the best-matching supported app language code for the current
/// Windows UI locale. Falls back to `"en"` on any error or unsupported locale.
#[cfg(target_os = "windows")]
fn detect_language() -> &'static str {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;

    let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH = 85
    // SAFETY: buf is valid and its full length is passed as the slice.
    let len = unsafe { GetUserDefaultLocaleName(&mut buf) };
    if len > 1 {
        // len includes the null terminator
        if let Ok(locale) = String::from_utf16(&buf[..(len as usize - 1)]) {
            let lang = locale.split('-').next().unwrap_or("");
            if lang == "uk" {
                return "uk";
            }
        }
    }
    "en"
}

#[cfg(not(target_os = "windows"))]
fn detect_language() -> &'static str {
    "en"
}

/// Creates required directories and default files on first run.
/// Called during Tauri setup.
pub fn ensure_dirs() -> anyhow::Result<()> {
    let base = data_dir()?;

    let dirs = ["logs", "Recordings"];
    for dir in &dirs {
        let path = base.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)?;
            log::info!("Created directory: {}", path.display());
        }
    }

    let settings_path = base.join("settings.json");
    if !settings_path.exists() {
        let mut defaults = crate::settings::migrate_settings(crate::settings::Settings::default());
        defaults.language = detect_language().to_string();
        let json = serde_json::to_string_pretty(&defaults)?;
        fs::write(&settings_path, json)?;
        log::info!("Created default settings.json with language={}", defaults.language);
    }

    Ok(())
}
