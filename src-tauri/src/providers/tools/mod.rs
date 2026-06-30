mod read_file;
mod safety;
mod search;

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

pub(super) use read_file::ReadRepositoryFileTool;
pub(super) use safety::{is_sensitive_path, safe_repository_file};
pub(super) use search::SearchRepositoryTool;

#[derive(Clone)]
pub(super) struct ToolUsageHook {
    pub exploration_requests: Arc<AtomicU32>,
}

impl<M: rig::completion::CompletionModel> rig::agent::PromptHook<M> for ToolUsageHook {
    async fn on_tool_call(
        &self,
        _tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        _args: &str,
    ) -> rig::agent::ToolCallHookAction {
        self.exploration_requests.fetch_add(1, Ordering::SeqCst);
        rig::agent::ToolCallHookAction::Continue
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub(super) enum ReviewToolError {
    #[error("{0}")]
    Rejected(String),
    #[error("Could not read repository context.")]
    ReadFailed,
}
