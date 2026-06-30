use crate::domain::{
    ChangedFile, CodeLocation, FeedbackSeverity, FeedbackState, FeedbackType,
    ModelProviderSettings, ReviewFeedback, ReviewProfileItem,
};

use super::types::AgentFeedbackItem;

pub(super) fn feedback_from_agent_item(
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

pub(super) fn code_location_from_agent_item(
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

pub(super) fn first_non_empty(values: &[String]) -> Option<String> {
    values
        .iter()
        .find(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string())
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
