import type { ChangeSetSnapshot } from "./change-set"

export interface StaleCheckResult {
  readonly stale: boolean
  readonly reason?: "fingerprint_changed" | "change_set_id_changed"
}

export function checkReviewSessionStale(
  reviewedChangeSet: Pick<ChangeSetSnapshot, "id" | "fingerprint">,
  currentChangeSet: Pick<ChangeSetSnapshot, "id" | "fingerprint">,
): StaleCheckResult {
  if (reviewedChangeSet.fingerprint !== currentChangeSet.fingerprint) {
    return { stale: true, reason: "fingerprint_changed" }
  }

  if (reviewedChangeSet.id !== currentChangeSet.id) {
    return { stale: true, reason: "change_set_id_changed" }
  }

  return { stale: false }
}
