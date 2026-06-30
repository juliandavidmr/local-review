use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDescriptor {
    pub path: String,
    pub name: String,
    pub current_branch: Option<String>,
    pub head_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChangeSource {
    WorkingTree {
        repository_path: String,
    },
    CurrentBranch {
        repository_path: String,
    },
    StagedChanges {
        repository_path: String,
    },
    UnstagedChanges {
        repository_path: String,
    },
    Commit {
        repository_path: String,
        commit_sha: String,
    },
    CompareRefs {
        repository_path: String,
        base_ref: String,
        head_ref: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangedFileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeLineKind {
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLine {
    pub kind: ChangeLineKind,
    pub content: String,
    pub old_line_number: Option<u32>,
    pub new_line_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeHunk {
    pub id: String,
    pub old_start_line: u32,
    pub new_start_line: u32,
    pub lines: Vec<ChangeLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: String,
    pub previous_path: Option<String>,
    pub status: ChangedFileStatus,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<ChangeHunk>,
    pub is_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetSnapshot {
    pub id: String,
    pub repository_path: String,
    pub source: ChangeSource,
    pub base_ref: Option<String>,
    pub head_ref: Option<String>,
    pub files: Vec<ChangedFile>,
    pub created_at: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileScopeKind {
    Global,
    Repository,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewProfileItem {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub scope_kind: ProfileScopeKind,
    pub selected: bool,
    pub enabled_by_default: bool,
    pub criteria: Vec<String>,
    pub file_globs: Vec<String>,
    pub prompt: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhCliStatus {
    pub installed: bool,
    pub authenticated: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    Inline,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackSeverity {
    Blocking,
    Important,
    Suggestion,
    Question,
    Nitpick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackState {
    Draft,
    Accepted,
    Edited,
    Dismissed,
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLocation {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub side: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFeedback {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub feedback_type: FeedbackType,
    pub severity: FeedbackSeverity,
    pub state: FeedbackState,
    pub profile_id: String,
    pub profile_name: String,
    pub pass_id: String,
    pub file: String,
    pub line: Option<u32>,
    pub body: String,
    pub editable_comment: String,
    pub suggested_action: String,
    pub confidence: String,
    pub limited_context: bool,
    pub quoted_code: Option<String>,
    pub evidence: Vec<String>,
    pub limitations: Vec<String>,
    pub code_location: Option<CodeLocation>,
    pub related_files: Vec<String>,
    pub model_provider: String,
    pub model: String,
    pub created_at: String,
}

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

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn default_provider_settings() -> ProviderSettings {
    ProviderSettings {
        model_providers: vec![
            ModelProviderSettings {
                id: "ollama".to_string(),
                kind: LocalModelProviderKind::Ollama,
                name: "Ollama".to_string(),
                base_url: "http://localhost:11434".to_string(),
                enabled: false,
                selected_model_id: None,
                use_for_human_tone_rewrite: false,
            },
            ModelProviderSettings {
                id: "lm-studio".to_string(),
                kind: LocalModelProviderKind::LmStudio,
                name: "LM Studio".to_string(),
                base_url: "http://localhost:1234/v1".to_string(),
                enabled: true,
                selected_model_id: None,
                use_for_human_tone_rewrite: false,
            },
        ],
        mcp_sources: vec![
            McpSourceSettings {
                id: "filesystem".to_string(),
                name: "Filesystem context".to_string(),
                description: Some("Guarded repository exploration.".to_string()),
                enabled: true,
            },
            McpSourceSettings {
                id: "github".to_string(),
                name: "GitHub context".to_string(),
                description: Some("Future configured MCP and gh context.".to_string()),
                enabled: false,
            },
        ],
        execution: ExecutionCapacitySettings {
            max_parallel_review_passes: 2,
            adaptive_parallelism_enabled: true,
        },
    }
}

pub fn default_profiles() -> Vec<ReviewProfileItem> {
    vec![
        ReviewProfileItem {
            id: "correctness".to_string(),
            name: "Correctness".to_string(),
            scope: "Global default".to_string(),
            scope_kind: ProfileScopeKind::Global,
            selected: true,
            enabled_by_default: true,
            criteria: vec![
                "Correctness".to_string(),
                "Regression risk".to_string(),
                "Edge cases".to_string(),
            ],
            file_globs: vec!["*".to_string()],
            prompt: "Review behavior regressions, incorrect assumptions, missing validation, and unsafe state transitions.".to_string(),
        },
        ReviewProfileItem {
            id: "architecture".to_string(),
            name: "Architecture".to_string(),
            scope: "Global default".to_string(),
            scope_kind: ProfileScopeKind::Global,
            selected: true,
            enabled_by_default: true,
            criteria: vec![
                "Hexagonal boundaries".to_string(),
                "Domain purity".to_string(),
                "Adapter isolation".to_string(),
            ],
            file_globs: vec!["*".to_string()],
            prompt: "Review architecture boundaries, coupling, and adherence to documented domain language.".to_string(),
        },
    ]
}
