use crate::domain::{
    now_iso, ChangeLineKind, ChangeSetSnapshot, ChangedFile, CodeLocation, FeedbackSeverity,
    FeedbackState, FeedbackType, LocalModelProviderKind, ModelDescriptor, ModelProviderSettings,
    ProviderConnectionStatus, ReviewFeedback, ReviewProfileItem,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentFeedbackOutput {
    feedback: Vec<AgentFeedbackItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentFeedbackItem {
    title: String,
    feedback_type: Option<String>,
    severity: String,
    file: Option<String>,
    line: Option<u32>,
    body: String,
    suggested_action: String,
    confidence: Option<String>,
    evidence: Vec<String>,
    limitations: Vec<String>,
    quoted_code: Option<String>,
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

pub async fn run_review_pass(
    provider: &ModelProviderSettings,
    profile: &ReviewProfileItem,
    change_set: &ChangeSetSnapshot,
    file: &ChangedFile,
    pass_index: usize,
) -> Result<Vec<ReviewFeedback>, String> {
    let model = provider
        .selected_model_id
        .clone()
        .ok_or_else(|| "No model selected.".to_string())?;
    let base_url = openai_base_url(provider);
    let prompt = review_prompt(profile, change_set, file);
    let raw = run_rig_agent(&base_url, &model, &prompt).await?;
    let parsed = parse_json_from_model(&raw)?;
    let pass_id = format!("pass-{}-{}", profile.id, pass_index + 1);
    let created_at = now_iso();

    Ok(parsed
        .feedback
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            feedback_from_agent_item(
                item,
                provider,
                profile,
                file,
                &model,
                &pass_id,
                &created_at,
                index,
            )
        })
        .collect())
}

async fn run_rig_agent(base_url: &str, model: &str, prompt: &str) -> Result<String, String> {
    use rig::{
        agent::AgentBuilder, client::CompletionClient, completion::Prompt, providers::openai,
    };

    let client = openai::CompletionsClient::builder()
        .api_key("local-review")
        .base_url(base_url)
        .build()
        .map_err(|error| error.to_string())?;
    let model = client.completion_model(model);
    let agent = AgentBuilder::new(model)
        .preamble("You are a local code review agent. Return only valid JSON matching the requested schema. Do not use markdown fences.")
        .temperature(0.1)
        .build();

    let response = agent
        .prompt(prompt.to_string())
        .await
        .map_err(|error| error.to_string())?;
    Ok(response.to_string())
}

fn openai_base_url(provider: &ModelProviderSettings) -> String {
    match provider.kind {
        LocalModelProviderKind::Ollama => {
            format!("{}/v1", provider.base_url.trim_end_matches('/'))
        }
        LocalModelProviderKind::LmStudio => provider.base_url.trim_end_matches('/').to_string(),
    }
}

fn review_prompt(
    profile: &ReviewProfileItem,
    change_set: &ChangeSetSnapshot,
    file: &ChangedFile,
) -> String {
    format!(
        "Review one file from a Local Review session.\n\nProfile: {}\nCriteria: {}\nProfile prompt: {}\nRepository: {}\nFile: {}\nAdditions: {}\nDeletions: {}\n\nRules:\n- Review only the provided changed file/hunks.\n- Produce concise, actionable feedback anchored to the change set.\n- Return no feedback if there is no meaningful issue.\n- Inline feedback must use a line present in the changed hunk.\n- Do not invent files, tests, or commands.\n\nChanged hunks:\n{}\n\nReturn JSON with this shape: {{\"feedback\":[...]}}.",
        profile.name,
        profile.criteria.join(", "),
        profile.prompt,
        change_set.repository_path,
        file.path,
        file.additions,
        file.deletions,
        render_hunks(file)
    )
}

fn render_hunks(file: &ChangedFile) -> String {
    file.hunks
        .iter()
        .map(|hunk| {
            let lines = hunk
                .lines
                .iter()
                .map(|line| {
                    let prefix = match line.kind {
                        ChangeLineKind::Added => "+",
                        ChangeLineKind::Removed => "-",
                        ChangeLineKind::Context => " ",
                    };
                    let number = line.new_line_number.or(line.old_line_number).unwrap_or(0);
                    format!("{prefix}{number}: {}", line.content)
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "@@ -{} +{} @@\n{}",
                hunk.old_start_line, hunk.new_start_line, lines
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn parse_json_from_model(raw: &str) -> Result<AgentFeedbackOutput, String> {
    let trimmed = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    serde_json::from_str(trimmed)
        .map_err(|error| format!("Invalid model JSON: {error}; raw: {raw}"))
}

fn feedback_from_agent_item(
    item: AgentFeedbackItem,
    provider: &ModelProviderSettings,
    profile: &ReviewProfileItem,
    file: &ChangedFile,
    model: &str,
    pass_id: &str,
    created_at: &str,
    index: usize,
) -> ReviewFeedback {
    let requested_file = item.file.unwrap_or_else(|| file.path.clone());
    let location = item
        .line
        .filter(|line| line_exists_in_file(file, *line))
        .map(|line| CodeLocation {
            file_path: requested_file.clone(),
            start_line: line,
            end_line: line,
            side: "new".to_string(),
        });
    let feedback_type = if location.is_some() {
        FeedbackType::Inline
    } else if item.feedback_type.as_deref() == Some("inline") {
        FeedbackType::Inline
    } else {
        FeedbackType::Summary
    };
    let evidence = if item.evidence.is_empty() {
        vec![format!("{} changed in current change set.", file.path)]
    } else {
        item.evidence
    };
    let limited_context = item.limitations.iter().any(|value| {
        value.to_lowercase().contains("limited") || value.to_lowercase().contains("not inspect")
    });

    ReviewFeedback {
        id: format!("{}-{}-{}", pass_id, file.path.replace('/', "-"), index + 1),
        title: item.title,
        feedback_type,
        severity: parse_severity(&item.severity),
        state: FeedbackState::Draft,
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        pass_id: pass_id.to_string(),
        file: requested_file,
        line: location.as_ref().map(|value| value.start_line),
        body: item.body.clone(),
        editable_comment: item.body,
        suggested_action: item.suggested_action,
        confidence: item.confidence.unwrap_or_else(|| "medium".to_string()),
        limited_context,
        quoted_code: item.quoted_code,
        evidence,
        limitations: item.limitations,
        code_location: location,
        related_files: vec![file.path.clone()],
        model_provider: provider.id.clone(),
        model: model.to_string(),
        created_at: created_at.to_string(),
    }
}

fn parse_severity(value: &str) -> FeedbackSeverity {
    match value {
        "blocking" => FeedbackSeverity::Blocking,
        "important" => FeedbackSeverity::Important,
        "question" => FeedbackSeverity::Question,
        "nitpick" => FeedbackSeverity::Nitpick,
        _ => FeedbackSeverity::Suggestion,
    }
}

fn line_exists_in_file(file: &ChangedFile, line: u32) -> bool {
    file.hunks.iter().any(|hunk| {
        hunk.lines
            .iter()
            .any(|candidate| candidate.new_line_number == Some(line))
    })
}
