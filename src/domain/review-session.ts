import type { ChangeSetSnapshot, ReviewIntent } from "./change-set"
import type { RepositoryExplorationLog } from "./exploration"
import type { ReviewFeedback } from "./review-feedback"
import type { ReviewPlan } from "./review-plan"
import type { ReviewProfile } from "./review-profile"

export type ReviewSessionStatus = "planned" | "running" | "complete" | "incomplete"

export interface SessionInstructions {
  readonly text: string
}

export interface ReviewSessionMetrics {
  readonly plannedPasses: number
  readonly completedPasses: number
  readonly failedPasses: number
}

export interface ReviewSession {
  readonly id: string
  readonly repositoryPath: string
  readonly intent: ReviewIntent
  readonly changeSet: ChangeSetSnapshot
  readonly selectedProfiles: readonly ReviewProfile[]
  readonly profileSnapshots: readonly ReviewProfile[]
  readonly plan: ReviewPlan
  readonly feedback: readonly ReviewFeedback[]
  readonly explorationLog: RepositoryExplorationLog
  readonly instructions?: SessionInstructions
  readonly status: ReviewSessionStatus
  readonly stale: boolean
  readonly metrics: ReviewSessionMetrics
  readonly createdAt: string
  readonly updatedAt: string
}

export function createPlannedReviewSession(input: {
  readonly id: string
  readonly intent: ReviewIntent
  readonly changeSet: ChangeSetSnapshot
  readonly selectedProfiles: readonly ReviewProfile[]
  readonly plan: ReviewPlan
  readonly instructions?: SessionInstructions
  readonly createdAt: string
}): ReviewSession {
  return {
    id: input.id,
    repositoryPath: input.changeSet.repositoryPath,
    intent: input.intent,
    changeSet: input.changeSet,
    selectedProfiles: input.selectedProfiles,
    profileSnapshots: input.selectedProfiles,
    plan: input.plan,
    feedback: [],
    explorationLog: { rounds: [] },
    instructions: input.instructions,
    status: "planned",
    stale: false,
    metrics: {
      plannedPasses: input.plan.passes.length,
      completedPasses: 0,
      failedPasses: 0,
    },
    createdAt: input.createdAt,
    updatedAt: input.createdAt,
  }
}

export function addFeedbackToSession(
  session: ReviewSession,
  feedback: readonly ReviewFeedback[],
  updatedAt: string,
): ReviewSession {
  return {
    ...session,
    feedback: [...session.feedback, ...feedback],
    updatedAt,
  }
}

export function markSessionStale(session: ReviewSession, updatedAt: string): ReviewSession {
  return {
    ...session,
    stale: true,
    updatedAt,
  }
}
