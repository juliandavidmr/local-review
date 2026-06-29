use std::{fs, path::PathBuf};

use crate::domain::{
    default_profiles, default_provider_settings, ProviderSettings, ReviewFeedback,
    ReviewProfileItem, ReviewWorkspaceSession,
};

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

pub fn load_provider_settings() -> Result<ProviderSettings, String> {
    let path = provider_settings_path()?;
    if !path.exists() {
        let settings = default_provider_settings();
        save_provider_settings(settings.clone())?;
        return Ok(settings);
    }

    let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

pub fn save_provider_settings(settings: ProviderSettings) -> Result<ProviderSettings, String> {
    let path = provider_settings_path()?;
    ensure_parent(&path)?;
    let normalized = normalize_provider_selection(settings);
    let raw = serde_json::to_string_pretty(&normalized).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())?;
    Ok(normalized)
}

pub fn load_review_sessions() -> Result<Vec<ReviewWorkspaceSession>, String> {
    let path = review_history_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

pub fn save_review_session(
    session: ReviewWorkspaceSession,
) -> Result<ReviewWorkspaceSession, String> {
    let mut sessions = load_review_sessions()?;
    let session_id = session.change_set.id.clone();
    if let Some(index) = sessions
        .iter()
        .position(|existing| existing.change_set.id == session_id)
    {
        sessions[index] = session.clone();
    } else {
        sessions.insert(0, session.clone());
    }
    write_review_sessions(sessions)?;
    Ok(session)
}

pub fn load_latest_review_session() -> Result<Option<ReviewWorkspaceSession>, String> {
    Ok(load_review_sessions()?.into_iter().next())
}

pub fn update_review_feedback(
    session_id: &str,
    feedback_id: &str,
    feedback: ReviewFeedback,
) -> Result<ReviewWorkspaceSession, String> {
    let mut sessions = load_review_sessions()?;
    let session = sessions
        .iter_mut()
        .find(|session| session.change_set.id == session_id)
        .ok_or_else(|| "Review session was not found in local history.".to_string())?;

    let item = session
        .feedback
        .iter_mut()
        .find(|item| item.id == feedback_id)
        .ok_or_else(|| "Review feedback was not found in local history.".to_string())?;
    *item = feedback;
    recalculate_publication_summary(session);

    let updated = session.clone();
    write_review_sessions(sessions)?;
    Ok(updated)
}

fn write_review_sessions(sessions: Vec<ReviewWorkspaceSession>) -> Result<(), String> {
    let path = review_history_path()?;
    ensure_parent(&path)?;
    let raw = serde_json::to_string_pretty(&sessions).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

fn recalculate_publication_summary(session: &mut ReviewWorkspaceSession) {
    session.publication.total_comments = session.feedback.len() as u32;
    session.publication.inline_comments = session
        .feedback
        .iter()
        .filter(|item| matches!(item.feedback_type, crate::domain::FeedbackType::Inline))
        .count() as u32;
    session.publication.summary_comments =
        session.publication.total_comments - session.publication.inline_comments;
    session.publication.limited_context_count = session
        .feedback
        .iter()
        .filter(|item| item.limited_context)
        .count() as u32;
}

fn normalize_provider_selection(settings: ProviderSettings) -> ProviderSettings {
    let selected_id = settings
        .model_providers
        .iter()
        .find(|provider| provider.enabled)
        .map(|provider| provider.id.clone());

    ProviderSettings {
        model_providers: settings
            .model_providers
            .into_iter()
            .map(|mut provider| {
                provider.enabled = selected_id
                    .as_ref()
                    .map(|id| id == &provider.id)
                    .unwrap_or(false);
                if !provider.enabled {
                    provider.selected_model_id = None;
                    provider.use_for_human_tone_rewrite = false;
                }
                provider
            })
            .collect(),
        ..settings
    }
}

fn profiles_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("profiles.json"))
}

fn provider_settings_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("provider-settings.json"))
}

fn review_history_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("review-history.json"))
}

fn config_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not resolve home directory.".to_string())?;
    Ok(home.join(".local-review"))
}

fn ensure_parent(path: &PathBuf) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Invalid local-review storage path.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())
}
