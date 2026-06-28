import {
  getLocationText,
  isLocationInChangeSet,
  type ChangeSetSnapshot,
} from "./change-set"
import {
  feedbackSeverities,
  feedbackStates,
  feedbackTypes,
  type ReviewFeedback,
} from "./review-feedback"
import type { ReviewPassOutput } from "./review-pass"
import { hasReachedExplorationGuardrail } from "./exploration"

export type QualityCheckSeverity = "error" | "warning"

export interface QualityCheckIssue {
  readonly severity: QualityCheckSeverity
  readonly code: string
  readonly message: string
  readonly feedbackId?: string
}

export interface QualityCheckResult {
  readonly valid: boolean
  readonly issues: readonly QualityCheckIssue[]
  readonly acceptedFeedback: readonly ReviewFeedback[]
}

export function runReviewQualityChecks(
  changeSet: ChangeSetSnapshot,
  output: ReviewPassOutput,
): QualityCheckResult {
  const outputIssues = validateOutputMetadata(output)
  const feedbackResults = output.feedback.map((feedback) =>
    validateFeedback(changeSet, output, feedback),
  )
  const feedbackIssues = feedbackResults.flatMap((result) => result.issues)
  const issues = [...outputIssues, ...feedbackIssues]

  return {
    valid: issues.every((issue) => issue.severity !== "error"),
    issues,
    acceptedFeedback: feedbackResults
      .filter((result) => result.valid)
      .map((result) => result.feedback),
  }
}

function validateOutputMetadata(output: ReviewPassOutput): readonly QualityCheckIssue[] {
  const issues: QualityCheckIssue[] = []

  if (!output.passId) {
    issues.push(error("missing_pass_id", "Review pass output must include a passId."))
  }

  if (!output.metadata.modelProvider || !output.metadata.model) {
    issues.push(error("missing_model_metadata", "Review pass output must include model provider and model."))
  }

  if (output.status === "completed" && output.metadata.missingContext?.length) {
    issues.push(warning("completed_with_missing_context", "Completed pass reports missing context."))
  }

  if (
    output.status === "completed" &&
    hasReachedExplorationGuardrail(
      output.metadata.explorationUsage,
      {
        maxRequests: output.metadata.explorationUsage.requests,
        maxFilesInspected: output.metadata.explorationUsage.filesInspected,
        maxBytesAdded: output.metadata.explorationUsage.bytesAdded,
        maxElapsedMs: output.metadata.explorationUsage.elapsedMs,
      },
    ) &&
    output.limitations.length > 0
  ) {
    issues.push(warning("completed_with_limitations", "Completed pass includes limitations that may require limited-context status."))
  }

  if (output.status === "incomplete" && output.feedback.length > 0) {
    issues.push(error("incomplete_with_feedback", "Incomplete review passes cannot contribute feedback."))
  }

  return issues
}

function validateFeedback(
  changeSet: ChangeSetSnapshot,
  output: ReviewPassOutput,
  feedback: ReviewFeedback,
): { readonly valid: boolean; readonly feedback: ReviewFeedback; readonly issues: readonly QualityCheckIssue[] } {
  const issues: QualityCheckIssue[] = []

  if (!feedback.id) issues.push(error("missing_feedback_id", "Feedback must include an id.", feedback.id))
  if (feedback.passId !== output.passId) {
    issues.push(error("pass_id_mismatch", "Feedback passId must match the review pass output.", feedback.id))
  }

  if (!feedbackTypes.includes(feedback.type)) {
    issues.push(error("invalid_feedback_type", "Feedback type is invalid.", feedback.id))
  }

  if (!feedbackSeverities.includes(feedback.severity)) {
    issues.push(error("invalid_severity", "Feedback severity is invalid.", feedback.id))
  }

  if (!feedbackStates.includes(feedback.state)) {
    issues.push(error("invalid_state", "Feedback state is invalid.", feedback.id))
  }

  if (!feedback.profileId || !feedback.body || !feedback.suggestedAction) {
    issues.push(error("missing_required_feedback_text", "Feedback must include profile, body, and suggested action.", feedback.id))
  }

  if (feedback.confidence < 0 || feedback.confidence > 1) {
    issues.push(error("invalid_confidence", "Feedback confidence must be between 0 and 1.", feedback.id))
  }

  if (feedback.evidence.length === 0) {
    issues.push(warning("missing_evidence", "Feedback should include at least one evidence reference.", feedback.id))
  }

  if (feedback.type === "inline") {
    if (!isLocationInChangeSet(changeSet, feedback.codeLocation)) {
      issues.push(error("location_outside_change_set", "Inline feedback must map to the reviewed change set.", feedback.id))
    }

    const locationText = getLocationText(changeSet, feedback.codeLocation)
    if (locationText !== undefined && locationText.trim() !== feedback.quotedCode.trim()) {
      issues.push(error("quoted_code_mismatch", "Inline feedback quotedCode must match the selected location.", feedback.id))
    }
  } else if (feedback.relatedFiles.length === 0) {
    issues.push(warning("summary_without_related_files", "Summary feedback should include related files.", feedback.id))
  }

  return {
    valid: issues.every((issue) => issue.severity !== "error"),
    feedback:
      output.status === "completed_with_limited_context"
        ? { ...feedback, hasLimitedContext: true }
        : feedback,
    issues,
  }
}

function error(
  code: string,
  message: string,
  feedbackId?: string,
): QualityCheckIssue {
  return { severity: "error", code, message, feedbackId }
}

function warning(
  code: string,
  message: string,
  feedbackId?: string,
): QualityCheckIssue {
  return { severity: "warning", code, message, feedbackId }
}
