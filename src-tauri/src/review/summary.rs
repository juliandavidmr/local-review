use crate::domain::{
    ChangeSetSnapshot, ChangeSource, ModelProviderSettings, ProviderSettings, PublicationSummary,
    ReviewFeedback, ReviewProfileItem,
};

pub(super) fn selected_provider(
    settings: &ProviderSettings,
) -> Result<ModelProviderSettings, String> {
    settings
        .model_providers
        .iter()
        .find(|candidate| candidate.enabled && candidate.selected_model_id.is_some())
        .ok_or_else(|| "Select one model provider and model before running review.".to_string())
        .cloned()
}

pub(super) fn selected_profiles(
    profiles: &[ReviewProfileItem],
) -> Result<Vec<ReviewProfileItem>, String> {
    let active_profiles = profiles
        .iter()
        .filter(|profile| profile.selected)
        .cloned()
        .collect::<Vec<_>>();

    if active_profiles.is_empty() {
        Err("Select at least one review profile.".to_string())
    } else {
        Ok(active_profiles)
    }
}

pub(super) fn publication_summary(
    feedback: &[ReviewFeedback],
    failed_passes: u32,
    cancelled: bool,
    change_set: &ChangeSetSnapshot,
    total_passes: u32,
) -> PublicationSummary {
    let inline_comments = feedback
        .iter()
        .filter(|item| matches!(item.feedback_type, crate::domain::FeedbackType::Inline))
        .count() as u32;
    let summary_comments = feedback.len() as u32 - inline_comments;
    let total_comments = feedback.len() as u32;
    let limited_context_count = feedback.iter().filter(|item| item.limited_context).count() as u32;
    let status = review_status(cancelled, failed_passes, change_set, total_passes);

    PublicationSummary {
        target: "gh pull request publication not selected".to_string(),
        total_comments,
        inline_comments,
        summary_comments,
        limited_context_count,
        incomplete_session: status == "incomplete",
    }
}

pub(super) fn review_status(
    cancelled: bool,
    failed_passes: u32,
    change_set: &ChangeSetSnapshot,
    total_passes: u32,
) -> String {
    if cancelled {
        "cancelled".to_string()
    } else if failed_passes > 0 || (change_set.files.len() > 0 && total_passes == 0) {
        "incomplete".to_string()
    } else {
        "completed".to_string()
    }
}

pub(super) fn change_source_label(source: &ChangeSource) -> &'static str {
    match source {
        ChangeSource::WorkingTree { .. } => "Working tree",
        ChangeSource::CurrentBranch { .. } => "Current branch",
        ChangeSource::StagedChanges { .. } => "Staged changes",
        ChangeSource::UnstagedChanges { .. } => "Unstaged changes",
        ChangeSource::Commit { .. } => "Commit",
        ChangeSource::CompareRefs { .. } => "Compare refs",
    }
}
