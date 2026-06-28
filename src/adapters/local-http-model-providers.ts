import {
  emptyExplorationUsage,
  type ModelProviderSettings,
  type ProviderConnectionStatus,
  type ReviewFeedback,
  type ReviewPassOutput,
} from "../domain"
import type {
  ConfigurableModelProvider,
  HumanToneRewriteInput,
  ModelDescriptor,
  RunReviewPassInput,
} from "../ports"

export class OllamaModelProvider implements ConfigurableModelProvider {
  readonly settings: ModelProviderSettings

  constructor(settings: ModelProviderSettings) {
    this.settings = settings
  }

  async checkConnection(): Promise<ProviderConnectionStatus> {
    try {
      const response = await fetch(`${trimTrailingSlash(this.settings.baseUrl)}/api/tags`)
      return {
        providerId: this.settings.id,
        ok: response.ok,
        message: response.ok ? "Ollama is reachable." : `Ollama returned ${response.status}.`,
      }
    } catch (error) {
      return {
        providerId: this.settings.id,
        ok: false,
        message: error instanceof Error ? error.message : "Ollama is unreachable.",
      }
    }
  }

  async listModels(): Promise<readonly ModelDescriptor[]> {
    const response = await fetch(`${trimTrailingSlash(this.settings.baseUrl)}/api/tags`)
    if (!response.ok) return []

    const payload = (await response.json()) as {
      models?: Array<{ name?: string; model?: string }>
    }

    return (payload.models ?? []).flatMap((model) => {
      const modelId = model.model ?? model.name
      if (!modelId) return []

      return {
        providerId: this.settings.id,
        modelId,
        displayName: modelId,
        available: true,
      }
    })
  }

  async runReviewPass(input: RunReviewPassInput): Promise<ReviewPassOutput> {
    return createNotImplementedOutput(input, this.settings)
  }

  async rewriteForHumanTone(input: HumanToneRewriteInput): Promise<readonly ReviewFeedback[]> {
    return input.feedback
  }
}

export class LmStudioModelProvider implements ConfigurableModelProvider {
  readonly settings: ModelProviderSettings

  constructor(settings: ModelProviderSettings) {
    this.settings = settings
  }

  async checkConnection(): Promise<ProviderConnectionStatus> {
    try {
      const response = await fetch(`${trimTrailingSlash(this.settings.baseUrl)}/models`)
      return {
        providerId: this.settings.id,
        ok: response.ok,
        message: response.ok ? "LM Studio is reachable." : `LM Studio returned ${response.status}.`,
      }
    } catch (error) {
      return {
        providerId: this.settings.id,
        ok: false,
        message: error instanceof Error ? error.message : "LM Studio is unreachable.",
      }
    }
  }

  async listModels(): Promise<readonly ModelDescriptor[]> {
    const response = await fetch(`${trimTrailingSlash(this.settings.baseUrl)}/models`)
    if (!response.ok) return []

    const payload = (await response.json()) as {
      data?: Array<{ id?: string }>
    }

    return (payload.data ?? []).flatMap((model) => {
      if (!model.id) return []

      return {
        providerId: this.settings.id,
        modelId: model.id,
        displayName: model.id,
        available: true,
      }
    })
  }

  async runReviewPass(input: RunReviewPassInput): Promise<ReviewPassOutput> {
    return createNotImplementedOutput(input, this.settings)
  }

  async rewriteForHumanTone(input: HumanToneRewriteInput): Promise<readonly ReviewFeedback[]> {
    return input.feedback
  }
}

export function createModelProviderAdapter(settings: ModelProviderSettings): ConfigurableModelProvider {
  return settings.kind === "ollama"
    ? new OllamaModelProvider(settings)
    : new LmStudioModelProvider(settings)
}

function createNotImplementedOutput(
  input: RunReviewPassInput,
  settings: ModelProviderSettings,
): ReviewPassOutput {
  return {
    passId: input.pass.id,
    status: "incomplete",
    feedback: [],
    metadata: {
      modelProvider: settings.id,
      model: settings.selectedModelId ?? "unselected",
      completedAt: new Date().toISOString(),
      explorationUsage: emptyExplorationUsage(),
      missingContext: ["Review pass generation is not wired to the local provider yet."],
    },
    limitations: ["Provider configuration is implemented; generation is still adapter work."],
  }
}

function trimTrailingSlash(value: string): string {
  return value.replace(/\/+$/, "")
}
