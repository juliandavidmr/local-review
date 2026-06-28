import {
  applyCurationDecision,
  type CurationDecision,
  type ReviewSession,
} from "../domain"

export function curateReviewFeedback(
  session: ReviewSession,
  decision: CurationDecision,
  updatedAt: string,
): ReviewSession {
  return {
    ...session,
    feedback: session.feedback.map((feedback) =>
      applyCurationDecision(feedback, decision),
    ),
    updatedAt,
  }
}
