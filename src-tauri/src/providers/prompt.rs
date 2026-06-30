use std::{fs, process::Command};

use crate::domain::{
    ChangeLineKind, ChangeSetSnapshot, ChangeSource, ChangedFile, ReviewProfileItem,
};

use super::tools::{is_sensitive_path, safe_repository_file, ReviewToolError};

pub(super) fn review_prompt(
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
        "Review one file from a Local Review session.\n\nProfile: {}\nCriteria: {}\nProfile prompt: {}\nRepository: {}\nFile: {}\nAdditions: {}\nDeletions: {}\n\nReview standard:\n- Produce only comments that would be credible in a human code review.\n- Each comment must identify a concrete defect, regression risk, missing invariant, unsafe edge case, or architecture boundary violation.\n- Each comment must explain the affected scenario and why the changed code creates the risk.\n- Each comment must include exact evidence from the changed code and, when needed, repository context gathered with tools.\n{}\n- Return no feedback for generic maintainability advice, speculative concerns, style preferences, or comments that only say to add tests without naming the missing behavior.\n- Return no feedback when the only concern is to verify that a dependency, import, enum, map, constant, or configuration remains complete/correct; first inspect it and name the exact missing or wrong entry.\n- Treat limitations as reasons not to comment. Do not turn uncertainty such as \"could be incomplete\", \"import path must be correct\", or \"verify this list\" into review feedback.\n- Do not invent files, tests, commands, product requirements, or repository conventions.\n- Inline feedback must use a changed new-line number or changed new-line range from the hunk.\n- The body must be self-contained and publication-ready because it is what may be posted to GitHub.\n- Return only JSON. Do not wrap it in markdown fences.\n\nEach feedback item must include these keys:\n- title: short specific string\n- severity: one of blocking, important, suggestion, question, nitpick\n- line: changed new-line number for single-line inline feedback\n- startLine: changed new-line number for the first line of a multi-line range, optional when line is present\n- endLine: changed new-line number for the last line of a multi-line range, optional when line is present\n- body: 2-5 sentence complete review comment with the issue, impact, and fix direction\n- suggestedAction: concrete action the author can take\n- confidence: high, medium, or low\n- evidence: array of specific evidence strings, including file:line references or tool-derived observations\n- limitations: array of specific limitations, empty array if none\n\nExample response:\n{{\"feedback\":[{{\"title\":\"Preserve validation before saving settings\",\"severity\":\"important\",\"line\":42,\"body\":\"This path now writes the provider settings before checking whether a selected model exists. If the model probe fails, the app can persist an unusable provider configuration and later review sessions will fail before they start. Keep the validation before the write, or roll back the saved settings when the probe returns no model.\",\"suggestedAction\":\"Move the selected-model validation before persistence, or make the save transactional so invalid provider settings are not stored.\",\"confidence\":\"high\",\"evidence\":[\"src-tauri/src/store.rs:42 saves settings before provider validation\",\"The changed branch handles probe errors after persistence\"],\"limitations\":[]}}]}}\n\nChanged hunks:\n{}\n\nExpanded current-file context around the changed hunks:\n{}\n\nReturn JSON now.",
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

pub(super) fn repository_grounding_prompt(
    profile: &ReviewProfileItem,
    change_set: &ChangeSetSnapshot,
    file: &ChangedFile,
    previous_response: &str,
) -> String {
    format!(
        "Your previous draft review for this file contained one or more claims that require repository grounding.\n\nYou must now use repository tools before returning final JSON:\n- Use search_repository to find the definition of every imported symbol, constant, enum, map, config object, helper, type, or caller that your feedback depends on.\n- Use read_repository_file to inspect the defining file or surrounding implementation before deciding whether there is a real defect.\n- If the concern is only \"verify this import\", \"ensure this list is complete\", \"could be incomplete\", or \"the import path must be correct\", inspect the relevant code. Return feedback only if you can name the exact missing entry, wrong import, broken contract, or caller scenario.\n- If repository exploration shows the changed code is valid or you cannot identify a concrete defect, return {{\"feedback\":[]}}.\n- Evidence for each returned item must include the tool-inspected file path and line or range that proves the defect.\n\nProfile: {}\nCriteria: {}\nRepository: {}\nFile: {}\n\nPrevious draft response to ground or discard:\n{}\n\nChanged hunks:\n{}\n\nExpanded current-file context around the changed hunks:\n{}\n\nReturn only final JSON now.",
        profile.name,
        profile.criteria.join(", "),
        change_set.repository_path,
        file.path,
        previous_response,
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
