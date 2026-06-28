export type LocalModelProviderKind = "ollama" | "lm_studio"

export interface ModelProviderSettings {
  readonly id: string
  readonly kind: LocalModelProviderKind
  readonly name: string
  readonly baseUrl: string
  readonly enabled: boolean
  readonly selectedModelId?: string
  readonly useForHumanToneRewrite?: boolean
}

export interface McpSourceSettings {
  readonly id: string
  readonly name: string
  readonly description?: string
  readonly enabled: boolean
}

export interface ProviderSettings {
  readonly modelProviders: readonly ModelProviderSettings[]
  readonly mcpSources: readonly McpSourceSettings[]
  readonly execution: ExecutionCapacitySettings
}

export interface ExecutionCapacitySettings {
  readonly maxParallelReviewPasses: number
  readonly adaptiveParallelismEnabled: boolean
}

export interface ProviderConnectionStatus {
  readonly providerId: string
  readonly ok: boolean
  readonly message: string
}

export const defaultProviderSettings: ProviderSettings = {
  modelProviders: [
    {
      id: "ollama",
      kind: "ollama",
      name: "Ollama",
      baseUrl: "http://localhost:11434",
      enabled: false,
    },
    {
      id: "lm-studio",
      kind: "lm_studio",
      name: "LM Studio",
      baseUrl: "http://localhost:1234/v1",
      enabled: true,
    },
  ],
  mcpSources: [
    {
      id: "filesystem",
      name: "Filesystem context",
      description: "Repository-owned files available through guarded exploration.",
      enabled: true,
    },
    {
      id: "github",
      name: "GitHub context",
      description: "Pull request and issue context from configured MCP sources.",
      enabled: false,
    },
  ],
  execution: {
    maxParallelReviewPasses: 2,
    adaptiveParallelismEnabled: true,
  },
}

export function updateModelProviderSettings(
  settings: ProviderSettings,
  providerId: string,
  update: (provider: ModelProviderSettings) => ModelProviderSettings,
): ProviderSettings {
  return {
    ...settings,
    modelProviders: settings.modelProviders.map((provider) =>
      provider.id === providerId ? update(provider) : provider,
    ),
  }
}

export function selectSingleModelProvider(
  settings: ProviderSettings,
  providerId: string,
  selectedModelId?: string,
): ProviderSettings {
  return {
    ...settings,
    modelProviders: settings.modelProviders.map((provider) => {
      const selected = provider.id === providerId

      return {
        ...provider,
        enabled: selected,
        selectedModelId: selected ? selectedModelId ?? provider.selectedModelId : undefined,
        useForHumanToneRewrite: selected ? provider.useForHumanToneRewrite : false,
      }
    }),
  }
}

export function updateMcpSourceSettings(
  settings: ProviderSettings,
  sourceId: string,
  update: (source: McpSourceSettings) => McpSourceSettings,
): ProviderSettings {
  return {
    ...settings,
    mcpSources: settings.mcpSources.map((source) =>
      source.id === sourceId ? update(source) : source,
    ),
  }
}
