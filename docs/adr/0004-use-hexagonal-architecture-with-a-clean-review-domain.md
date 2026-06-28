# Use Hexagonal Architecture With a Clean Review Domain

The application will separate review domain logic and application use cases from UI and infrastructure using a lightweight hexagonal architecture. This keeps the first local Tauri implementation from leaking `gh`, filesystem, local model providers, MCP plumbing, or React concerns into the business rules, and leaves room for a future web UI backed by different adapters such as the GitHub API.

**Considered Options**

- Put review logic directly in React components and Tauri commands.
- Keep a TypeScript core but allow infrastructure-specific dependencies inside it.
- Use a clean domain/application layer with ports and adapters.

**Consequences**

Domain and application code must depend on ports such as Git, pull request, model provider, MCP, profile store, review history, and publisher interfaces. Tauri, `gh`, Ollama, LM Studio, local filesystem, GitHub API, and future web backends are adapters behind those ports rather than dependencies of the review domain.
