# Store Review Profiles Outside Repositories

Review profiles, imported agent definitions, settings, and review history are stored under the application's own user configuration directory, such as `~/.local-review/`, rather than inside reviewed repositories. This keeps the tool local-first and portable across arbitrary repositories while still allowing profiles to be global or associated with specific repository and folder paths.

**Considered Options**

- Store profiles in repository-native agent folders such as `.claude/agents/`.
- Store profiles in an application-owned configuration directory outside the repository.

**Consequences**

The application can import Claude Code, opencode, and similar agent definitions, but imported files are sources, not the canonical profile store. Repository-specific behavior is represented by profile scope metadata instead of files written into the repository.
