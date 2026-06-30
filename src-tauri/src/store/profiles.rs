use std::fs;

use crate::domain::{default_profiles, ReviewProfileItem};

use super::paths::{ensure_parent, profiles_path};

pub fn load_profiles() -> Result<Vec<ReviewProfileItem>, String> {
    let path = profiles_path()?;
    if !path.exists() {
        let profiles = default_profiles();
        save_profiles(profiles.clone())?;
        return Ok(profiles);
    }

    let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

pub fn save_profiles(profiles: Vec<ReviewProfileItem>) -> Result<(), String> {
    let path = profiles_path()?;
    ensure_parent(&path)?;
    let raw = serde_json::to_string_pretty(&profiles).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

pub fn save_profile(profile: ReviewProfileItem) -> Result<Vec<ReviewProfileItem>, String> {
    let mut profiles = load_profiles()?;
    if let Some(index) = profiles
        .iter()
        .position(|existing| existing.id == profile.id)
    {
        profiles[index] = profile;
    } else {
        profiles.insert(0, profile);
    }
    save_profiles(profiles.clone())?;
    Ok(profiles)
}

pub fn delete_profile(profile_id: &str) -> Result<Vec<ReviewProfileItem>, String> {
    let profiles = load_profiles()?
        .into_iter()
        .filter(|profile| profile.id != profile_id)
        .collect::<Vec<_>>();
    save_profiles(profiles.clone())?;
    Ok(profiles)
}
