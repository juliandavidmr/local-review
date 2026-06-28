import type { ReviewBudget } from "../domain"
import type { ExecutionCapacityEstimator } from "../ports"

export class LocalExecutionCapacityEstimator implements ExecutionCapacityEstimator {
  private readonly maxParallelReviewPasses: number

  constructor(maxParallelReviewPasses: number) {
    this.maxParallelReviewPasses = Math.max(1, maxParallelReviewPasses)
  }

  async estimateReviewBudget(): Promise<ReviewBudget> {
    return {
      maxModifiedLinesPerPass: 240,
      contextTokensPerPass: 24_000,
      explorationBudget: {
        contextBudget: {
          maxTokens: 8_000 * this.maxParallelReviewPasses,
        },
        guardrails: {
          maxRequests: 8,
          maxFilesInspected: 24,
          maxBytesAdded: 120_000,
          maxElapsedMs: 120_000,
        },
      },
    }
  }
}
