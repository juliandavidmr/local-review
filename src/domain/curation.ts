import type { CodeLocation } from "./change-set"
import type { ReviewFeedback } from "./review-feedback"

export type CurationDecision =
  | { readonly type: "accept"; readonly feedbackId: string }
  | {
      readonly type: "edit"
      readonly feedbackId: string
      readonly body: string
      readonly codeLocation?: CodeLocation
      readonly quotedCode?: string
    }
  | {
      readonly type: "dismiss"
      readonly feedbackId: string
      readonly reason?: string
    }

export function applyCurationDecision(
  feedback: ReviewFeedback,
  decision: CurationDecision,
): ReviewFeedback {
  if (feedback.id !== decision.feedbackId) return feedback
  if (feedback.state === "published") return feedback

  switch (decision.type) {
    case "accept":
      return { ...feedback, state: "accepted" }
    case "edit":
      return editFeedback(feedback, decision)
    case "dismiss":
      return { ...feedback, state: "dismissed" }
  }
}

export function applyCurationDecisions(
  feedbackItems: readonly ReviewFeedback[],
  decisions: readonly CurationDecision[],
): readonly ReviewFeedback[] {
  return decisions.reduce(
    (items, decision) =>
      items.map((feedback) => applyCurationDecision(feedback, decision)),
    feedbackItems,
  )
}

function editFeedback(
  feedback: ReviewFeedback,
  decision: Extract<CurationDecision, { readonly type: "edit" }>,
): ReviewFeedback {
  if (feedback.type === "inline") {
    return {
      ...feedback,
      body: decision.body,
      state: "edited",
      codeLocation: decision.codeLocation ?? feedback.codeLocation,
      quotedCode: decision.quotedCode ?? feedback.quotedCode,
    }
  }

  return {
    ...feedback,
    body: decision.body,
    state: "edited",
  }
}
