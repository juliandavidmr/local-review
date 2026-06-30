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
    #[serde(default)]
    title: String,
    #[serde(default = "default_feedback_severity")]
    severity: String,
    file: Option<String>,
    line: Option<u32>,
    #[serde(default)]
    body: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    suggested_action: String,
    confidence: Option<String>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    limitations: Vec<String>,
    quoted_code: Option<String>,
}

fn default_feedback_severity() -> String {
    "suggestion".to_string()
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
    let parsed = match parse_json_from_model(&raw) {
        Ok(parsed) => parsed,
        Err(parse_error) => {
            eprintln!(
                "[local-review-pass] json_repair_start file={} profile={} error={}",
                file.path, profile.name, parse_error
            );
            let repaired = repair_model_json(&base_url, &model, &raw).await?;
            parse_json_from_model(&repaired).map_err(|repair_error| {
                format!(
                    "{parse_error}; repair also failed: {repair_error}; repaired raw: {repaired}"
                )
            })?
        }
    };
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

async fn repair_model_json(base_url: &str, model: &str, raw: &str) -> Result<String, String> {
    let prompt = format!(
        "Repair this malformed JSON response from a code review model.\n\nRules:\n- Return only valid JSON.\n- Preserve all feedback content.\n- The root must be an object with a feedback array.\n- Each feedback item should use keys title, severity, line, body, suggestedAction, evidence, limitations.\n- Do not add markdown fences.\n\nMalformed response:\n{}",
        raw
    );

    run_rig_agent(base_url, model, &prompt).await
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
        "Review one file from a Local Review session.\n\nProfile: {}\nCriteria: {}\nProfile prompt: {}\nRepository: {}\nFile: {}\nAdditions: {}\nDeletions: {}\n\nRules:\n- Review only the provided changed file/hunks.\n- Produce concise, actionable feedback anchored to the change set.\n- Return no feedback if there is no meaningful issue.\n- Inline feedback must use a line present in the changed hunk.\n- Do not invent files, tests, or commands.\n- Return only JSON. Do not wrap it in markdown fences.\n\nEach feedback item must include these keys:\n- title: short string\n- severity: one of blocking, important, suggestion, question, nitpick\n- line: changed new-line number when inline, otherwise omit\n- body: complete review comment\n- suggestedAction: concrete action for the author\n- evidence: array of short strings\n- limitations: array of short strings, empty array if none\n\nExample response:\n{{\"feedback\":[{{\"title\":\"Validate accepted MIME types\",\"severity\":\"suggestion\",\"line\":2,\"body\":\"The MIME type list is now manually maintained, so adding an invalid value later would only fail at runtime.\",\"suggestedAction\":\"Type the list with a MIME-type union or derive it from a single validated source.\",\"evidence\":[\"The changed constant defines accepted video MIME types.\"],\"limitations\":[]}}]}}\n\nChanged hunks:\n{}\n\nReturn JSON now.",
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
    let requested_file = item
        .file
        .filter(|candidate| candidate == &file.path)
        .unwrap_or_else(|| file.path.clone());
    let body = first_non_empty(&[item.body, item.message])
        .unwrap_or_else(|| "The model returned feedback without a body.".to_string());
    let title = if item.title.trim().is_empty() {
        summarize_title(&body)
    } else {
        item.title
    };
    let suggested_action = first_non_empty(&[item.suggested_action]).unwrap_or_else(|| {
        "Review this finding and decide whether to adjust the changed code.".to_string()
    });
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
    } else {
        FeedbackType::Summary
    };
    let quoted_code = location
        .as_ref()
        .and_then(|value| get_line_content(file, value.start_line))
        .or(item.quoted_code);
    let evidence = location
        .as_ref()
        .and_then(|value| {
            quoted_code.as_ref().map(|code| {
                vec![format!(
                    "{}:{}\n{}",
                    value.file_path, value.start_line, code
                )]
            })
        })
        .unwrap_or_else(|| {
            if item.evidence.is_empty() {
                vec![format!("{} changed in current change set.", file.path)]
            } else {
                item.evidence
            }
        });
    let limited_context = item.limitations.iter().any(|value| {
        value.to_lowercase().contains("limited") || value.to_lowercase().contains("not inspect")
    });

    ReviewFeedback {
        id: format!("{}-{}-{}", pass_id, file.path.replace('/', "-"), index + 1),
        title,
        feedback_type,
        severity: parse_severity(&item.severity),
        state: FeedbackState::Draft,
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        pass_id: pass_id.to_string(),
        file: requested_file,
        line: location.as_ref().map(|value| value.start_line),
        body: body.clone(),
        editable_comment: body,
        suggested_action,
        confidence: item.confidence.unwrap_or_else(|| "medium".to_string()),
        limited_context,
        quoted_code,
        evidence,
        limitations: item.limitations,
        code_location: location,
        related_files: vec![file.path.clone()],
        model_provider: provider.id.clone(),
        model: model.to_string(),
        created_at: created_at.to_string(),
    }
}

fn first_non_empty(values: &[String]) -> Option<String> {
    values
        .iter()
        .find(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string())
}

fn summarize_title(body: &str) -> String {
    let first_sentence = body
        .split(['.', '\n'])
        .next()
        .unwrap_or("Review generated feedback")
        .trim();

    if first_sentence.is_empty() {
        "Review generated feedback".to_string()
    } else {
        first_sentence.chars().take(80).collect()
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

fn get_line_content(file: &ChangedFile, line: u32) -> Option<String> {
    file.hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .find(|candidate| candidate.new_line_number == Some(line))
        .map(|candidate| candidate.content.clone())
}
