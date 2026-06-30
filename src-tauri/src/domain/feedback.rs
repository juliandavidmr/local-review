use serde::{Deserialize, Serialize};

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
