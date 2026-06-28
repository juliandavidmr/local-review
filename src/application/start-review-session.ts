import {
  createPlannedReviewSession,
  planReviewPasses,
  suggestReviewProfiles,
  type ChangeSetSnapshot,
  type ReviewIntent,
  type ReviewProfile,
  type ReviewSession,
  type SessionInstructions,
} from "../domain"
import type { ReviewBudget } from "../domain/review-plan"

export interface StartReviewSessionInput {
  readonly id: string
  readonly intent: ReviewIntent
  readonly changeSet: ChangeSetSnapshot
  readonly availableProfiles: readonly ReviewProfile[]
  readonly selectedProfileIds: readonly string[]
  readonly budget: ReviewBudget
  readonly instructions?: SessionInstructions
  readonly createdAt: string
}

export function startReviewSession(input: StartReviewSessionInput): ReviewSession {
  const selectedProfiles = selectProfiles(
    input.availableProfiles,
    input.changeSet,
    input.selectedProfileIds,
  )
  const plan = planReviewPasses(input.changeSet, selectedProfiles, input.budget)

  return createPlannedReviewSession({
    id: input.id,
    intent: input.intent,
    changeSet: input.changeSet,
    selectedProfiles,
    plan,
    instructions: input.instructions,
    createdAt: input.createdAt,
  })
}

function selectProfiles(
  availableProfiles: readonly ReviewProfile[],
  changeSet: ChangeSetSnapshot,
  selectedProfileIds: readonly string[],
): readonly ReviewProfile[] {
  const selectedIds = new Set(selectedProfileIds)
  if (selectedIds.size > 0) {
    return availableProfiles.filter((profile) => selectedIds.has(profile.id))
  }

  return suggestReviewProfiles(
    availableProfiles,
    changeSet.repositoryPath,
    changeSet.files,
  )
    .filter((suggestion) => suggestion.selected)
    .map((suggestion) => suggestion.profile)
}
