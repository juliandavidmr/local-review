use serde::{Deserialize, Serialize};

use super::{
    ChangeSetSnapshot, ProviderSettings, RepositoryDescriptor, ReviewFeedback, ReviewProfileItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStatus {
    pub status: String,
    pub completed_passes: u32,
    pub total_passes: u32,
    pub changed_files: u32,
    pub modified_lines: u32,
    pub exploration_requests: u32,
    pub guardrail_hits: u32,
    #[serde(default)]
    pub current_file: Option<String>,
    #[serde(default)]
    pub current_profile: Option<String>,
    #[serde(default)]
    pub current_phase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicationSummary {
    pub target: String,
    pub total_comments: u32,
    pub inline_comments: u32,
    pub summary_comments: u32,
    pub limited_context_count: u32,
    pub incomplete_session: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewWorkspaceSession {
    pub repository: RepositoryDescriptor,
    pub change_source: String,
    pub change_set: ChangeSetSnapshot,
    pub profiles: Vec<ReviewProfileItem>,
    pub provider_settings: ProviderSettings,
    pub execution: ExecutionStatus,
    pub feedback: Vec<ReviewFeedback>,
    pub publication: PublicationSummary,
}
