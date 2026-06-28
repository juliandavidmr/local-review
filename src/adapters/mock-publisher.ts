import type { CodeLocation, ReviewFeedback } from "../domain"
import type {
  PublicationMapping,
  PublicationResult,
  Publisher,
  PublishReviewInput,
} from "../ports"

export class MockPublisher implements Publisher {
  private readonly publishedFeedback = new Map<string, ReviewFeedback>()

  async publish(input: PublishReviewInput): Promise<PublicationResult> {
    for (const feedback of input.feedback) {
      this.publishedFeedback.set(feedback.id, feedback)
    }

    return {
      publishedFeedbackIds: input.feedback.map((feedback) => feedback.id),
      failed: [],
    }
  }

  async mapCodeLocation(location: CodeLocation): Promise<PublicationMapping> {
    return {
      platform: "mock",
      path: location.filePath,
      line: location.startLine,
      side: location.side === "old" ? "LEFT" : "RIGHT",
    }
  }

  getPublishedFeedback(): readonly ReviewFeedback[] {
    return Array.from(this.publishedFeedback.values())
  }
}
