import {
  getPublicationEligibility,
  markFeedbackPublished,
  type PublicationEligibility,
  type ReviewSession,
} from "../domain"
import type {
  PublicationResult,
  PublicationTarget,
  Publisher,
} from "../ports"

export interface PreparePublicationInput {
  readonly session: ReviewSession
  readonly allowIncompleteSession?: boolean
}

export function preparePublication(
  input: PreparePublicationInput,
): PublicationEligibility {
  return getPublicationEligibility(input.session, {
    allowIncompleteSession: input.allowIncompleteSession,
  })
}

export async function publishAcceptedFeedback(input: {
  readonly session: ReviewSession
  readonly target: PublicationTarget
  readonly publisher: Publisher
  readonly allowIncompleteSession?: boolean
  readonly updatedAt: string
}): Promise<{ readonly session: ReviewSession; readonly result: PublicationResult }> {
  const eligibility = preparePublication(input)
  if (!eligibility.eligible) {
    throw new Error(`Review session is not eligible for publication: ${eligibility.blockers.join(", ")}`)
  }

  const result = await input.publisher.publish({
    target: input.target,
    feedback: eligibility.feedback,
  })
  const publishedIds = new Set(result.publishedFeedbackIds)

  return {
    session: {
      ...input.session,
      feedback: input.session.feedback.map((feedback) =>
        publishedIds.has(feedback.id) ? markFeedbackPublished(feedback) : feedback,
      ),
      updatedAt: input.updatedAt,
    },
    result,
  }
}
