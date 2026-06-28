import {
  addFeedbackToSession,
  runReviewQualityChecks,
  type QualityCheckResult,
  type ReviewPassOutput,
  type ReviewSession,
} from "../domain"

export interface ProcessReviewPassOutputResult {
  readonly session: ReviewSession
  readonly qualityCheck: QualityCheckResult
}

export function processReviewPassOutput(
  session: ReviewSession,
  output: ReviewPassOutput,
  updatedAt: string,
): ProcessReviewPassOutputResult {
  const qualityCheck = runReviewQualityChecks(session.changeSet, output)
  const nextSession = addFeedbackToSession(
    session,
    qualityCheck.acceptedFeedback,
    updatedAt,
  )

  return {
    session: {
      ...nextSession,
      metrics: {
        ...nextSession.metrics,
        completedPasses:
          output.status === "incomplete"
            ? nextSession.metrics.completedPasses
            : nextSession.metrics.completedPasses + 1,
        failedPasses:
          output.status === "incomplete"
            ? nextSession.metrics.failedPasses + 1
            : nextSession.metrics.failedPasses,
      },
      status: deriveSessionStatus(nextSession.metrics.plannedPasses, {
        completedPasses:
          output.status === "incomplete"
            ? nextSession.metrics.completedPasses
            : nextSession.metrics.completedPasses + 1,
        failedPasses:
          output.status === "incomplete"
            ? nextSession.metrics.failedPasses + 1
            : nextSession.metrics.failedPasses,
      }),
    },
    qualityCheck,
  }
}

function deriveSessionStatus(
  plannedPasses: number,
  metrics: Pick<ReviewSession["metrics"], "completedPasses" | "failedPasses">,
): ReviewSession["status"] {
  const finishedPasses = metrics.completedPasses + metrics.failedPasses
  if (finishedPasses < plannedPasses) return "running"
  return metrics.failedPasses > 0 ? "incomplete" : "complete"
}
