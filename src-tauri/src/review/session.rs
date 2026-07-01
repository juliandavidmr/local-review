use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::time::Duration;

use tauri::AppHandle;
use tokio::task::JoinSet;

use crate::{
    domain::{
        ChangeSetSnapshot, ChangedFile, ExecutionCapacitySettings, ExecutionStatus,
        ProviderSettings, RepositoryDescriptor, ReviewFeedback, ReviewProfileItem,
        ReviewWorkspaceSession,
    },
    providers::{self, ReviewPassResult},
    store,
};

use super::{
    cancellation::{clear_review_cancellation, review_cancelled},
    progress::emit_review_progress,
    summary::{
        change_source_label, publication_summary, review_status, selected_profiles,
        selected_provider,
    },
};

const MODEL_READY_TIMEOUT: Duration = Duration::from_secs(90);

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
    let repository_tools_enabled = provider_settings
        .mcp_sources
        .iter()
        .any(|source| source.id == "filesystem" && source.enabled);
    let mut pending_passes = ReviewPassQueue::new(&change_set, &active_profiles);
    let total_passes = pending_passes.total_passes();
    let max_parallel_passes = review_parallelism(
        &provider_settings.execution,
        repository_tools_enabled,
        total_passes,
    );

    let completed_passes = Arc::new(AtomicU32::new(0));
    let failed_passes = Arc::new(AtomicU32::new(0));
    let exploration_requests = Arc::new(AtomicU32::new(0));
    let mut completed_feedback = Vec::<(usize, Vec<ReviewFeedback>)>::new();
    let mut cancelled = false;

    eprintln!(
        "[local-review-pass] review_start review_id={} provider={} model={} files={} profiles={} total_passes={} max_parallel_passes={}",
        review_id,
        provider.id,
        provider
            .selected_model_id
            .as_deref()
            .unwrap_or("unselected"),
        change_set.files.len(),
        active_profiles.len(),
        total_passes,
        max_parallel_passes
    );

    emit_review_progress(
        &app,
        &review_id,
        "running",
        0,
        total_passes,
        0,
        0,
        None,
        None,
        Some(format!(
            "Waiting for selected model `{}` to become ready",
            provider
                .selected_model_id
                .as_deref()
                .unwrap_or("unselected")
        )),
        Vec::new(),
    );
    let readiness =
        providers::wait_for_selected_model_ready(&provider, MODEL_READY_TIMEOUT).await?;
    eprintln!(
        "[local-review-pass] model_ready review_id={} provider={} model={} attempts={}",
        review_id,
        provider.id,
        provider
            .selected_model_id
            .as_deref()
            .unwrap_or("unselected"),
        readiness.attempts
    );
    emit_review_progress(
        &app,
        &review_id,
        "running",
        0,
        total_passes,
        0,
        0,
        None,
        None,
        Some("Selected model is ready; starting review passes".to_string()),
        Vec::new(),
    );

    let change_set_for_tasks = Arc::new(change_set.clone());
    let mut running_passes = JoinSet::new();

    loop {
        while running_passes.len() < max_parallel_passes {
            if review_cancelled(&review_id) {
                cancelled = true;
                emit_review_progress(
                    &app,
                    &review_id,
                    "cancelled",
                    completed_passes.load(Ordering::SeqCst),
                    total_passes,
                    failed_passes.load(Ordering::SeqCst),
                    exploration_requests.load(Ordering::SeqCst),
                    None,
                    None,
                    Some("Stopping review after currently running passes".to_string()),
                    Vec::new(),
                );
                break;
            }

            let Some(work_item) = pending_passes.next() else {
                break;
            };

            eprintln!(
                "[local-review-pass] pass_start review_id={} pass={} file={} profile={} additions={} deletions={}",
                review_id,
                work_item.pass_index + 1,
                work_item.file.path,
                work_item.profile.name,
                work_item.file.additions,
                work_item.file.deletions
            );

            emit_review_progress(
                &app,
                &review_id,
                "running",
                completed_passes.load(Ordering::SeqCst),
                total_passes,
                failed_passes.load(Ordering::SeqCst),
                exploration_requests.load(Ordering::SeqCst),
                Some(work_item.file.path.clone()),
                Some(work_item.profile.name.clone()),
                Some("Reviewing changed hunks with the selected model".to_string()),
                Vec::new(),
            );

            let task_provider = provider.clone();
            let task_change_set = change_set_for_tasks.clone();
            let task_app = app.clone();
            let task_review_id = review_id.clone();
            let task_completed_passes = completed_passes.clone();
            let task_failed_passes = failed_passes.clone();
            let task_exploration_requests = exploration_requests.clone();

            running_passes.spawn(async move {
                let result = providers::run_review_pass(
                    &task_provider,
                    &work_item.profile,
                    task_change_set.as_ref(),
                    &work_item.file,
                    work_item.pass_index,
                    repository_tools_enabled,
                    crate::providers::AgentProgressContext {
                        app: task_app,
                        review_id: task_review_id,
                        current_file: work_item.file.path.clone(),
                        current_profile: work_item.profile.name.clone(),
                        completed_passes: task_completed_passes,
                        total_passes,
                        failed_passes: task_failed_passes,
                        exploration_requests: task_exploration_requests,
                        current_phase: "Exploring repository context".to_string(),
                    },
                )
                .await;

                ReviewPassTaskResult { work_item, result }
            });
        }

        if running_passes.is_empty() {
            break;
        }

        let Some(task_result) = running_passes.join_next().await else {
            break;
        };

        match task_result {
            Ok(ReviewPassTaskResult { work_item, result }) => match result {
                Ok(pass_result) => {
                    eprintln!(
                        "[local-review-pass] pass_ok review_id={} pass={} file={} profile={} feedback_count={}",
                        review_id,
                        work_item.pass_index + 1,
                        work_item.file.path,
                        work_item.profile.name,
                        pass_result.feedback.len()
                    );
                    if !repository_tools_enabled {
                        exploration_requests
                            .fetch_add(pass_result.exploration_requests, Ordering::SeqCst);
                    }
                    completed_passes.fetch_add(1, Ordering::SeqCst);
                    completed_feedback.push((work_item.pass_index, pass_result.feedback.clone()));
                    emit_review_progress(
                        &app,
                        &review_id,
                        "running",
                        completed_passes.load(Ordering::SeqCst),
                        total_passes,
                        failed_passes.load(Ordering::SeqCst),
                        exploration_requests.load(Ordering::SeqCst),
                        Some(work_item.file.path),
                        Some(work_item.profile.name),
                        Some("Finished pass and publishing usable feedback".to_string()),
                        pass_result.feedback,
                    );
                }
                Err(error) => {
                    eprintln!(
                        "[local-review-pass] pass_error review_id={} pass={} file={} profile={} error={}",
                        review_id,
                        work_item.pass_index + 1,
                        work_item.file.path,
                        work_item.profile.name,
                        error
                    );
                    failed_passes.fetch_add(1, Ordering::SeqCst);
                    emit_review_progress(
                        &app,
                        &review_id,
                        "running",
                        completed_passes.load(Ordering::SeqCst),
                        total_passes,
                        failed_passes.load(Ordering::SeqCst),
                        exploration_requests.load(Ordering::SeqCst),
                        Some(work_item.file.path),
                        Some(work_item.profile.name),
                        Some("Review pass failed; continuing with remaining passes".to_string()),
                        Vec::new(),
                    );
                }
            },
            Err(error) => {
                eprintln!(
                    "[local-review-pass] pass_join_error review_id={} error={}",
                    review_id, error
                );
                failed_passes.fetch_add(1, Ordering::SeqCst);
                emit_review_progress(
                    &app,
                    &review_id,
                    "running",
                    completed_passes.load(Ordering::SeqCst),
                    total_passes,
                    failed_passes.load(Ordering::SeqCst),
                    exploration_requests.load(Ordering::SeqCst),
                    None,
                    None,
                    Some("Review pass failed before returning a result".to_string()),
                    Vec::new(),
                );
            }
        }

        if review_cancelled(&review_id) && !cancelled {
            cancelled = true;
            emit_review_progress(
                &app,
                &review_id,
                "cancelled",
                completed_passes.load(Ordering::SeqCst),
                total_passes,
                failed_passes.load(Ordering::SeqCst),
                exploration_requests.load(Ordering::SeqCst),
                None,
                None,
                Some("Stopping review after currently running passes".to_string()),
                Vec::new(),
            );
        }
    }

    completed_feedback.sort_by_key(|(pass_index, _)| *pass_index);
    let feedback = completed_feedback
        .into_iter()
        .flat_map(|(_, pass_feedback)| pass_feedback)
        .collect::<Vec<_>>();
    let completed_passes = completed_passes.load(Ordering::SeqCst);
    let failed_passes = failed_passes.load(Ordering::SeqCst);
    let exploration_requests = exploration_requests.load(Ordering::SeqCst);

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
            current_file: None,
            current_profile: None,
            current_phase: None,
        },
        feedback,
        publication,
    };

    store::save_review_session(session)
}

#[derive(Debug)]
struct ReviewPassWorkItem {
    file: ChangedFile,
    profile: ReviewProfileItem,
    pass_index: usize,
}

struct ReviewPassTaskResult {
    work_item: ReviewPassWorkItem,
    result: Result<ReviewPassResult, String>,
}

struct ReviewPassQueue<'a> {
    change_set: &'a ChangeSetSnapshot,
    active_profiles: &'a [ReviewProfileItem],
    file_index: usize,
    profile_index: usize,
    pass_index: usize,
    total_passes: u32,
}

impl<'a> ReviewPassQueue<'a> {
    fn new(change_set: &'a ChangeSetSnapshot, active_profiles: &'a [ReviewProfileItem]) -> Self {
        let total_passes = change_set
            .files
            .iter()
            .filter(|file| !file.is_generated)
            .count() as u32
            * active_profiles.len() as u32;

        Self {
            change_set,
            active_profiles,
            file_index: 0,
            profile_index: 0,
            pass_index: 0,
            total_passes,
        }
    }

    fn total_passes(&self) -> u32 {
        self.total_passes
    }
}

impl Iterator for ReviewPassQueue<'_> {
    type Item = ReviewPassWorkItem;

    fn next(&mut self) -> Option<Self::Item> {
        while self.file_index < self.change_set.files.len() {
            let file = &self.change_set.files[self.file_index];
            if file.is_generated {
                self.file_index += 1;
                self.profile_index = 0;
                continue;
            }

            if self.profile_index >= self.active_profiles.len() {
                self.file_index += 1;
                self.profile_index = 0;
                continue;
            }

            let profile = &self.active_profiles[self.profile_index];
            let pass_index = self.pass_index;
            self.profile_index += 1;
            self.pass_index += 1;

            return Some(ReviewPassWorkItem {
                file: file.clone(),
                profile: profile.clone(),
                pass_index,
            });
        }

        None
    }
}

fn review_parallelism(
    execution: &ExecutionCapacitySettings,
    repository_tools_enabled: bool,
    total_passes: u32,
) -> usize {
    const HARD_LOCAL_CAP: usize = 4;
    const TOOL_USE_CAP: usize = 2;
    const ADAPTIVE_LOCAL_CAP: usize = 3;

    if total_passes == 0 {
        return 1;
    }

    let requested = execution.max_parallel_review_passes.max(1) as usize;
    let adaptive_cap = if !execution.adaptive_parallelism_enabled {
        HARD_LOCAL_CAP
    } else if repository_tools_enabled {
        TOOL_USE_CAP
    } else {
        ADAPTIVE_LOCAL_CAP
    };

    requested
        .min(adaptive_cap)
        .min(HARD_LOCAL_CAP)
        .min(total_passes as usize)
        .max(1)
}

pub fn cancel_review_session(review_id: String) -> Result<(), String> {
    super::cancellation::cancel_review_session(review_id)
}

#[cfg(test)]
mod tests {
    use crate::domain::{
        ChangeSetSnapshot, ChangeSource, ChangedFile, ChangedFileStatus, ExecutionCapacitySettings,
        ProfileScopeKind, ReviewProfileItem,
    };

    use super::{review_parallelism, ReviewPassQueue};

    #[test]
    fn review_parallelism_is_never_less_than_one() {
        let execution = ExecutionCapacitySettings {
            max_parallel_review_passes: 0,
            adaptive_parallelism_enabled: true,
        };

        assert_eq!(review_parallelism(&execution, false, 0), 1);
    }

    #[test]
    fn adaptive_parallelism_caps_tool_using_passes_at_two() {
        let execution = ExecutionCapacitySettings {
            max_parallel_review_passes: 4,
            adaptive_parallelism_enabled: true,
        };

        assert_eq!(review_parallelism(&execution, true, 8), 2);
    }

    #[test]
    fn adaptive_parallelism_caps_local_passes_at_three_without_tools() {
        let execution = ExecutionCapacitySettings {
            max_parallel_review_passes: 4,
            adaptive_parallelism_enabled: true,
        };

        assert_eq!(review_parallelism(&execution, false, 8), 3);
    }

    #[test]
    fn manual_parallelism_still_has_a_hard_local_cap() {
        let execution = ExecutionCapacitySettings {
            max_parallel_review_passes: 99,
            adaptive_parallelism_enabled: false,
        };

        assert_eq!(review_parallelism(&execution, false, 20), 4);
    }

    #[test]
    fn review_parallelism_does_not_exceed_total_passes() {
        let execution = ExecutionCapacitySettings {
            max_parallel_review_passes: 4,
            adaptive_parallelism_enabled: false,
        };

        assert_eq!(review_parallelism(&execution, false, 2), 2);
    }

    #[test]
    fn review_pass_queue_counts_only_non_generated_files() {
        let change_set = change_set_with_files(vec![
            changed_file("src/a.rs", false),
            changed_file("dist/generated.js", true),
            changed_file("src/b.rs", false),
        ]);
        let profiles = vec![profile("correctness"), profile("architecture")];

        let queue = ReviewPassQueue::new(&change_set, &profiles);

        assert_eq!(queue.total_passes(), 4);
    }

    #[test]
    fn review_pass_queue_preserves_file_then_profile_order() {
        let change_set = change_set_with_files(vec![
            changed_file("src/a.rs", false),
            changed_file("dist/generated.js", true),
            changed_file("src/b.rs", false),
        ]);
        let profiles = vec![profile("correctness"), profile("architecture")];
        let mut queue = ReviewPassQueue::new(&change_set, &profiles);

        let passes = queue
            .by_ref()
            .map(|work_item| {
                (
                    work_item.pass_index,
                    work_item.file.path,
                    work_item.profile.id,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            passes,
            vec![
                (0, "src/a.rs".to_string(), "correctness".to_string()),
                (1, "src/a.rs".to_string(), "architecture".to_string()),
                (2, "src/b.rs".to_string(), "correctness".to_string()),
                (3, "src/b.rs".to_string(), "architecture".to_string()),
            ]
        );
        assert!(queue.next().is_none());
    }

    #[test]
    fn review_pass_queue_handles_no_revisable_files() {
        let change_set = change_set_with_files(vec![changed_file("dist/generated.js", true)]);
        let profiles = vec![profile("correctness")];
        let mut queue = ReviewPassQueue::new(&change_set, &profiles);

        assert_eq!(queue.total_passes(), 0);
        assert!(queue.next().is_none());
    }

    fn change_set_with_files(files: Vec<ChangedFile>) -> ChangeSetSnapshot {
        ChangeSetSnapshot {
            id: "change-set".to_string(),
            repository_path: "/tmp/repo".to_string(),
            source: ChangeSource::WorkingTree {
                repository_path: "/tmp/repo".to_string(),
            },
            base_ref: None,
            head_ref: None,
            files,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            fingerprint: "fingerprint".to_string(),
        }
    }

    fn changed_file(path: &str, is_generated: bool) -> ChangedFile {
        ChangedFile {
            path: path.to_string(),
            previous_path: None,
            status: ChangedFileStatus::Modified,
            additions: 1,
            deletions: 0,
            hunks: Vec::new(),
            is_generated,
        }
    }

    fn profile(id: &str) -> ReviewProfileItem {
        ReviewProfileItem {
            id: id.to_string(),
            name: id.to_string(),
            scope: "Global default".to_string(),
            scope_kind: ProfileScopeKind::Global,
            selected: true,
            enabled_by_default: true,
            criteria: Vec::new(),
            file_globs: vec!["*".to_string()],
            prompt: "Review this change.".to_string(),
        }
    }
}
