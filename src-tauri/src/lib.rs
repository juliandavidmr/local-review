mod domain;
mod gh;
mod git;
mod providers;
mod store;

use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

use domain::{
    ChangeSetSnapshot, ChangeSource, ExecutionStatus, GhCliStatus, ModelDescriptor,
    ModelProviderSettings, ProviderConnectionStatus, ProviderSettings, PublicationSummary,
    RepositoryDescriptor, ReviewFeedback, ReviewProfileItem, ReviewWorkspaceSession,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

static CANCELLED_REVIEWS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[tauri::command]
fn open_repository(repository_path: String) -> Result<RepositoryDescriptor, String> {
    git::open_repository(&repository_path)
}

#[tauri::command]
fn build_change_set(source: ChangeSource) -> Result<ChangeSetSnapshot, String> {
    git::build_change_set(source)
}

#[tauri::command]
fn load_profiles() -> Result<Vec<ReviewProfileItem>, String> {
    store::load_profiles()
}

#[tauri::command]
fn save_profile(profile: ReviewProfileItem) -> Result<Vec<ReviewProfileItem>, String> {
    store::save_profile(profile)
}

#[tauri::command]
fn delete_profile(profile_id: String) -> Result<Vec<ReviewProfileItem>, String> {
    store::delete_profile(&profile_id)
}

#[tauri::command]
fn load_provider_settings() -> Result<ProviderSettings, String> {
    store::load_provider_settings()
}

#[tauri::command]
fn save_provider_settings(settings: ProviderSettings) -> Result<ProviderSettings, String> {
    store::save_provider_settings(settings)
}

#[tauri::command]
fn load_review_sessions() -> Result<Vec<ReviewWorkspaceSession>, String> {
    store::load_review_sessions()
}

#[tauri::command]
fn load_latest_review_session() -> Result<Option<ReviewWorkspaceSession>, String> {
    store::load_latest_review_session()
}

#[tauri::command]
fn save_review_session(session: ReviewWorkspaceSession) -> Result<ReviewWorkspaceSession, String> {
    store::save_review_session(session)
}

#[tauri::command]
fn update_review_feedback(
    session_id: String,
    feedback_id: String,
    feedback: ReviewFeedback,
) -> Result<ReviewWorkspaceSession, String> {
    store::update_review_feedback(&session_id, &feedback_id, feedback)
}

#[tauri::command]
fn check_gh_cli_status() -> GhCliStatus {
    gh::check_status()
}

#[tauri::command]
async fn list_provider_models(
    provider: ModelProviderSettings,
) -> Result<Vec<ModelDescriptor>, String> {
    providers::list_models(provider).await
}

#[tauri::command]
async fn check_provider_connection(
    provider: ModelProviderSettings,
) -> Result<ProviderConnectionStatus, String> {
    providers::check_connection(provider).await
}

#[tauri::command]
async fn run_review_session(
    app: AppHandle,
    review_id: String,
    repository: RepositoryDescriptor,
    change_set: ChangeSetSnapshot,
    profiles: Vec<ReviewProfileItem>,
    provider_settings: ProviderSettings,
) -> Result<ReviewWorkspaceSession, String> {
    let provider = provider_settings
        .model_providers
        .iter()
        .find(|candidate| candidate.enabled && candidate.selected_model_id.is_some())
        .ok_or_else(|| "Select one model provider and model before running review.".to_string())?
        .clone();
    let active_profiles = profiles
        .iter()
        .filter(|profile| profile.selected)
        .cloned()
        .collect::<Vec<_>>();

    if active_profiles.is_empty() {
        return Err("Select at least one review profile.".to_string());
    }

    let mut feedback = Vec::<ReviewFeedback>::new();
    let mut completed_passes = 0;
    let mut failed_passes = 0;
    let mut pass_index = 0;
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
            match providers::run_review_pass(&provider, profile, &change_set, file, pass_index)
                .await
            {
                Ok(feedback_from_pass) => {
                    eprintln!(
                        "[local-review-pass] pass_ok review_id={} pass={} file={} profile={} feedback_count={}",
                        review_id,
                        pass_index + 1,
                        file.path,
                        profile.name,
                        feedback_from_pass.len()
                    );
                    completed_passes += 1;
                    pass_feedback = feedback_from_pass.clone();
                    feedback.extend(feedback_from_pass);
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

    let inline_comments = feedback
        .iter()
        .filter(|item| matches!(item.feedback_type, domain::FeedbackType::Inline))
        .count() as u32;
    let summary_comments = feedback.len() as u32 - inline_comments;
    let total_comments = feedback.len() as u32;
    let limited_context_count = feedback.iter().filter(|item| item.limited_context).count() as u32;
    let modified_lines = change_set
        .files
        .iter()
        .map(|file| file.additions + file.deletions)
        .sum::<u32>();
    let status = if cancelled {
        "cancelled"
    } else if failed_passes > 0 || (change_set.files.len() > 0 && total_passes == 0) {
        "incomplete"
    } else {
        "completed"
    };

    let session = ReviewWorkspaceSession {
        repository,
        change_source: change_source_label(&change_set.source).to_string(),
        change_set: change_set.clone(),
        profiles: active_profiles,
        provider_settings,
        execution: ExecutionStatus {
            status: status.to_string(),
            completed_passes,
            total_passes,
            changed_files: change_set.files.len() as u32,
            modified_lines,
            exploration_requests: 0,
            guardrail_hits: failed_passes,
        },
        feedback,
        publication: PublicationSummary {
            target: "gh pull request publication not selected".to_string(),
            total_comments,
            inline_comments,
            summary_comments,
            limited_context_count,
            incomplete_session: status == "incomplete",
        },
    };

    store::save_review_session(session)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewProgressEvent {
    review_id: String,
    execution: ExecutionStatus,
    feedback: Vec<ReviewFeedback>,
}

fn emit_review_progress(
    app: &AppHandle,
    review_id: &str,
    status: &str,
    completed_passes: u32,
    total_passes: u32,
    failed_passes: u32,
    feedback: Vec<ReviewFeedback>,
) {
    let _ = app.emit(
        "review-progress",
        ReviewProgressEvent {
            review_id: review_id.to_string(),
            execution: ExecutionStatus {
                status: status.to_string(),
                completed_passes,
                total_passes,
                changed_files: 0,
                modified_lines: 0,
                exploration_requests: 0,
                guardrail_hits: failed_passes,
            },
            feedback,
        },
    );
}

#[tauri::command]
fn cancel_review_session(review_id: String) -> Result<(), String> {
    cancelled_reviews()
        .lock()
        .map_err(|_| "Could not lock cancellation registry.".to_string())?
        .insert(review_id);
    Ok(())
}

fn review_cancelled(review_id: &str) -> bool {
    cancelled_reviews()
        .lock()
        .map(|reviews| reviews.contains(review_id))
        .unwrap_or(false)
}

fn clear_review_cancellation(review_id: &str) {
    if let Ok(mut reviews) = cancelled_reviews().lock() {
        reviews.remove(review_id);
    }
}

fn cancelled_reviews() -> &'static Mutex<HashSet<String>> {
    CANCELLED_REVIEWS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn change_source_label(source: &ChangeSource) -> &'static str {
    match source {
        ChangeSource::WorkingTree { .. } => "Working tree",
        ChangeSource::CurrentBranch { .. } => "Current branch",
        ChangeSource::StagedChanges { .. } => "Staged changes",
        ChangeSource::UnstagedChanges { .. } => "Unstaged changes",
        ChangeSource::Commit { .. } => "Commit",
        ChangeSource::CompareRefs { .. } => "Compare refs",
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_repository,
            build_change_set,
            load_profiles,
            save_profile,
            delete_profile,
            load_provider_settings,
            save_provider_settings,
            load_review_sessions,
            load_latest_review_session,
            save_review_session,
            update_review_feedback,
            check_gh_cli_status,
            list_provider_models,
            check_provider_connection,
            cancel_review_session,
            run_review_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
