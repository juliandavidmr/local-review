import {
  checkReviewSessionStale,
  markSessionStale,
  type ChangeSetSnapshot,
  type ReviewSession,
  type StaleCheckResult,
} from "../domain"

export interface StaleReviewSessionResult {
  readonly session: ReviewSession
  readonly staleCheck: StaleCheckResult
}

export function refreshReviewSessionStaleness(
  session: ReviewSession,
  currentChangeSet: ChangeSetSnapshot,
  updatedAt: string,
): StaleReviewSessionResult {
  const staleCheck = checkReviewSessionStale(session.changeSet, currentChangeSet)

  return {
    session: staleCheck.stale ? markSessionStale(session, updatedAt) : session,
    staleCheck,
  }
}
