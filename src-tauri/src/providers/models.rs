use crate::domain::{
    LocalModelProviderKind, ModelDescriptor, ModelProviderSettings, ProviderConnectionStatus,
};
use serde::Deserialize;

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

pub(super) fn openai_base_url(provider: &ModelProviderSettings) -> String {
    match provider.kind {
        LocalModelProviderKind::Ollama => {
            format!("{}/v1", provider.base_url.trim_end_matches('/'))
        }
        LocalModelProviderKind::LmStudio => provider.base_url.trim_end_matches('/').to_string(),
    }
}
