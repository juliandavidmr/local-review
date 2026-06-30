use tauri::AppHandle;

use crate::{
    domain::{
        ChangeSetSnapshot, ExecutionStatus, ProviderSettings, RepositoryDescriptor, ReviewFeedback,
        ReviewProfileItem, ReviewWorkspaceSession,
    },
    providers, store,
};

use super::{
    cancellation::{clear_review_cancellation, review_cancelled},
    progress::emit_review_progress,
    summary::{
        change_source_label, publication_summary, review_status, selected_profiles,
        selected_provider,
    },
};

pub async fn run_review_session(
    app: AppHandle,
    review_id: String,
    repository: RepositoryDescriptor,
    change_set: ChangeSetSnapshot,
    profiles: Vec<ReviewProfileItem>,
    provider_settings: ProviderSettings,
) -> Result<ReviewWorkspaceSession, String> {
    let provider = selected_provider(&provider_settings)?;
    let active_profiles = selected_profiles(&profiles)?;

    let mut feedback = Vec::<ReviewFeedback>::new();
    let mut completed_passes = 0;
    let mut failed_passes = 0;
    let mut pass_index = 0;
    let mut exploration_requests = 0;
    let repository_tools_enabled = provider_settings
        .mcp_sources
        .iter()
        .any(|source| source.id == "filesystem" && source.enabled);
    let total_passes = change_set
        .files
        .iter()
        .filter(|file| !file.is_generated)
        .count() as u32
        * active_profiles.len() as u32;
    let mut cancelled = false;

    eprintln!(
        "[local-review-pass] review_start review_id={} provider={} model={} files={} profiles={} total_passes={}",
        review_id,
        provider.id,
        provider
            .selected_model_id
            .as_deref()
            .unwrap_or("unselected"),
        change_set.files.len(),
        active_profiles.len(),
        total_passes
    );

    for file in change_set.files.iter().filter(|file| !file.is_generated) {
        for profile in &active_profiles {
            if review_cancelled(&review_id) {
                cancelled = true;
                emit_review_progress(
                    &app,
                    &review_id,
                    "cancelled",
                    completed_passes,
                    total_passes,
                    failed_passes,
                    exploration_requests,
                    Vec::new(),
                );
                break;
            }

            eprintln!(
                "[local-review-pass] pass_start review_id={} pass={} file={} profile={} additions={} deletions={}",
                review_id,
                pass_index + 1,
                file.path,
                profile.name,
                file.additions,
                file.deletions
            );

            let mut pass_feedback = Vec::new();
            match providers::run_review_pass(
                &provider,
                profile,
                &change_set,
                file,
                pass_index,
                repository_tools_enabled,
            )
            .await
            {
                Ok(pass_result) => {
                    eprintln!(
                        "[local-review-pass] pass_ok review_id={} pass={} file={} profile={} feedback_count={}",
                        review_id,
                        pass_index + 1,
                        file.path,
                        profile.name,
                        pass_result.feedback.len()
                    );
                    completed_passes += 1;
                    exploration_requests += pass_result.exploration_requests;
                    pass_feedback = pass_result.feedback.clone();
                    feedback.extend(pass_result.feedback);
                }
                Err(error) => {
                    eprintln!(
                        "[local-review-pass] pass_error review_id={} pass={} file={} profile={} error={}",
                        review_id,
                        pass_index + 1,
                        file.path,
                        profile.name,
                        error
                    );
                    failed_passes += 1;
                }
            }

            pass_index += 1;
            emit_review_progress(
                &app,
                &review_id,
                "running",
                completed_passes,
                total_passes,
                failed_passes,
                exploration_requests,
                pass_feedback,
            );
        }

        if cancelled {
            break;
        }
    }

    eprintln!(
        "[local-review-pass] review_finish review_id={} completed_passes={} failed_passes={} cancelled={}",
        review_id, completed_passes, failed_passes, cancelled
    );
    clear_review_cancellation(&review_id);

    let publication = publication_summary(
        &feedback,
        failed_passes,
        cancelled,
        &change_set,
        total_passes,
    );
    let modified_lines = change_set
        .files
        .iter()
        .map(|file| file.additions + file.deletions)
        .sum::<u32>();
    let status = review_status(cancelled, failed_passes, &change_set, total_passes);

    let session = ReviewWorkspaceSession {
        repository,
        change_source: change_source_label(&change_set.source).to_string(),
        change_set: change_set.clone(),
        profiles: active_profiles,
        provider_settings,
        execution: ExecutionStatus {
            status,
            completed_passes,
            total_passes,
            changed_files: change_set.files.len() as u32,
            modified_lines,
            exploration_requests,
            guardrail_hits: failed_passes,
        },
        feedback,
        publication,
    };

    store::save_review_session(session)
}

pub fn cancel_review_session(review_id: String) -> Result<(), String> {
    super::cancellation::cancel_review_session(review_id)
}
