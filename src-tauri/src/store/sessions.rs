use std::fs;

use crate::domain::{ReviewFeedback, ReviewWorkspaceSession};

use super::paths::{ensure_parent, review_history_path};

pub fn load_review_sessions() -> Result<Vec<ReviewWorkspaceSession>, String> {
    let path = review_history_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let sessions = serde_json::from_str::<Vec<ReviewWorkspaceSession>>(&raw)
        .map_err(|error| error.to_string())?;

    Ok(sessions.into_iter().map(finalize_stale_session).collect())
}

pub fn save_review_session(
    session: ReviewWorkspaceSession,
) -> Result<ReviewWorkspaceSession, String> {
    let mut sessions = load_review_sessions()?;
    let session_id = session.change_set.id.clone();
    if let Some(index) = sessions
        .iter()
        .position(|existing| existing.change_set.id == session_id)
    {
        sessions[index] = session.clone();
    } else {
        sessions.insert(0, session.clone());
    }
    write_review_sessions(sessions)?;
    Ok(session)
}

pub fn load_latest_review_session() -> Result<Option<ReviewWorkspaceSession>, String> {
    Ok(load_review_sessions()?.into_iter().next())
}

fn finalize_stale_session(mut session: ReviewWorkspaceSession) -> ReviewWorkspaceSession {
    if session.execution.status == "running" {
        session.execution.status = "incomplete".to_string();
        session.publication.incomplete_session = true;
    }

    session
}

pub fn update_review_feedback(
    session_id: &str,
    feedback_id: &str,
    feedback: ReviewFeedback,
) -> Result<ReviewWorkspaceSession, String> {
    let mut sessions = load_review_sessions()?;
    let session = sessions
        .iter_mut()
        .find(|session| session.change_set.id == session_id)
        .ok_or_else(|| "Review session was not found in local history.".to_string())?;

    let item = session
        .feedback
        .iter_mut()
        .find(|item| item.id == feedback_id)
        .ok_or_else(|| "Review feedback was not found in local history.".to_string())?;
    *item = feedback;
    recalculate_publication_summary(session);

    let updated = session.clone();
    write_review_sessions(sessions)?;
    Ok(updated)
}

pub fn delete_review_feedback(
    session_id: &str,
    feedback_id: &str,
) -> Result<ReviewWorkspaceSession, String> {
    let mut sessions = load_review_sessions()?;
    let session = sessions
        .iter_mut()
        .find(|session| session.change_set.id == session_id)
        .ok_or_else(|| "Review session was not found in local history.".to_string())?;

    let initial_count = session.feedback.len();
    session.feedback.retain(|item| item.id != feedback_id);
    if session.feedback.len() == initial_count {
        return Err("Review feedback was not found in local history.".to_string());
    }

    recalculate_publication_summary(session);

    let updated = session.clone();
    write_review_sessions(sessions)?;
    Ok(updated)
}

fn write_review_sessions(sessions: Vec<ReviewWorkspaceSession>) -> Result<(), String> {
    let path = review_history_path()?;
    ensure_parent(&path)?;
    let raw = serde_json::to_string_pretty(&sessions).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

fn recalculate_publication_summary(session: &mut ReviewWorkspaceSession) {
    session.publication.total_comments = session.feedback.len() as u32;
    session.publication.inline_comments = session
        .feedback
        .iter()
        .filter(|item| matches!(item.feedback_type, crate::domain::FeedbackType::Inline))
        .count() as u32;
    session.publication.summary_comments =
        session.publication.total_comments - session.publication.inline_comments;
    session.publication.limited_context_count = session
        .feedback
        .iter()
        .filter(|item| item.limited_context)
        .count() as u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        default_profiles, default_provider_settings, ChangeSetSnapshot, ChangeSource,
        ExecutionStatus, PublicationSummary, RepositoryDescriptor,
    };

    #[test]
    fn stale_running_sessions_load_as_incomplete() {
        let stale = ReviewWorkspaceSession {
            repository: RepositoryDescriptor {
                path: "/tmp/repo".to_string(),
                name: "repo".to_string(),
                current_branch: Some("main".to_string()),
                head_sha: Some("abc123".to_string()),
            },
            change_source: "Current branch".to_string(),
            change_set: ChangeSetSnapshot {
                id: "session-1".to_string(),
                repository_path: "/tmp/repo".to_string(),
                source: ChangeSource::CurrentBranch {
                    repository_path: "/tmp/repo".to_string(),
                },
                base_ref: None,
                head_ref: None,
                files: Vec::new(),
                created_at: "2026-06-29T00:00:00Z".to_string(),
                fingerprint: "fingerprint".to_string(),
            },
            profiles: default_profiles(),
            provider_settings: default_provider_settings(),
            execution: ExecutionStatus {
                status: "running".to_string(),
                completed_passes: 0,
                total_passes: 1,
                changed_files: 1,
                modified_lines: 1,
                exploration_requests: 0,
                guardrail_hits: 0,
                current_file: None,
                current_profile: None,
                current_phase: None,
            },
            feedback: Vec::new(),
            publication: PublicationSummary {
                target: "gh pull request publication not selected".to_string(),
                total_comments: 0,
                inline_comments: 0,
                summary_comments: 0,
                limited_context_count: 0,
                incomplete_session: false,
            },
        };

        let loaded = finalize_stale_session(stale);

        assert_eq!(loaded.execution.status, "incomplete");
        assert!(loaded.publication.incomplete_session);
    }
}
