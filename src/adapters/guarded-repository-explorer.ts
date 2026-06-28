import type { ExplorationRound, RepositoryExplorationUsage } from "../domain"
import { emptyExplorationUsage } from "../domain"
import type { RepositoryExplorationRequest, RepositoryExplorer } from "../ports"

const sensitivePathPatterns: readonly RegExp[] = [
  /(^|\/)\.git(\/|$)/,
  /(^|\/)\.ssh(\/|$)/,
  /(^|\/)\.aws(\/|$)/,
  /(^|\/)\.config\/gh(\/|$)/,
  /(^|\/)\.env($|[./-])/,
  /(^|\/)node_modules(\/|$)/,
  /(^|\/)dist(\/|$)/,
  /(^|\/)build(\/|$)/,
  /(^|\/)coverage(\/|$)/,
  /(^|\/)target(\/|$)/,
  /(^|\/)vendor(\/|$)/,
  /(^|\/)(id_rsa|id_dsa|id_ecdsa|id_ed25519)(\.pub)?$/,
  /(^|\/)(npm|yarn|pnpm|bun)-debug\.log$/,
]

export class GuardedRepositoryExplorer implements RepositoryExplorer {
  private usage: RepositoryExplorationUsage = emptyExplorationUsage()

  async explore(input: RepositoryExplorationRequest): Promise<ExplorationRound> {
    const startedAt = Date.now()
    const requestedPaths = extractPathCandidates(input.request)
    const refusedPaths = requestedPaths.filter(isSensitiveRepositoryPath)
    const nextUsage = this.nextUsage(startedAt)
    const refused = refusedPaths.length > 0

    this.usage = nextUsage

    return {
      id: `exploration:${Date.now()}:${this.usage.requests}`,
      requestedAt: new Date(startedAt).toISOString(),
      requestType: input.requestType,
      request: input.request,
      accessedPaths: refused ? [] : requestedPaths,
      resultSummary: refused
        ? `Refused repository exploration for sensitive path(s): ${refusedPaths.join(", ")}.`
        : "Repository exploration guard accepted the request, but no real repository access is implemented.",
      usageAfterRound: this.usage,
    }
  }

  getUsage(): RepositoryExplorationUsage {
    return this.usage
  }

  private nextUsage(startedAt: number): RepositoryExplorationUsage {
    return {
      requests: this.usage.requests + 1,
      filesInspected: this.usage.filesInspected,
      bytesAdded: this.usage.bytesAdded,
      elapsedMs: this.usage.elapsedMs + Math.max(Date.now() - startedAt, 0),
    }
  }
}

export function isSensitiveRepositoryPath(path: string): boolean {
  const normalized = normalizeRepositoryPath(path)
  return sensitivePathPatterns.some((pattern) => pattern.test(normalized))
}

export function extractPathCandidates(request: string): readonly string[] {
  return request
    .split(/\s+/)
    .map((token) => token.replace(/^['"`(<[]+|['"`),\]>:;]+$/g, ""))
    .filter((token) => token.includes("/") || token.startsWith("."))
    .filter((token, index, tokens) => tokens.indexOf(token) === index)
}

function normalizeRepositoryPath(path: string): string {
  return path.replace(/\\/g, "/").replace(/\/+/g, "/").replace(/^\.\//, "")
}
