use std::fs;

use crate::domain::{default_provider_settings, ProviderSettings};

use super::paths::{ensure_parent, provider_settings_path};

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
