import type { ReviewFeedback } from "./review-feedback"
import { isPublishableFeedback } from "./review-feedback"
import type { ReviewSession } from "./review-session"

export interface PublicationEligibility {
  readonly eligible: boolean
  readonly feedback: readonly ReviewFeedback[]
  readonly blockers: readonly string[]
  readonly warnings: readonly string[]
  readonly summary: PublicationSummary
}

export interface PublicationSummary {
  readonly totalComments: number
  readonly inlineCount: number
  readonly summaryCount: number
  readonly severities: readonly string[]
  readonly limitedContextCount: number
  readonly hasIncompleteSessionWarning: boolean
}

export function getPublicationEligibility(
  session: ReviewSession,
  options: { readonly allowIncompleteSession?: boolean } = {},
): PublicationEligibility {
  const feedback = session.feedback.filter(isPublishableFeedback)
  const blockers: string[] = []
  const warnings: string[] = []

  if (session.stale) blockers.push("stale_session")
  if (feedback.length === 0) blockers.push("no_publishable_feedback")
  if (session.status === "incomplete" && !options.allowIncompleteSession) {
    blockers.push("incomplete_session")
  }
  if (session.status === "incomplete") warnings.push("incomplete_session")

  return {
    eligible: blockers.length === 0,
    feedback,
    blockers,
    warnings,
    summary: summarizePublicationFeedback(feedback, session.status === "incomplete"),
  }
}

export function summarizePublicationFeedback(
  feedback: readonly ReviewFeedback[],
  hasIncompleteSessionWarning: boolean,
): PublicationSummary {
  const severities = Array.from(new Set(feedback.map((item) => item.severity)))

  return {
    totalComments: feedback.length,
    inlineCount: feedback.filter((item) => item.type === "inline").length,
    summaryCount: feedback.filter((item) => item.type === "summary").length,
    severities,
    limitedContextCount: feedback.filter((item) => item.hasLimitedContext).length,
    hasIncompleteSessionWarning,
  }
}
