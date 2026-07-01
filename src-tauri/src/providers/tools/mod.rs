mod read_file;
mod safety;
mod search;

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::domain::ExecutionStatus;
use tauri::Emitter;

pub(super) use read_file::ReadRepositoryFileTool;
pub(super) use safety::{is_sensitive_path, safe_repository_file};
pub(super) use search::SearchRepositoryTool;

use super::types::AgentProgressContext;

#[derive(Clone)]
pub(super) struct ToolUsageHook {
    pub exploration_requests: Arc<AtomicU32>,
    pub progress: Option<AgentProgressContext>,
}

impl<M: rig::completion::CompletionModel> rig::agent::PromptHook<M> for ToolUsageHook {
    async fn on_tool_call(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
    ) -> rig::agent::ToolCallHookAction {
        let pass_exploration_requests =
            self.exploration_requests.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(progress) = &self.progress {
            let exploration_requests =
                progress.exploration_requests.fetch_add(1, Ordering::SeqCst) + 1;
            eprintln!(
                "[local-review-tool] call review_id={} file={} profile={} pass_tool_calls={} total_tool_calls={} tool={} args={}",
                progress.review_id,
                progress.current_file,
                progress.current_profile,
                pass_exploration_requests,
                exploration_requests,
                tool_name,
                compact_tool_args(args)
            );
            let _ = progress.app.emit(
                "review-progress",
                serde_json::json!({
                    "reviewId": progress.review_id,
                    "execution": ExecutionStatus {
                        status: "running".to_string(),
                        completed_passes: progress.completed_passes.load(Ordering::SeqCst),
                        total_passes: progress.total_passes,
                        changed_files: 0,
                        modified_lines: 0,
                        exploration_requests,
                        guardrail_hits: progress.failed_passes.load(Ordering::SeqCst),
                        current_file: Some(progress.current_file.clone()),
                        current_profile: Some(progress.current_profile.clone()),
                        current_phase: Some(format!(
                            "{}: {tool_name}",
                            progress.current_phase
                        )),
                    },
                    "feedback": Vec::<crate::domain::ReviewFeedback>::new(),
                }),
            );
        } else {
            eprintln!(
                "[local-review-tool] call pass_tool_calls={} tool={} args={}",
                pass_exploration_requests,
                tool_name,
                compact_tool_args(args)
            );
        }
        rig::agent::ToolCallHookAction::Continue
    }
}

fn compact_tool_args(args: &str) -> String {
    let value = args.split_whitespace().collect::<Vec<_>>().join(" ");
    let max_chars = 500usize;
    if value.chars().count() <= max_chars {
        value
    } else {
        format!(
            "{}...<omitted>",
            value.chars().take(max_chars).collect::<String>()
        )
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub(super) enum ReviewToolError {
    #[error("{0}")]
    Rejected(String),
    #[error("Could not read repository context.")]
    ReadFailed,
}
