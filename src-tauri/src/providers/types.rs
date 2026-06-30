use crate::domain::ReviewFeedback;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AgentFeedbackOutput {
    pub feedback: Vec<AgentFeedbackItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AgentFeedbackItem {
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_feedback_severity")]
    pub severity: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub suggested_action: String,
    pub confidence: Option<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub limitations: Vec<String>,
    pub quoted_code: Option<String>,
}

pub(super) fn default_feedback_severity() -> String {
    "suggestion".to_string()
}

pub(super) struct ReviewAgentResult {
    pub raw: String,
    pub exploration_requests: u32,
}

pub(crate) struct ReviewPassResult {
    pub feedback: Vec<ReviewFeedback>,
    pub exploration_requests: u32,
}
