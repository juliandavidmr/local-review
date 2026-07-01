use crate::domain::{
    LocalModelProviderKind, ModelDescriptor, ModelProviderSettings, ProviderConnectionStatus,
};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const MODEL_READY_RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug, Deserialize)]
struct OllamaTags {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiModels {
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModel {
    id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ModelReadiness {
    pub attempts: u32,
}

pub async fn check_connection(
    provider: ModelProviderSettings,
) -> Result<ProviderConnectionStatus, String> {
    match list_models(provider.clone()).await {
        Ok(models) if models.is_empty() => Ok(ProviderConnectionStatus {
            provider_id: provider.id,
            ok: true,
            message: "Connected, but no models were returned.".to_string(),
        }),
        Ok(models) => Ok(ProviderConnectionStatus {
            provider_id: provider.id,
            ok: true,
            message: format!("Connected. {} model(s) available.", models.len()),
        }),
        Err(error) => Ok(ProviderConnectionStatus {
            provider_id: provider.id,
            ok: false,
            message: error,
        }),
    }
}

pub async fn list_models(provider: ModelProviderSettings) -> Result<Vec<ModelDescriptor>, String> {
    let client = reqwest::Client::new();

    match provider.kind {
        LocalModelProviderKind::Ollama => {
            let url = format!("{}/api/tags", provider.base_url.trim_end_matches('/'));
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|error| format!("Could not reach Ollama: {error}"))?;
            let tags = response
                .error_for_status()
                .map_err(|error| error.to_string())?
                .json::<OllamaTags>()
                .await
                .map_err(|error| error.to_string())?;

            Ok(tags
                .models
                .into_iter()
                .map(|model| ModelDescriptor {
                    provider_id: provider.id.clone(),
                    model_id: model.name.clone(),
                    display_name: model.name,
                    available: true,
                })
                .collect())
        }
        LocalModelProviderKind::LmStudio => {
            let url = format!("{}/models", provider.base_url.trim_end_matches('/'));
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|error| format!("Could not reach LM Studio: {error}"))?;
            let models = response
                .error_for_status()
                .map_err(|error| error.to_string())?
                .json::<OpenAiModels>()
                .await
                .map_err(|error| error.to_string())?;

            Ok(models
                .data
                .into_iter()
                .map(|model| ModelDescriptor {
                    provider_id: provider.id.clone(),
                    model_id: model.id.clone(),
                    display_name: model.id,
                    available: true,
                })
                .collect())
        }
    }
}

pub(crate) async fn wait_for_selected_model_ready(
    provider: &ModelProviderSettings,
    timeout: Duration,
) -> Result<ModelReadiness, String> {
    let model = provider
        .selected_model_id
        .as_deref()
        .ok_or_else(|| "No model selected.".to_string())?;
    let started_at = Instant::now();
    let mut attempts = 0;

    loop {
        attempts += 1;
        match check_selected_model_ready_once(provider, model).await {
            Ok(()) => return Ok(ModelReadiness { attempts }),
            Err(error) => {
                if started_at.elapsed() >= timeout {
                    return Err(format!(
                        "Selected model `{model}` was not ready after {}s: {error}",
                        timeout.as_secs()
                    ));
                }
            }
        }

        sleep(MODEL_READY_RETRY_DELAY).await;
    }
}

async fn check_selected_model_ready_once(
    provider: &ModelProviderSettings,
    model: &str,
) -> Result<(), String> {
    let models = list_models(provider.clone()).await?;
    if !selected_model_is_listed(&models, model) {
        return Err(format!(
            "Selected model `{model}` is not listed by {}.",
            provider.name
        ));
    }

    probe_openai_compatible_completion(provider, model).await
}

fn selected_model_is_listed(models: &[ModelDescriptor], model: &str) -> bool {
    models
        .iter()
        .any(|candidate| candidate.available && candidate.model_id == model)
}

async fn probe_openai_compatible_completion(
    provider: &ModelProviderSettings,
    model: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", openai_base_url(provider));
    let response = client
        .post(url)
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": "Reply with ok."
                }
            ],
            "max_tokens": 1,
            "temperature": 0,
            "stream": false
        }))
        .send()
        .await
        .map_err(|error| format!("Could not probe selected model readiness: {error}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(format!(
        "Readiness probe failed with status {status}: {}",
        body.trim()
    ))
}

pub(super) fn openai_base_url(provider: &ModelProviderSettings) -> String {
    match provider.kind {
        LocalModelProviderKind::Ollama => {
            format!("{}/v1", provider.base_url.trim_end_matches('/'))
        }
        LocalModelProviderKind::LmStudio => provider.base_url.trim_end_matches('/').to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::ModelDescriptor;

    use super::selected_model_is_listed;

    #[test]
    fn selected_model_is_listed_requires_available_exact_match() {
        let models = vec![
            ModelDescriptor {
                provider_id: "lm-studio".to_string(),
                model_id: "other-model".to_string(),
                display_name: "other-model".to_string(),
                available: true,
            },
            ModelDescriptor {
                provider_id: "lm-studio".to_string(),
                model_id: "target-model".to_string(),
                display_name: "target-model".to_string(),
                available: false,
            },
        ];

        assert!(!selected_model_is_listed(&models, "target-model"));
    }

    #[test]
    fn selected_model_is_listed_accepts_available_exact_match() {
        let models = vec![ModelDescriptor {
            provider_id: "lm-studio".to_string(),
            model_id: "target-model".to_string(),
            display_name: "target-model".to_string(),
            available: true,
        }];

        assert!(selected_model_is_listed(&models, "target-model"));
    }
}
