import type { CodeLocation } from "./change-set"

export type FeedbackType = "inline" | "summary"

export type FeedbackSeverity =
  | "blocking"
  | "important"
  | "suggestion"
  | "question"
  | "nitpick"

export type FeedbackState =
  | "draft"
  | "accepted"
  | "edited"
  | "dismissed"
  | "published"

export interface EvidenceReference {
  readonly kind: "change_set" | "repository_context" | "mcp" | "profile_rule"
  readonly reference: string
  readonly note?: string
}

export interface ReviewFeedbackBase {
  readonly id: string
  readonly severity: FeedbackSeverity
  readonly profileId: string
  readonly passId: string
  readonly body: string
  readonly suggestedAction: string
  readonly confidence: number
  readonly state: FeedbackState
  readonly modelProvider: string
  readonly model: string
  readonly createdAt: string
  readonly evidence: readonly EvidenceReference[]
  readonly limitations: readonly string[]
  readonly hasLimitedContext?: boolean
}

export interface InlineFeedback extends ReviewFeedbackBase {
  readonly type: "inline"
  readonly codeLocation: CodeLocation
  readonly quotedCode: string
}

export interface SummaryFeedback extends ReviewFeedbackBase {
  readonly type: "summary"
  readonly relatedFiles: readonly string[]
}

export type ReviewFeedback = InlineFeedback | SummaryFeedback

export const feedbackSeverities: readonly FeedbackSeverity[] = [
  "blocking",
  "important",
  "suggestion",
  "question",
  "nitpick",
]

export const feedbackStates: readonly FeedbackState[] = [
  "draft",
  "accepted",
  "edited",
  "dismissed",
  "published",
]

export const feedbackTypes: readonly FeedbackType[] = ["inline", "summary"]

export function isPublishableFeedback(feedback: ReviewFeedback): boolean {
  return feedback.state === "accepted" || feedback.state === "edited"
}

export function markFeedbackPublished(feedback: ReviewFeedback): ReviewFeedback {
  return {
    ...feedback,
    state: "published",
  }
}
