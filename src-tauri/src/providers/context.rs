use serde_json::Value;

use crate::domain::{LocalModelProviderKind, ModelProviderSettings};

const FALLBACK_CONTEXT_TOKENS: u32 = 6_144;
const OUTPUT_RESERVE_TOKENS: u32 = 1_024;
const TOOL_SCHEMA_RESERVE_TOKENS: u32 = 1_400;
const STATIC_PROMPT_RESERVE_TOKENS: u32 = 700;
const CHARS_PER_TOKEN_ESTIMATE: u32 = 2;
const USABLE_CONTEXT_PERCENT: u32 = 75;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ModelPromptBudget {
    pub context_tokens: u32,
    pub max_prompt_chars: usize,
}

pub(crate) async fn model_prompt_budget(
    provider: &ModelProviderSettings,
    model_id: &str,
    tools_enabled: bool,
) -> ModelPromptBudget {
    let context_tokens = detect_context_tokens(provider, model_id)
        .await
        .unwrap_or(FALLBACK_CONTEXT_TOKENS)
        .max(2_048);
    ModelPromptBudget {
        context_tokens,
        max_prompt_chars: prompt_char_budget(context_tokens, tools_enabled),
    }
}

fn prompt_char_budget(context_tokens: u32, tools_enabled: bool) -> usize {
    let usable_context_tokens = context_tokens
        .saturating_mul(USABLE_CONTEXT_PERCENT)
        .saturating_div(100);
    let reserved_tokens = OUTPUT_RESERVE_TOKENS
        + STATIC_PROMPT_RESERVE_TOKENS
        + if tools_enabled {
            TOOL_SCHEMA_RESERVE_TOKENS
        } else {
            0
        };
    let prompt_tokens = usable_context_tokens
        .saturating_sub(reserved_tokens)
        .max(768)
        .min(usable_context_tokens);

    prompt_tokens.saturating_mul(CHARS_PER_TOKEN_ESTIMATE) as usize
}

async fn detect_context_tokens(provider: &ModelProviderSettings, model_id: &str) -> Option<u32> {
    match provider.kind {
        LocalModelProviderKind::LmStudio => detect_lmstudio_context(provider, model_id).await,
        LocalModelProviderKind::Ollama => detect_ollama_context(provider, model_id).await,
    }
}

async fn detect_lmstudio_context(provider: &ModelProviderSettings, model_id: &str) -> Option<u32> {
    let client = reqwest::Client::new();
    for url in lmstudio_metadata_urls(provider) {
        let value = client
            .get(url)
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?
            .json::<Value>()
            .await
            .ok()?;

        if let Some(model) = find_model_metadata(&value, model_id) {
            if let Some(context) = context_tokens_from_value(model) {
                return Some(context);
            }
        }
        if let Some(context) = context_tokens_from_value(&value) {
            return Some(context);
        }
    }

    None
}

fn lmstudio_metadata_urls(provider: &ModelProviderSettings) -> Vec<String> {
    let base = provider.base_url.trim_end_matches('/');
    let root = base.strip_suffix("/v1").unwrap_or(base);

    vec![format!("{base}/models"), format!("{root}/api/v0/models")]
}

async fn detect_ollama_context(provider: &ModelProviderSettings, model_id: &str) -> Option<u32> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/show", provider.base_url.trim_end_matches('/'));
    let value = client
        .post(url)
        .json(&serde_json::json!({
            "name": model_id,
            "verbose": true,
        }))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<Value>()
        .await
        .ok()?;

    context_tokens_from_value(&value)
}

fn find_model_metadata<'a>(value: &'a Value, model_id: &str) -> Option<&'a Value> {
    match value {
        Value::Array(items) => items.iter().find(|item| value_has_model_id(item, model_id)),
        Value::Object(map) => {
            if value_has_model_id(value, model_id) {
                return Some(value);
            }
            map.get("data")
                .and_then(|data| find_model_metadata(data, model_id))
                .or_else(|| {
                    map.get("models")
                        .and_then(|models| find_model_metadata(models, model_id))
                })
        }
        _ => None,
    }
}

fn value_has_model_id(value: &Value, model_id: &str) -> bool {
    value
        .as_object()
        .map(|map| {
            ["id", "name", "model", "model_id"]
                .iter()
                .filter_map(|key| map.get(*key))
                .filter_map(Value::as_str)
                .any(|candidate| candidate == model_id)
        })
        .unwrap_or(false)
}

fn context_tokens_from_value(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) => number.as_u64().and_then(valid_context_value),
        Value::String(text) => text.parse::<u64>().ok().and_then(valid_context_value),
        Value::Array(items) => items.iter().find_map(context_tokens_from_value),
        Value::Object(map) => {
            for (key, value) in map {
                let normalized = key.to_ascii_lowercase();
                if is_context_key(&normalized) {
                    if let Some(context) = context_tokens_from_value(value) {
                        return Some(context);
                    }
                }
            }

            map.values().find_map(context_tokens_from_value)
        }
        _ => None,
    }
}

fn is_context_key(key: &str) -> bool {
    key.contains("context_length")
        || key.contains("max_context")
        || key.contains("context_window")
        || key == "n_ctx"
        || key == "num_ctx"
        || key.ends_with(".context_length")
}

fn valid_context_value(value: u64) -> Option<u32> {
    if (512..=1_000_000).contains(&value) {
        Some(value as u32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{context_tokens_from_value, find_model_metadata, prompt_char_budget};
    use serde_json::json;

    #[test]
    fn extracts_lmstudio_style_context_length() {
        let value = json!({
            "data": [
                {"id": "other", "max_context_length": 8192},
                {"id": "gemma", "context_length": 6144}
            ]
        });
        let model = find_model_metadata(&value, "gemma").expect("model metadata");

        assert_eq!(context_tokens_from_value(model), Some(6144));
    }

    #[test]
    fn extracts_ollama_style_context_length() {
        let value = json!({
            "model_info": {
                "llama.context_length": 4096
            }
        });

        assert_eq!(context_tokens_from_value(&value), Some(4096));
    }

    #[test]
    fn prompt_budget_is_more_conservative_when_tools_are_enabled() {
        let without_tools = prompt_char_budget(8_192, false);
        let with_tools = prompt_char_budget(8_192, true);

        assert!(with_tools < without_tools);
        assert!(with_tools <= 5_000);
    }
}
