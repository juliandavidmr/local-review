import {
  planReviewPasses,
  suggestReviewProfiles,
  type ChangeSetSnapshot,
  type ReviewProfile,
  type SuggestedProfile,
} from "../domain"
import type { ReviewBudget, ReviewPlan } from "../domain/review-plan"

export interface PlanReviewInput {
  readonly changeSet: ChangeSetSnapshot
  readonly availableProfiles: readonly ReviewProfile[]
  readonly selectedProfileIds?: readonly string[]
  readonly budget: ReviewBudget
}

export interface PlannedReviewSessionCore {
  readonly suggestedProfiles: readonly SuggestedProfile[]
  readonly selectedProfiles: readonly ReviewProfile[]
  readonly plan: ReviewPlan
}

export function planReview(input: PlanReviewInput): PlannedReviewSessionCore {
  const suggestedProfiles = suggestReviewProfiles(
    input.availableProfiles,
    input.changeSet.repositoryPath,
    input.changeSet.files,
    input.selectedProfileIds,
  )
  const selectedProfiles = suggestedProfiles
    .filter((suggestion) => suggestion.selected)
    .map((suggestion) => suggestion.profile)

  return {
    suggestedProfiles,
    selectedProfiles,
    plan: planReviewPasses(input.changeSet, selectedProfiles, input.budget),
  }
}
