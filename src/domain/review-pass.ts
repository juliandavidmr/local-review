import type { RepositoryExplorationUsage } from "./exploration"
import type { ReviewFeedback } from "./review-feedback"

export type ReviewPassStatus =
  | "completed"
  | "completed_with_limited_context"
  | "incomplete"

export interface ReviewPassOutputMetadata {
  readonly modelProvider: string
  readonly model: string
  readonly completedAt: string
  readonly explorationUsage: RepositoryExplorationUsage
  readonly missingContext?: readonly string[]
}

export interface ReviewPassOutput {
  readonly passId: string
  readonly status: ReviewPassStatus
  readonly feedback: readonly ReviewFeedback[]
  readonly metadata: ReviewPassOutputMetadata
  readonly limitations: readonly string[]
}
