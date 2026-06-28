import type { EvidenceReference, McpSourceSettings } from "../domain"
import type {
  DecisionSupportSource,
  McpContextRequest,
  McpContextResult,
  McpProvider,
} from "../ports"

export class ConfigurableMcpProvider implements McpProvider {
  private readonly sources: readonly McpSourceSettings[]

  constructor(sources: readonly McpSourceSettings[]) {
    this.sources = sources
  }

  async listEnabledSources(): Promise<readonly DecisionSupportSource[]> {
    return this.sources
      .filter((source) => source.enabled)
      .map((source) => ({
        id: source.id,
        name: source.name,
        description: source.description,
      }))
  }

  async requestContext(input: McpContextRequest): Promise<McpContextResult> {
    const source = this.sources.find(
      (candidate) => candidate.id === input.sourceId && candidate.enabled,
    )
    const evidence: EvidenceReference[] = source
      ? [
          {
            kind: "mcp",
            reference: source.id,
            note: input.prompt,
          },
        ]
      : []

    return {
      sourceId: input.sourceId,
      evidence,
      summary: source
        ? "MCP source is enabled, but live MCP calls are not wired in this adapter."
        : "MCP source is disabled or unavailable.",
    }
  }
}
