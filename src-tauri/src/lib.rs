mod domain;
mod git;
mod providers;
mod store;

use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

use domain::{
    ChangeSetSnapshot, ChangeSource, ExecutionStatus, ModelDescriptor, ModelProviderSettings,
    ProviderConnectionStatus, ProviderSettings, PublicationSummary, RepositoryDescriptor,
    ReviewFeedback, ReviewProfileItem, ReviewWorkspaceSession,
};

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

    for file in change_set.files.iter().filter(|file| !file.is_generated) {
        for profile in &active_profiles {
            if review_cancelled(&review_id) {
                cancelled = true;
                break;
            }

            match providers::run_review_pass(&provider, profile, &change_set, file, pass_index)
                .await
            {
                Ok(mut pass_feedback) => {
                    completed_passes += 1;
                    feedback.append(&mut pass_feedback);
                }
                Err(_) => {
                    failed_passes += 1;
                }
            }
            pass_index += 1;
        }

        if cancelled {
            break;
        }
    }

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

    Ok(ReviewWorkspaceSession {
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
    })
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
            list_provider_models,
            check_provider_connection,
            cancel_review_session,
            run_review_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
