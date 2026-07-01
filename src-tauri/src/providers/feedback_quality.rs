use crate::domain::ChangedFile;

use super::{
    feedback_mapping::{code_location_from_agent_item, first_non_empty},
    types::AgentFeedbackItem,
};

pub(super) fn agent_item_quality_issue(
    item: &AgentFeedbackItem,
    file: &ChangedFile,
    used_repository_exploration: bool,
) -> Option<String> {
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
    if looks_like_verification_task_without_defect_evidence(item, &body, action) {
        return Some("verification_task_without_defect_evidence".to_string());
    }
    if !used_repository_exploration && feedback_requires_repository_exploration(item) {
        return Some("repository_exploration_required".to_string());
    }
    if item
        .limitations
        .iter()
        .any(|value| value.to_lowercase().contains("unable to determine"))
    {
        return Some("model_reports_insufficient_context".to_string());
    }
    if item
        .file
        .as_ref()
        .is_some_and(|requested_file| requested_file != &file.path)
    {
        return Some("feedback_for_different_file".to_string());
    }
    if item.line.is_none() && item.start_line.is_none() {
        return Some("missing_changed_line_range".to_string());
    }
    if code_location_from_agent_item(item, file, &file.path).is_none() {
        return Some("invalid_changed_line_range".to_string());
    }

    None
}

pub(super) fn feedback_requires_repository_exploration(item: &AgentFeedbackItem) -> bool {
    let body = first_non_empty(&[item.body.clone(), item.message.clone()]).unwrap_or_default();
    let combined = repository_claim_context(item, &body, &item.suggested_action);

    mentions_external_review_target(&combined)
        || claims_external_symbol_is_missing(&combined)
        || item
            .limitations
            .iter()
            .any(|limitation| mentions_external_review_target(&limitation.to_lowercase()))
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

fn looks_like_verification_task_without_defect_evidence(
    item: &AgentFeedbackItem,
    body: &str,
    action: &str,
) -> bool {
    let combined = repository_claim_context(item, body, action);
    let asks_for_verification = mentions_external_review_target(&combined);

    asks_for_verification && !evidence_names_actual_defect(&item.evidence)
}

fn repository_claim_context(item: &AgentFeedbackItem, body: &str, action: &str) -> String {
    std::iter::once(body)
        .chain(std::iter::once(action))
        .chain(item.evidence.iter().map(String::as_str))
        .chain(item.limitations.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase()
}

fn mentions_external_review_target(value: &str) -> bool {
    value.contains("verify the import")
        || value.contains("verify import")
        || value.contains("import path")
        || value.contains("properly imported")
        || value.contains("ensure all necessary")
        || value.contains("ensure all required")
        || value.contains("review ") && value.contains(" to ensure ")
        || value.contains("may miss edge cases")
        || value.contains("if ") && value.contains(" incomplete")
        || value.contains("missing entries in")
        || value.contains("could cause missing")
        || value.contains("must be correct")
        || value.contains("imported symbol")
        || value.contains("external constant")
        || value.contains("configuration remains")
}

fn claims_external_symbol_is_missing(value: &str) -> bool {
    let mentions_symbol = [
        "constant",
        "function",
        "helper",
        "symbol",
        "import",
        "export",
        "identifier",
        "type",
        "enum",
        "method",
        "definition",
        "dependency",
    ]
    .iter()
    .any(|word| value.contains(word));

    mentions_symbol
        && [
            "does not exist",
            "doesn't exist",
            "not exist",
            "is missing",
            "missing from",
            "missing import",
            "not imported",
            "not exported",
            "does not export",
            "doesn't export",
            "not defined",
            "undefined",
            "unresolved",
            "cannot resolve",
            "can't resolve",
            "cannot find",
            "can't find",
            "not found",
            "not in scope",
            "unknown identifier",
            "no helper",
            "no function",
            "no import",
            "no matching",
        ]
        .iter()
        .any(|phrase| value.contains(phrase))
}

fn evidence_names_actual_defect(evidence: &[String]) -> bool {
    evidence.iter().any(|value| {
        let lower = value.to_lowercase();
        [
            "is missing",
            "missing from",
            "does not include",
            "not included",
            "unresolved import",
            "does not export",
            "compile error",
            "runtime error",
            "required mime type",
            "expected mime type",
            "expected video type",
        ]
        .iter()
        .any(|phrase| lower.contains(phrase))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            ChangeHunk, ChangeLine, ChangeLineKind, ChangedFile, ChangedFileStatus,
            LocalModelProviderKind, ModelProviderSettings, ReviewProfileItem,
        },
        providers::{feedback_mapping::feedback_from_agent_item, types::AgentFeedbackItem},
    };

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
            agent_item_quality_issue(&item, &changed_file(), false),
            Some("generic_review_text".to_string())
        );
    }

    #[test]
    fn rejects_verification_task_framed_as_feedback() {
        let item = AgentFeedbackItem {
            title: "Verify video MIME type source".to_string(),
            severity: "suggestion".to_string(),
            file: None,
            line: Some(10),
            start_line: None,
            end_line: None,
            body: "The code now dynamically generates the list of allowed MIME types based on the contents of EXTENDED_VIDEOS. This improves maintainability but may miss edge cases if EXTENDED_VIDEOS is incomplete or not properly imported.".to_string(),
            message: String::new(),
            suggested_action: "Review EXTENDED_VIDEOS to ensure all necessary video extensions are included and verify the import path.".to_string(),
            confidence: Some("low".to_string()),
            evidence: vec![
                "src/example.rs:10 derives allowed MIME types from EXTENDED_VIDEOS.".to_string(),
            ],
            limitations: vec![
                "Missing entries in EXTENDED_VIDEOS could cause missing MIME types".to_string(),
                "Import path for EXTENDED_VIDEOS must be correct".to_string(),
            ],
            quoted_code: None,
        };

        assert!(feedback_requires_repository_exploration(&item));
        assert_eq!(
            agent_item_quality_issue(&item, &changed_file(), false),
            Some("verification_task_without_defect_evidence".to_string())
        );
    }

    #[test]
    fn rejects_inline_feedback_that_targets_context_only_lines() {
        let item = AgentFeedbackItem {
            title: "Remove extra import brace".to_string(),
            severity: "important".to_string(),
            file: None,
            line: Some(8),
            start_line: None,
            end_line: None,
            body: "The import statement has an extra curly brace before validateFileHealth, which would cause TypeScript parsing to fail before the file can compile. The cited code does not show a changed implementation path, so this should not be published as review feedback on an unchanged context line.".to_string(),
            message: String::new(),
            suggested_action:
                "Remove the extra brace from the import statement before running the TypeScript compiler again."
                    .to_string(),
            confidence: Some("high".to_string()),
            evidence: vec![
                "src/example.rs:8 import { validateFileHealth } from './utils/validate-file-health';"
                    .to_string(),
            ],
            limitations: vec![],
            quoted_code: None,
        };

        assert_eq!(
            agent_item_quality_issue(&item, &changed_file(), false),
            Some("invalid_changed_line_range".to_string())
        );
    }

    #[test]
    fn requires_repository_exploration_before_claiming_symbol_is_missing() {
        let item = AgentFeedbackItem {
            title: "Do not call an undefined helper".to_string(),
            severity: "important".to_string(),
            file: None,
            line: Some(10),
            start_line: None,
            end_line: None,
            body: "The changed line now calls buildUploadStatus, but that helper function is not defined in the visible diff. If the function does not exist in the repository, this file will fail to compile as soon as this path is built.".to_string(),
            message: String::new(),
            suggested_action:
                "Inspect the repository for buildUploadStatus before commenting, and only report a defect if the helper is actually missing."
                    .to_string(),
            confidence: Some("medium".to_string()),
            evidence: vec![
                "src/example.rs:10 calls buildUploadStatus from the added line.".to_string(),
            ],
            limitations: vec![],
            quoted_code: None,
        };

        assert!(feedback_requires_repository_exploration(&item));
        assert_eq!(
            agent_item_quality_issue(&item, &changed_file(), false),
            Some("repository_exploration_required".to_string())
        );
    }

    #[test]
    fn accepts_missing_symbol_feedback_after_repository_exploration() {
        let item = AgentFeedbackItem {
            title: "Do not call an undefined helper".to_string(),
            severity: "important".to_string(),
            file: None,
            line: Some(10),
            start_line: None,
            end_line: None,
            body: "The changed line now calls buildUploadStatus, but repository search shows no helper function with that name and no import that would bring it into scope. That means this path can fail compilation when the changed file is built, instead of producing the intended upload status.".to_string(),
            message: String::new(),
            suggested_action:
                "Add or import buildUploadStatus, or replace the call with the existing helper that produces the upload status."
                    .to_string(),
            confidence: Some("high".to_string()),
            evidence: vec![
                "src/example.rs:10 calls buildUploadStatus from the added line.".to_string(),
                "search_repository buildUploadStatus returned no matching helper definition.".to_string(),
            ],
            limitations: vec![],
            quoted_code: None,
        };

        assert!(feedback_requires_repository_exploration(&item));
        assert_eq!(agent_item_quality_issue(&item, &changed_file(), true), None);
    }

    #[test]
    fn rejects_feedback_for_a_different_file_after_repository_exploration() {
        let item = AgentFeedbackItem {
            title: "Fix unrelated helper".to_string(),
            severity: "important".to_string(),
            file: Some("src/unrelated.rs".to_string()),
            line: Some(10),
            start_line: None,
            end_line: None,
            body: "Repository exploration found a helper in another file that appears to return success before validation. That file is not the file currently being reviewed, so this would publish feedback against code outside the selected changed hunk and confuse the author reviewing this branch.".to_string(),
            message: String::new(),
            suggested_action:
                "Anchor this finding to the changed file, or discard it when the changed hunk does not introduce the defect."
                    .to_string(),
            confidence: Some("medium".to_string()),
            evidence: vec![
                "src/unrelated.rs:10 returns before validating input.".to_string(),
            ],
            limitations: vec![],
            quoted_code: None,
        };

        assert_eq!(
            agent_item_quality_issue(&item, &changed_file(), true),
            Some("feedback_for_different_file".to_string())
        );
    }

    #[test]
    fn rejects_summary_feedback_without_changed_line_anchor() {
        let item = AgentFeedbackItem {
            title: "Avoid reviewing unrelated repository context".to_string(),
            severity: "important".to_string(),
            file: None,
            line: None,
            start_line: None,
            end_line: None,
            body: "The repository search result points at a possible validation issue, but the model did not anchor the finding to an added line from the current diff. Publishing it as summary feedback would allow comments about code that was not changed in the selected branch.".to_string(),
            message: String::new(),
            suggested_action:
                "Return the changed line that introduces the issue, or return no feedback when the issue is only in repository context."
                    .to_string(),
            confidence: Some("medium".to_string()),
            evidence: vec![
                "src/example.rs:8 context line was inspected by a repository tool.".to_string(),
            ],
            limitations: vec![],
            quoted_code: None,
        };

        assert_eq!(
            agent_item_quality_issue(&item, &changed_file(), true),
            Some("missing_changed_line_range".to_string())
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

        assert_eq!(agent_item_quality_issue(&item, &file, true), None);

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
