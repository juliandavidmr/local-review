# Allow Autonomous MCP Use During Review

Reviewer models may call enabled MCPs freely during a review session when they decide additional context or evidence is needed. This favors review quality and model autonomy over per-call user approval, while keeping MCP usage within the locally configured set of available MCPs.

**Considered Options**

- Require user approval before each MCP call.
- Let review profiles strictly declare when MCPs can be used.
- Allow the reviewer model to invoke enabled MCPs autonomously.

**Consequences**

The review engine must treat enabled MCPs as part of the reviewer model's available decision support surface. Users control which MCPs are configured overall, but v1 does not interrupt the review flow for per-call approval.
