use crate::domain::ReviewFeedback;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer};
use std::sync::{atomic::AtomicU32, Arc};
use tauri::AppHandle;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct AgentFeedbackOutput {
    #[serde(default)]
    pub feedback: Vec<AgentFeedbackItem>,
}

#[derive(Debug, Deserialize, JsonSchema)]
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
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub evidence: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub limitations: Vec<String>,
    pub quoted_code: Option<String>,
}

fn deserialize_string_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .filter_map(|item| match item {
                serde_json::Value::String(text) if !text.trim().is_empty() => Some(text),
                _ => None,
            })
            .collect(),
        serde_json::Value::String(text) if !text.trim().is_empty() => vec![text],
        _ => Vec::new(),
    })
}

pub(super) fn default_feedback_severity() -> String {
    "suggestion".to_string()
}

pub(super) struct ReviewAgentResult {
    pub raw: String,
    pub exploration_requests: u32,
}

#[derive(Clone)]
pub(crate) struct AgentProgressContext {
    pub app: AppHandle,
    pub review_id: String,
    pub current_file: String,
    pub current_profile: String,
    pub completed_passes: Arc<AtomicU32>,
    pub total_passes: u32,
    pub failed_passes: Arc<AtomicU32>,
    pub exploration_requests: Arc<AtomicU32>,
    pub current_phase: String,
}

pub(crate) struct ReviewPassResult {
    pub feedback: Vec<ReviewFeedback>,
    pub exploration_requests: u32,
}

#[cfg(test)]
mod tests {
    use super::AgentFeedbackOutput;

    #[test]
    fn feedback_output_schema_matches_review_json_shape() {
        let schema = schemars::schema_for!(AgentFeedbackOutput);
        let value = serde_json::to_value(schema).expect("schema should serialize");

        assert_eq!(value["title"], "AgentFeedbackOutput");
        assert!(value["properties"]["feedback"].is_object());
        assert_eq!(value["properties"]["feedback"]["type"], "array");
    }

    #[test]
    fn accepts_repaired_feedback_with_string_evidence_and_limitations() {
        let raw = r#"{
            "feedback": [
                {
                    "title": "Update file status without validating filesStatus existence",
                    "severity": "important",
                    "line": 38,
                    "body": "The store calls get().updateFileStatus(lessonId, status, preregister",
                    "suggestedAction": "",
                    "evidence": "",
                    "limitations": "repaired from malformed model output"
                }
            ]
        }"#;

        let parsed: AgentFeedbackOutput =
            serde_json::from_str(raw).expect("string lists should be normalized");

        assert_eq!(parsed.feedback.len(), 1);
        assert!(parsed.feedback[0].evidence.is_empty());
        assert_eq!(
            parsed.feedback[0].limitations,
            vec!["repaired from malformed model output"]
        );
    }
}
