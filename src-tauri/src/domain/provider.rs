use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelProviderKind {
    Ollama,
    LmStudio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderSettings {
    pub id: String,
    pub kind: LocalModelProviderKind,
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
    pub selected_model_id: Option<String>,
    pub use_for_human_tone_rewrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSourceSettings {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionCapacitySettings {
    pub max_parallel_review_passes: u32,
    pub adaptive_parallelism_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSettings {
    pub model_providers: Vec<ModelProviderSettings>,
    pub mcp_sources: Vec<McpSourceSettings>,
    pub execution: ExecutionCapacitySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDescriptor {
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConnectionStatus {
    pub provider_id: String,
    pub ok: bool,
    pub message: String,
}
