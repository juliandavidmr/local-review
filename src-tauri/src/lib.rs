mod domain;
mod gh;
mod git;
mod providers;
mod review;
mod store;

use domain::{
    ChangeSetSnapshot, ChangeSource, GhCliStatus, ModelDescriptor, ModelProviderSettings,
    ProviderConnectionStatus, ProviderSettings, RepositoryDescriptor, ReviewFeedback,
    ReviewProfileItem, ReviewWorkspaceSession,
};
use tauri::AppHandle;

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
fn delete_review_feedback(
    session_id: String,
    feedback_id: String,
) -> Result<ReviewWorkspaceSession, String> {
    store::delete_review_feedback(&session_id, &feedback_id)
}

#[tauri::command]
fn check_gh_cli_status() -> GhCliStatus {
    gh::check_status()
}

#[tauri::command]
fn publish_review_feedback(
    repository_path: String,
    feedback: ReviewFeedback,
) -> Result<(), String> {
    gh::publish_review_feedback(&repository_path, &feedback)
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
    review::run_review_session(
        app,
        review_id,
        repository,
        change_set,
        profiles,
        provider_settings,
    )
    .await
}

#[tauri::command]
fn cancel_review_session(review_id: String) -> Result<(), String> {
    review::cancel_review_session(review_id)
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
            delete_review_feedback,
            check_gh_cli_status,
            publish_review_feedback,
            list_provider_models,
            check_provider_connection,
            cancel_review_session,
            run_review_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
