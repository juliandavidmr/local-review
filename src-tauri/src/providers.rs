use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use crate::domain::{
    now_iso, ChangeLineKind, ChangeSetSnapshot, ChangeSource, ChangedFile, CodeLocation,
    FeedbackSeverity, FeedbackState, FeedbackType, LocalModelProviderKind, ModelDescriptor,
    ModelProviderSettings, ProviderConnectionStatus, ReviewFeedback, ReviewProfileItem,
};
use serde::Deserialize;
use serde_json::json;

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
    start_line: Option<u32>,
    end_line: Option<u32>,
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

struct ReviewAgentResult {
    raw: String,
    exploration_requests: u32,
}

pub(crate) struct ReviewPassResult {
    pub feedback: Vec<ReviewFeedback>,
    pub exploration_requests: u32,
}

#[derive(Clone)]
struct ToolUsageHook {
    exploration_requests: Arc<AtomicU32>,
}

impl<M: rig::completion::CompletionModel> rig::agent::PromptHook<M> for ToolUsageHook {
    async fn on_tool_call(
        &self,
        _tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        _args: &str,
    ) -> rig::agent::ToolCallHookAction {
        self.exploration_requests.fetch_add(1, Ordering::SeqCst);
        rig::agent::ToolCallHookAction::Continue
    }
}

#[derive(Clone, Debug, thiserror::Error)]
enum ReviewToolError {
    #[error("{0}")]
    Rejected(String),
    #[error("Could not read repository context.")]
    ReadFailed,
}

#[derive(Clone)]
struct ReadRepositoryFileTool {
    repository_path: String,
}

impl ReadRepositoryFileTool {
    fn new(repository_path: impl Into<String>) -> Self {
        Self {
            repository_path: repository_path.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadRepositoryFileArgs {
    path: String,
    start_line: Option<usize>,
    end_line: Option<usize>,
    context_lines: Option<usize>,
}

impl rig::tool::Tool for ReadRepositoryFileTool {
    const NAME: &'static str = "read_repository_file";
    type Error = ReviewToolError;
    type Args = ReadRepositoryFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read a bounded line range from a non-sensitive repository file. Use this for surrounding function, caller, type, or test context before making a review claim.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository-relative file path."
                    },
                    "startLine": {
                        "type": "integer",
                        "description": "1-based starting line. Omit to start at the beginning."
                    },
                    "endLine": {
                        "type": "integer",
                        "description": "1-based ending line. Omit to read from startLine through a bounded window."
                    },
                    "contextLines": {
                        "type": "integer",
                        "description": "Extra lines to include before and after the requested range, capped by the tool."
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let file_path = safe_repository_file(&self.repository_path, &args.path)?;
        let raw = fs::read_to_string(&file_path).map_err(|_| ReviewToolError::ReadFailed)?;
        let lines = raw.lines().collect::<Vec<_>>();
        if lines.is_empty() {
            return Ok(format!("{} is empty.", args.path));
        }

        let requested_start = args.start_line.unwrap_or(1).max(1);
        let requested_end = args
            .end_line
            .unwrap_or_else(|| requested_start.saturating_add(80))
            .max(requested_start);
        let context = args.context_lines.unwrap_or(8).min(30);
        let start = requested_start.saturating_sub(context).max(1);
        let end = requested_end.saturating_add(context).min(lines.len());
        let max_lines = 220usize;
        let end = end.min(start.saturating_add(max_lines).saturating_sub(1));

        let body = lines[start - 1..end]
            .iter()
            .enumerate()
            .map(|(index, line)| format!("{:>5}: {}", start + index, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!("{}:{}-{}\n{}", args.path, start, end, body))
    }
}

#[derive(Clone)]
struct SearchRepositoryTool {
    repository_path: String,
}

impl SearchRepositoryTool {
    fn new(repository_path: impl Into<String>) -> Self {
        Self {
            repository_path: repository_path.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchRepositoryArgs {
    query: String,
    file_glob: Option<String>,
    max_results: Option<usize>,
}

impl rig::tool::Tool for SearchRepositoryTool {
    const NAME: &'static str = "search_repository";
    type Error = ReviewToolError;
    type Args = SearchRepositoryArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search non-sensitive repository files for a literal string. Use this to find callers, definitions, tests, configuration, and repeated patterns before writing review feedback.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Literal text to search for. Keep it specific."
                    },
                    "fileGlob": {
                        "type": "string",
                        "description": "Optional simple glob such as *.ts, **/*.rs, src/**, or an exact path."
                    },
                    "maxResults": {
                        "type": "integer",
                        "description": "Maximum matches to return, capped by the tool."
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let query = args.query.trim();
        if query.len() < 2 {
            return Err(ReviewToolError::Rejected(
                "Search query must contain at least two non-space characters.".to_string(),
            ));
        }

        let root = canonical_repository_root(&self.repository_path)?;
        let max_results = args.max_results.unwrap_or(24).clamp(1, 50);
        let mut matches = Vec::new();
        search_repository_files(
            &root,
            &root,
            query,
            args.file_glob.as_deref(),
            max_results,
            &mut matches,
        );

        if matches.is_empty() {
            Ok(format!("No matches for {query:?}."))
        } else {
            Ok(matches.join("\n"))
        }
    }
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
    repository_tools_enabled: bool,
) -> Result<ReviewPassResult, String> {
    let model = provider
        .selected_model_id
        .clone()
        .ok_or_else(|| "No model selected.".to_string())?;
    let base_url = openai_base_url(provider);
    let prompt = review_prompt(profile, change_set, file, repository_tools_enabled);
    let agent_result = run_rig_agent(
        &base_url,
        &model,
        &prompt,
        &change_set.repository_path,
        repository_tools_enabled,
    )
    .await?;
    let parsed = match parse_json_from_model(&agent_result.raw) {
        Ok(parsed) => parsed,
        Err(parse_error) => {
            eprintln!(
                "[local-review-pass] json_repair_start file={} profile={} error={}",
                file.path, profile.name, parse_error
            );
            let repaired = repair_model_json(&base_url, &model, &agent_result.raw).await?;
            parse_json_from_model(&repaired).map_err(|repair_error| {
                format!(
                    "{parse_error}; repair also failed: {repair_error}; repaired raw: {repaired}"
                )
            })?
        }
    };
    let pass_id = format!("pass-{}-{}", profile.id, pass_index + 1);
    let created_at = now_iso();

    let feedback = parsed
        .feedback
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if let Some(reason) = agent_item_quality_issue(&item, file) {
                eprintln!(
                    "[local-review-pass] feedback_rejected file={} profile={} reason={}",
                    file.path, profile.name, reason
                );
                None
            } else {
                Some(feedback_from_agent_item(
                    item,
                    provider,
                    profile,
                    file,
                    &model,
                    &pass_id,
                    &created_at,
                    index,
                    agent_result.exploration_requests > 0,
                ))
            }
        })
        .collect();

    Ok(ReviewPassResult {
        feedback,
        exploration_requests: agent_result.exploration_requests,
    })
}

async fn run_rig_agent(
    base_url: &str,
    model: &str,
    prompt: &str,
    repository_path: &str,
    repository_tools_enabled: bool,
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

async fn repair_model_json(base_url: &str, model: &str, raw: &str) -> Result<String, String> {
    let prompt = format!(
        "Repair this malformed JSON response from a code review model.\n\nRules:\n- Return only valid JSON.\n- Preserve all feedback content.\n- The root must be an object with a feedback array.\n- Each feedback item should use keys title, severity, line, body, suggestedAction, evidence, limitations.\n- Do not add markdown fences.\n\nMalformed response:\n{}",
        raw
    );

    let result = run_rig_agent(base_url, model, &prompt, ".", false).await?;
    Ok(result.raw)
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
    repository_tools_enabled: bool,
) -> String {
    let tool_instruction = if repository_tools_enabled {
        "- Use read_repository_file or search_repository before making claims about callers, definitions, tests, configuration, or behavior outside the visible hunk."
    } else {
        "- Repository exploration tools are disabled for this pass; return limitations instead of guessing about callers, definitions, tests, configuration, or behavior outside the visible hunk."
    };

    format!(
        "Review one file from a Local Review session.\n\nProfile: {}\nCriteria: {}\nProfile prompt: {}\nRepository: {}\nFile: {}\nAdditions: {}\nDeletions: {}\n\nReview standard:\n- Produce only comments that would be credible in a human code review.\n- Each comment must identify a concrete defect, regression risk, missing invariant, unsafe edge case, or architecture boundary violation.\n- Each comment must explain the affected scenario and why the changed code creates the risk.\n- Each comment must include exact evidence from the changed code and, when needed, repository context gathered with tools.\n{}\n- Return no feedback for generic maintainability advice, speculative concerns, style preferences, or comments that only say to add tests without naming the missing behavior.\n- Do not invent files, tests, commands, product requirements, or repository conventions.\n- Inline feedback must use a changed new-line number or changed new-line range from the hunk.\n- The body must be self-contained and publication-ready because it is what may be posted to GitHub.\n- Return only JSON. Do not wrap it in markdown fences.\n\nEach feedback item must include these keys:\n- title: short specific string\n- severity: one of blocking, important, suggestion, question, nitpick\n- line: changed new-line number for single-line inline feedback\n- startLine: changed new-line number for the first line of a multi-line range, optional when line is present\n- endLine: changed new-line number for the last line of a multi-line range, optional when line is present\n- body: 2-5 sentence complete review comment with the issue, impact, and fix direction\n- suggestedAction: concrete action the author can take\n- confidence: high, medium, or low\n- evidence: array of specific evidence strings, including file:line references or tool-derived observations\n- limitations: array of specific limitations, empty array if none\n\nExample response:\n{{\"feedback\":[{{\"title\":\"Preserve validation before saving settings\",\"severity\":\"important\",\"line\":42,\"body\":\"This path now writes the provider settings before checking whether a selected model exists. If the model probe fails, the app can persist an unusable provider configuration and later review sessions will fail before they start. Keep the validation before the write, or roll back the saved settings when the probe returns no model.\",\"suggestedAction\":\"Move the selected-model validation before persistence, or make the save transactional so invalid provider settings are not stored.\",\"confidence\":\"high\",\"evidence\":[\"src-tauri/src/store.rs:42 saves settings before provider validation\",\"The changed branch handles probe errors after persistence\"],\"limitations\":[]}}]}}\n\nChanged hunks:\n{}\n\nExpanded current-file context around the changed hunks:\n{}\n\nReturn JSON now.",
        profile.name,
        profile.criteria.join(", "),
        profile.prompt,
        change_set.repository_path,
        file.path,
        file.additions,
        file.deletions,
        tool_instruction,
        render_hunks(file),
        render_expanded_file_context(change_set, file)
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

fn render_expanded_file_context(change_set: &ChangeSetSnapshot, file: &ChangedFile) -> String {
    if is_sensitive_path(&file.path) {
        return "Additional context skipped because the path may contain sensitive data."
            .to_string();
    }

    let Ok(raw) = read_review_file_for_context(change_set, file) else {
        return "Current file context unavailable, likely because the file was deleted or is outside the repository.".to_string();
    };
    let lines = raw.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return "Current file is empty.".to_string();
    }

    let mut ranges = file
        .hunks
        .iter()
        .filter_map(|hunk| {
            let changed_lines = hunk
                .lines
                .iter()
                .filter(|line| matches!(line.kind, ChangeLineKind::Added | ChangeLineKind::Context))
                .filter_map(|line| line.new_line_number)
                .collect::<Vec<_>>();
            let min = changed_lines.iter().min().copied()? as usize;
            let max = changed_lines.iter().max().copied()? as usize;
            Some((
                min.saturating_sub(25).max(1),
                max.saturating_add(25).min(lines.len()),
            ))
        })
        .collect::<Vec<_>>();

    if ranges.is_empty() {
        return "No current-line context is available for this file.".to_string();
    }

    ranges.sort_by_key(|range| range.0);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        if let Some(last) = merged.last_mut() {
            if start <= last.1.saturating_add(5) {
                last.1 = last.1.max(end);
                continue;
            }
        }
        merged.push((start, end));
    }

    let mut rendered = Vec::new();
    let mut rendered_lines = 0usize;
    for (start, end) in merged {
        if rendered_lines >= 260 {
            rendered.push("...additional context omitted...".to_string());
            break;
        }
        let end = end.min(start.saturating_add(260 - rendered_lines).saturating_sub(1));
        let body = lines[start - 1..end]
            .iter()
            .enumerate()
            .map(|(index, line)| format!("{:>5}: {}", start + index, line))
            .collect::<Vec<_>>()
            .join("\n");
        rendered.push(format!("{}:{}-{}\n{}", file.path, start, end, body));
        rendered_lines += end.saturating_sub(start).saturating_add(1);
    }

    rendered.join("\n\n")
}

fn read_review_file_for_context(
    change_set: &ChangeSetSnapshot,
    file: &ChangedFile,
) -> Result<String, ReviewToolError> {
    match &change_set.source {
        ChangeSource::StagedChanges { .. } => read_git_object(
            &change_set.repository_path,
            &format!(":{}", file.path),
            &file.path,
        ),
        ChangeSource::Commit { commit_sha, .. } => read_git_object(
            &change_set.repository_path,
            &format!("{commit_sha}:{}", file.path),
            &file.path,
        ),
        ChangeSource::CompareRefs { head_ref, .. } => read_git_object(
            &change_set.repository_path,
            &format!("{head_ref}:{}", file.path),
            &file.path,
        ),
        ChangeSource::CurrentBranch { .. } => read_git_object(
            &change_set.repository_path,
            &format!("HEAD:{}", file.path),
            &file.path,
        ),
        ChangeSource::WorkingTree { .. } | ChangeSource::UnstagedChanges { .. } => {
            let path = safe_repository_file(&change_set.repository_path, &file.path)?;
            fs::read_to_string(path).map_err(|_| ReviewToolError::ReadFailed)
        }
    }
}

fn read_git_object(
    repository_path: &str,
    object: &str,
    file_path: &str,
) -> Result<String, ReviewToolError> {
    if is_sensitive_path(file_path) {
        return Err(ReviewToolError::Rejected(
            "Sensitive files are not available to review tools.".to_string(),
        ));
    }

    let output = Command::new("git")
        .current_dir(repository_path)
        .args(["show", object])
        .output()
        .map_err(|_| ReviewToolError::ReadFailed)?;
    if !output.status.success() {
        return Err(ReviewToolError::ReadFailed);
    }

    String::from_utf8(output.stdout).map_err(|_| ReviewToolError::ReadFailed)
}

fn safe_repository_file(
    repository_path: &str,
    relative_path: &str,
) -> Result<PathBuf, ReviewToolError> {
    if relative_path.trim().is_empty() || Path::new(relative_path).is_absolute() {
        return Err(ReviewToolError::Rejected(
            "Use a non-empty repository-relative path.".to_string(),
        ));
    }
    if is_sensitive_path(relative_path) {
        return Err(ReviewToolError::Rejected(
            "Sensitive files are not available to review tools.".to_string(),
        ));
    }

    let root = canonical_repository_root(repository_path)?;
    let path = root.join(relative_path);
    let canonical = fs::canonicalize(path).map_err(|_| {
        ReviewToolError::Rejected("Requested file is not readable in the repository.".to_string())
    })?;

    if !canonical.starts_with(&root) || !canonical.is_file() {
        return Err(ReviewToolError::Rejected(
            "Requested path is outside the repository or is not a file.".to_string(),
        ));
    }

    Ok(canonical)
}

fn canonical_repository_root(repository_path: &str) -> Result<PathBuf, ReviewToolError> {
    fs::canonicalize(repository_path)
        .map_err(|_| ReviewToolError::Rejected("Repository path is not readable.".to_string()))
}

fn search_repository_files(
    root: &Path,
    directory: &Path,
    query: &str,
    file_glob: Option<&str>,
    max_results: usize,
    matches: &mut Vec<String>,
) {
    if matches.len() >= max_results {
        return;
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let query_lower = query.to_lowercase();

    for entry in entries.flatten() {
        if matches.len() >= max_results {
            return;
        }
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .ok()
            .and_then(|path| path.to_str())
            .unwrap_or("");
        if relative.is_empty() || should_skip_repository_path(relative, path.is_dir()) {
            continue;
        }

        if path.is_dir() {
            search_repository_files(root, &path, query, file_glob, max_results, matches);
            continue;
        }

        if file_glob
            .map(|glob| !path_matches_simple_glob(relative, glob))
            .unwrap_or(false)
        {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.len() > 512 * 1024 {
            continue;
        }

        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in raw.lines().enumerate() {
            if matches.len() >= max_results {
                return;
            }
            if line.to_lowercase().contains(&query_lower) {
                matches.push(format!("{}:{}: {}", relative, index + 1, line.trim()));
            }
        }
    }
}

fn should_skip_repository_path(relative_path: &str, is_dir: bool) -> bool {
    let normalized = relative_path.replace('\\', "/");
    let first = normalized.split('/').next().unwrap_or("");
    if is_sensitive_path(&normalized) {
        return true;
    }
    if is_dir {
        matches!(
            first,
            ".git" | "node_modules" | "target" | "dist" | "build" | ".next" | ".turbo" | ".cache"
        )
    } else {
        false
    }
}

fn is_sensitive_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_lowercase();
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());
    file_name == ".env"
        || file_name.starts_with(".env.")
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || file_name.ends_with(".crt")
        || file_name.ends_with(".cer")
        || file_name.ends_with(".dump")
        || file_name.ends_with(".sql")
        || file_name.contains("id_rsa")
        || file_name.contains("secret")
        || file_name.contains("credential")
}

fn path_matches_simple_glob(path: &str, glob: &str) -> bool {
    if glob.trim().is_empty() || glob == "*" {
        return true;
    }
    if glob.ends_with("/**") {
        return path.starts_with(glob.trim_end_matches("/**"));
    }
    if glob.starts_with("**/*.") {
        return path.ends_with(glob.trim_start_matches("**/*"));
    }
    if glob.starts_with("*.") {
        return path.ends_with(glob.trim_start_matches('*'));
    }
    path == glob
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
    used_repository_exploration: bool,
) -> ReviewFeedback {
    let requested_file = item
        .file
        .as_ref()
        .filter(|candidate| *candidate == &file.path)
        .cloned()
        .unwrap_or_else(|| file.path.clone());
    let location = code_location_from_agent_item(&item, file, &requested_file);
    let body = first_non_empty(&[item.body, item.message])
        .unwrap_or_else(|| "The model returned feedback without a body.".to_string());
    let title = if item.title.trim().is_empty() {
        summarize_title(&body)
    } else {
        item.title
    };
    let suggested_action = item.suggested_action.trim().to_string();
    let feedback_type = if location.is_some() {
        FeedbackType::Inline
    } else {
        FeedbackType::Summary
    };
    let quoted_code = location
        .as_ref()
        .and_then(|value| get_range_content(file, value.start_line, value.end_line))
        .or(item.quoted_code);
    let evidence = item.evidence;
    let limited_context = item.limitations.iter().any(|value| {
        value.to_lowercase().contains("limited") || value.to_lowercase().contains("not inspect")
    }) || !used_repository_exploration
        && body_mentions_external_context(&body);
    let editable_comment = publication_comment(&body, &suggested_action);

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
        editable_comment,
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

fn agent_item_quality_issue(item: &AgentFeedbackItem, file: &ChangedFile) -> Option<String> {
    let Some(body) = first_non_empty(&[item.body.clone(), item.message.clone()]) else {
        return Some("missing_body".to_string());
    };
    let action = item.suggested_action.trim();

    if body.split_whitespace().count() < 22 {
        return Some("body_too_short".to_string());
    }
    if action.split_whitespace().count() < 6 {
        return Some("suggested_action_too_short".to_string());
    }
    if item.evidence.is_empty() || item.evidence.iter().all(|value| value.trim().is_empty()) {
        return Some("missing_specific_evidence".to_string());
    }
    if looks_like_generic_review_text(&body) || looks_like_generic_review_text(action) {
        return Some("generic_review_text".to_string());
    }
    if item
        .limitations
        .iter()
        .any(|value| value.to_lowercase().contains("unable to determine"))
    {
        return Some("model_reports_insufficient_context".to_string());
    }
    if (item.line.is_some() || item.start_line.is_some() || item.end_line.is_some())
        && code_location_from_agent_item(item, file, &file.path).is_none()
    {
        return Some("invalid_changed_line_range".to_string());
    }

    None
}

fn looks_like_generic_review_text(value: &str) -> bool {
    let lower = value.to_lowercase();
    let generic_phrases = [
        "best practice",
        "code quality",
        "potential issue",
        "may cause issues",
        "might cause issues",
        "review this finding",
        "make sure to",
        "ensure that this",
        "consider adding tests",
        "could be improved",
    ];

    generic_phrases.iter().any(|phrase| lower.contains(phrase))
}

fn body_mentions_external_context(body: &str) -> bool {
    let lower = body.to_lowercase();
    let watched_words = [
        "caller", "callers", "test", "tests", "config", "schema", "api",
    ];

    lower
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| watched_words.contains(&word))
}

fn publication_comment(body: &str, suggested_action: &str) -> String {
    if suggested_action.trim().is_empty()
        || body
            .to_lowercase()
            .contains(&suggested_action.to_lowercase())
    {
        body.trim().to_string()
    } else {
        format!(
            "{}\n\nSuggested action: {}",
            body.trim(),
            suggested_action.trim()
        )
    }
}

fn code_location_from_agent_item(
    item: &AgentFeedbackItem,
    file: &ChangedFile,
    requested_file: &str,
) -> Option<CodeLocation> {
    let start_line = item.start_line.or(item.line)?;
    let end_line = item.end_line.unwrap_or(start_line);
    if start_line == 0 || end_line < start_line {
        return None;
    }
    if !range_exists_in_file(file, start_line, end_line) {
        return None;
    }

    Some(CodeLocation {
        file_path: requested_file.to_string(),
        start_line,
        end_line,
        side: "new".to_string(),
    })
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

fn range_exists_in_file(file: &ChangedFile, start_line: u32, end_line: u32) -> bool {
    (start_line..=end_line).all(|line| {
        file.hunks.iter().any(|hunk| {
            hunk.lines
                .iter()
                .any(|candidate| candidate.new_line_number == Some(line))
        })
    })
}

fn get_range_content(file: &ChangedFile, start_line: u32, end_line: u32) -> Option<String> {
    let mut lines = Vec::new();
    for line_number in start_line..=end_line {
        let line = file
            .hunks
            .iter()
            .flat_map(|hunk| hunk.lines.iter())
            .find(|candidate| candidate.new_line_number == Some(line_number))?;
        lines.push(line.content.clone());
    }

    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ChangeHunk, ChangeLine, ChangedFileStatus};

    #[test]
    fn rejects_generic_feedback_before_curation() {
        let item = AgentFeedbackItem {
            title: "Improve quality".to_string(),
            severity: "suggestion".to_string(),
            file: None,
            line: Some(10),
            start_line: None,
            end_line: None,
            body: "This could be improved because it may cause issues later, so consider adding tests and making sure the code follows best practices.".to_string(),
            message: String::new(),
            suggested_action: "Consider adding tests for this code path to improve quality.".to_string(),
            confidence: Some("low".to_string()),
            evidence: vec!["src/example.rs:10 changed".to_string()],
            limitations: vec![],
            quoted_code: None,
        };

        assert_eq!(
            agent_item_quality_issue(&item, &changed_file()),
            Some("generic_review_text".to_string())
        );
    }

    #[test]
    fn accepts_specific_multiline_feedback_and_preserves_publishable_comment() {
        let provider = ModelProviderSettings {
            id: "lm-studio".to_string(),
            kind: LocalModelProviderKind::LmStudio,
            name: "LM Studio".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            enabled: true,
            selected_model_id: Some("model".to_string()),
            use_for_human_tone_rewrite: false,
        };
        let profile = ReviewProfileItem {
            id: "correctness".to_string(),
            name: "Correctness".to_string(),
            scope: "Global".to_string(),
            scope_kind: crate::domain::ProfileScopeKind::Global,
            selected: true,
            enabled_by_default: true,
            criteria: vec!["Correctness".to_string()],
            file_globs: vec!["*".to_string()],
            prompt: "Find concrete regressions.".to_string(),
        };
        let item = AgentFeedbackItem {
            title: "Keep empty input validation".to_string(),
            severity: "important".to_string(),
            file: None,
            line: None,
            start_line: Some(10),
            end_line: Some(11),
            body: "The new branch returns success before checking whether the request body is empty. An empty payload can now reach the save path, which means callers get a successful response even though no usable settings were provided. Keep the empty-body validation ahead of the success return so invalid input still fails early.".to_string(),
            message: String::new(),
            suggested_action: "Move the empty-body validation before the success return and keep the error response for missing settings.".to_string(),
            confidence: Some("high".to_string()),
            evidence: vec![
                "src/example.rs:10-11 returns before validating the request body.".to_string(),
                "The changed lines are inside the save path.".to_string(),
            ],
            limitations: vec![],
            quoted_code: None,
        };
        let file = changed_file();

        assert_eq!(agent_item_quality_issue(&item, &file), None);

        let feedback = feedback_from_agent_item(
            item,
            &provider,
            &profile,
            &file,
            "model",
            "pass-1",
            "2026-06-30T00:00:00Z",
            0,
            true,
        );

        assert_eq!(feedback.line, Some(10));
        assert_eq!(
            feedback.code_location.as_ref().map(|loc| loc.end_line),
            Some(11)
        );
        assert!(feedback.editable_comment.contains("Suggested action:"));
        assert_eq!(
            feedback.quoted_code.as_deref(),
            Some("return Ok(())\nsave_settings(input)")
        );
    }

    fn changed_file() -> ChangedFile {
        ChangedFile {
            path: "src/example.rs".to_string(),
            previous_path: None,
            status: ChangedFileStatus::Modified,
            additions: 2,
            deletions: 1,
            is_generated: false,
            hunks: vec![ChangeHunk {
                id: "hunk-8-8".to_string(),
                old_start_line: 8,
                new_start_line: 8,
                lines: vec![
                    ChangeLine {
                        kind: ChangeLineKind::Context,
                        content: "fn save(input: Settings) -> Result<()> {".to_string(),
                        old_line_number: Some(8),
                        new_line_number: Some(8),
                    },
                    ChangeLine {
                        kind: ChangeLineKind::Removed,
                        content: "validate(input)?;".to_string(),
                        old_line_number: Some(9),
                        new_line_number: None,
                    },
                    ChangeLine {
                        kind: ChangeLineKind::Added,
                        content: "return Ok(())".to_string(),
                        old_line_number: None,
                        new_line_number: Some(10),
                    },
                    ChangeLine {
                        kind: ChangeLineKind::Added,
                        content: "save_settings(input)".to_string(),
                        old_line_number: None,
                        new_line_number: Some(11),
                    },
                ],
            }],
        }
    }
}
