use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::domain::{ExecutionStatus, ReviewFeedback};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewProgressEvent {
    review_id: String,
    execution: ExecutionStatus,
    feedback: Vec<ReviewFeedback>,
}

pub(super) fn emit_review_progress(
    app: &AppHandle,
    review_id: &str,
    status: &str,
    completed_passes: u32,
    total_passes: u32,
    failed_passes: u32,
    exploration_requests: u32,
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
                exploration_requests,
                guardrail_hits: failed_passes,
            },
            feedback,
        },
    );
}
