import { emptyExplorationUsage, type ReviewFeedback, type ReviewPassOutput } from "../domain"
import type {
  HumanToneRewriteInput,
  ModelDescriptor,
  ModelProvider,
  RunReviewPassInput,
} from "../ports"

export interface MockModelProviderOptions {
  readonly models?: readonly ModelDescriptor[]
  readonly feedbackByPassId?: ReadonlyMap<string, readonly ReviewFeedback[]>
}

const defaultMockModels: readonly ModelDescriptor[] = [
  {
    providerId: "mock",
    modelId: "mock-reviewer",
    displayName: "Mock Reviewer",
    available: true,
  },
]

export class MockModelProvider implements ModelProvider {
  private readonly models: readonly ModelDescriptor[]
  private readonly feedbackByPassId: ReadonlyMap<string, readonly ReviewFeedback[]>

  constructor(options: MockModelProviderOptions = {}) {
    this.models = options.models ?? defaultMockModels
    this.feedbackByPassId = options.feedbackByPassId ?? new Map()
  }

  async listModels(): Promise<readonly ModelDescriptor[]> {
    return this.models
  }

  async runReviewPass(input: RunReviewPassInput): Promise<ReviewPassOutput> {
    const model = this.models.find((candidate) => candidate.available) ?? this.models[0]
    const feedback = this.feedbackByPassId.get(input.pass.id) ?? []

    return {
      passId: input.pass.id,
      status: "completed",
      feedback,
      metadata: {
        modelProvider: model?.providerId ?? "mock",
        model: model?.modelId ?? "mock-reviewer",
        completedAt: new Date().toISOString(),
        explorationUsage: emptyExplorationUsage(),
      },
      limitations: ["MockModelProvider does not call a real model."],
    }
  }

  async rewriteForHumanTone(input: HumanToneRewriteInput): Promise<readonly ReviewFeedback[]> {
    return input.feedback
  }
}
