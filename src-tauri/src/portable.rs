use std::fs;
use std::path::PathBuf;

/// Returns the directory containing the running executable.
pub fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Cannot determine exe directory"))
}

/// Creates required directories and default files on first run.
/// Called during Tauri setup.
pub fn ensure_dirs() -> anyhow::Result<()> {
    let base = exe_dir()?;

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
        let defaults = crate::settings::Settings::default();
        let json = serde_json::to_string_pretty(&defaults)?;
        fs::write(&settings_path, json)?;
        log::info!("Created default settings.json");
    }

    Ok(())
}
