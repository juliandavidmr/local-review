export interface ContextBudget {
  readonly maxTokens: number
}

export interface RepositoryExplorationGuardrails {
  readonly maxRequests: number
  readonly maxFilesInspected: number
  readonly maxBytesAdded: number
  readonly maxElapsedMs: number
}

export interface RepositoryExplorationBudget {
  readonly guardrails: RepositoryExplorationGuardrails
  readonly contextBudget: ContextBudget
}

export interface RepositoryExplorationUsage {
  readonly requests: number
  readonly filesInspected: number
  readonly bytesAdded: number
  readonly elapsedMs: number
}

export type ExplorationRequestType =
  | "read_file"
  | "search"
  | "parse_structure"
  | "detect_generated_file"
  | "find_symbol_context"

export interface ExplorationRound {
  readonly id: string
  readonly requestedAt: string
  readonly requestType: ExplorationRequestType
  readonly request: string
  readonly accessedPaths: readonly string[]
  readonly resultSummary: string
  readonly usageAfterRound: RepositoryExplorationUsage
}

export interface RepositoryExplorationLog {
  readonly rounds: readonly ExplorationRound[]
}

export function emptyExplorationUsage(): RepositoryExplorationUsage {
  return {
    requests: 0,
    filesInspected: 0,
    bytesAdded: 0,
    elapsedMs: 0,
  }
}

export function hasReachedExplorationGuardrail(
  usage: RepositoryExplorationUsage,
  guardrails: RepositoryExplorationGuardrails,
): boolean {
  return (
    usage.requests >= guardrails.maxRequests ||
    usage.filesInspected >= guardrails.maxFilesInspected ||
    usage.bytesAdded >= guardrails.maxBytesAdded ||
    usage.elapsedMs >= guardrails.maxElapsedMs
  )
}
