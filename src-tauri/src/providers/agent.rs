use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use super::{
    tools::{ReadRepositoryFileTool, SearchRepositoryTool, ToolUsageHook},
    types::{AgentFeedbackOutput, AgentProgressContext, ReviewAgentResult},
};

pub(super) async fn run_rig_agent(
    base_url: &str,
    model: &str,
    prompt: &str,
    repository_path: &str,
    repository_tools_enabled: bool,
    progress: Option<AgentProgressContext>,
) -> Result<ReviewAgentResult, String> {
    use rig::{
        agent::AgentBuilder, client::CompletionClient, completion::Prompt, providers::openai,
    };

    let client = openai::CompletionsClient::builder()
        .api_key("local-review")
        .base_url(base_url)
        .build()
        .map_err(|error| error.to_string())?;
    let model = client.completion_model(model);
    if repository_tools_enabled {
        let exploration_requests = Arc::new(AtomicU32::new(0));
        let hook = ToolUsageHook {
            exploration_requests: exploration_requests.clone(),
            progress,
        };
        let agent = AgentBuilder::new(model)
            .preamble("You are a senior code reviewer preparing draft comments for direct publication. Return only valid JSON matching the requested schema. Do not use markdown fences.")
            .temperature(0.1)
            .tool(ReadRepositoryFileTool::new(repository_path))
            .tool(SearchRepositoryTool::new(repository_path))
            .build();

        let response = agent
            .prompt(prompt.to_string())
            .max_turns(6)
            .with_hook(hook)
            .await
            .map_err(|error| error.to_string())?;
        Ok(ReviewAgentResult {
            raw: response.to_string(),
            exploration_requests: exploration_requests.load(Ordering::SeqCst),
        })
    } else {
        let agent = AgentBuilder::new(model)
            .preamble("You are a senior code reviewer preparing draft comments for direct publication. Return only valid JSON matching the requested schema. Do not use markdown fences.")
            .temperature(0.1)
            .build();

        let response = agent
            .prompt(prompt.to_string())
            .max_turns(2)
            .await
            .map_err(|error| error.to_string())?;
        Ok(ReviewAgentResult {
            raw: response.to_string(),
            exploration_requests: 0,
        })
    }
}

pub(super) async fn repair_model_json(
    base_url: &str,
    model: &str,
    raw: &str,
) -> Result<String, String> {
    let prompt = format!(
        "Repair this malformed JSON response from a code review model.\n\nRules:\n- Return only valid JSON.\n- Preserve all feedback content.\n- The root must be an object with a feedback array.\n- Each feedback item should use keys title, severity, line, body, suggestedAction, evidence, limitations.\n- Do not add markdown fences.\n\nMalformed response:\n{}",
        raw
    );

    let result = run_rig_agent(base_url, model, &prompt, ".", false, None).await?;
    Ok(result.raw)
}

pub(super) fn parse_json_from_model(raw: &str) -> Result<AgentFeedbackOutput, String> {
    let trimmed = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if trimmed.is_empty() {
        return Ok(AgentFeedbackOutput {
            feedback: Vec::new(),
        });
    }

    serde_json::from_str(trimmed)
        .or_else(|_| {
            let start = trimmed.find('{').ok_or_else(|| {
                serde_json::Error::io(std::io::Error::other("missing JSON object"))
            })?;
            let end = trimmed.rfind('}').ok_or_else(|| {
                serde_json::Error::io(std::io::Error::other("missing JSON object"))
            })?;
            serde_json::from_str(&trimmed[start..=end])
        })
        .map_err(|error| format!("Invalid model JSON: {error}; raw: {raw}"))
}

#[cfg(test)]
mod tests {
    use super::parse_json_from_model;

    #[test]
    fn parses_empty_object_as_no_feedback() {
        let parsed = parse_json_from_model("{}").expect("empty object should be accepted");

        assert!(parsed.feedback.is_empty());
    }

    #[test]
    fn parses_empty_response_as_no_feedback() {
        let parsed = parse_json_from_model("").expect("empty response should be accepted");

        assert!(parsed.feedback.is_empty());
    }
}
