use std::{fs, path::PathBuf};

pub(super) fn profiles_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("profiles.json"))
}

pub(super) fn provider_settings_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("provider-settings.json"))
}

pub(super) fn review_history_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("review-history.json"))
}

pub(super) fn ensure_parent(path: &PathBuf) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Invalid local-review storage path.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())
}

fn config_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not resolve home directory.".to_string())?;
    Ok(home.join(".local-review"))
}
