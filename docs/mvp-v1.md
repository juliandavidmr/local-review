# MVP v1

This document captures the first releasable scope for Local Review: a local-first desktop application for reviewing Git changes with local LLMs, curating generated feedback, and optionally publishing approved feedback to a pull request through `gh`.

## Product Shape

Local Review serves both Developers preparing their own changes and Maintainer/Reviewers evaluating someone else's changes. Both use one review flow: open a repository, choose a change source, select review profiles, run a review session, curate feedback, and optionally publish approved feedback.

The primary unit of work is a Review Session. A session is tied to a repository, a Review Intent, a Change Source, a Change Set snapshot, selected Review Profiles, Review Passes, generated Review Feedback, curation decisions, metrics, MCP audit data, and publication outcome.

## In Scope

- Installable desktop app built with Tauri, React, and TypeScript.
- Hexagonal architecture with clean domain/application logic separated from UI and infrastructure.
- Open any local Git repository.
- Build a Change Set from:
  - pull request via `gh`
  - working tree
  - commit
  - compare between refs
- Use `gh` as the only GitHub integration in v1.
- Review Profiles stored outside reviewed repositories under `~/.local-review/`.
- Global, repository path, and folder path Profile Scope.
- Import Claude Code, opencode, or similar local agent definitions into application-owned Review Profiles.
- One Markdown file per Review Profile, using minimal YAML frontmatter.
- No separate prompt versioning; Review History stores the profile snapshot used by each session.
- Session Instructions for one-off guidance that should not be saved into a Review Profile.
- Local Model Providers: Ollama and LM Studio.
- Autonomous MCP use by reviewer models when needed by the review context.
- MCP Audit Log visible after/during review.
- Reviewer model-requested Repository Exploration for reading files, searching related code, parsing structure, detecting generated files, and finding nearby symbols or imports.
- Repository Exploration Scope covers any non-ignored file in the reviewed repository.
- Repository Exploration respects `.gitignore` and other configured ignore files by default.
- Sensitive File Guardrail blocks sensitive files from entering model context by default.
- Repository Exploration Budget, Exploration Guardrails, and Repository Exploration Log for bounding and auditing model-requested exploration.
- Adaptive parallel Review Pass execution based on local Execution Capacity and provider saturation.
- Full Change Set coverage. Review Budget only segments work; it never skips diff coverage.
- File Review Segments as the primary Review Segment strategy.
- Hunk Review Partitions for changed files that exceed one review pass budget.
- Profile-Specific Review Passes for each selected review profile that applies to a file review segment.
- Internal Session Overview Pass for a Large Change Set, producing a Session Overview but no Review Feedback.
- Fixed validated Review Pass Output contract with complete metadata.
- Review Quality Checks after each Review Pass, using deterministic validation only.
- Incremental Curation while passes are still running.
- Review Workspace with a 30/70 layout:
  - left 30%: flat filterable feedback list
  - right 70%: selected feedback detail, diff/code context, editable comment, evidence references, and actions
- Filters for state, severity, profile, type, and free text.
- Individual and batch publication of Accepted or Edited feedback through `gh`.
- Human-Tone Rewrite before batch publication.
- Local Review History with Change Set snapshot, feedback states, publication outcome, operational metrics, MCP audit log, and profile snapshots.

## Out of Scope

- Autofix or code modification.
- Running tests, linters, typecheck, scanners, builds, or project validation commands.
- Manual overrides for ignored or sensitive files during Repository Exploration.
- Automatic feedback deduplication, merging, or conflict resolution.
- Global code quality score.
- Direct GitHub API integration.
- Editing remote comments after publication.
- Republishing already Published Feedback.
- Writing product configuration into reviewed repositories.
- Separate prompt version files.
- Per-MCP-call approval prompts.

## Review Flow

1. User opens a local Git repository.
2. User chooses Review Intent: preparing their own changes or reviewing someone else's changes.
3. User chooses a Change Source.
4. The app builds a Change Set snapshot.
5. The app suggests applicable Review Profiles by Profile Scope, with this priority:
   - manually selected profiles for the session
   - profiles scoped to the most specific folder path
   - profiles scoped to the repository path
   - global default profiles
   - other global profiles available but unselected
6. User confirms profiles, model/provider settings, and optional Session Instructions.
7. The app estimates Execution Capacity and plans Review Passes, including Repository Exploration Budgets.
8. For a Large Change Set, the app runs an internal Session Overview Pass that produces a Session Overview for later passes but no Review Feedback.
9. The app runs segmented Review Passes until the entire Change Set has valid coverage. During a pass, the reviewer model may request multiple Exploration Rounds within its Repository Exploration Budget and Exploration Guardrails.
10. Review Passes return validated structured output.
11. The app runs Review Quality Checks over pass output.
12. User curates feedback incrementally: accept, edit, dismiss, or leave draft.
13. Before batch publication, the user may apply a Human-Tone Rewrite to accepted/edited feedback.
14. User publishes individual accepted/edited feedback or a batch of accepted/edited feedback.
15. The app stores the Review Session in local Review History.

## Coverage and Failure Policy

The app must review the entire Change Set. v1 uses File Review Segments as the primary segmentation strategy: each changed file is planned as its own review segment. For each file review segment, the app creates Profile-Specific Review Passes for each selected review profile that applies to that file. If one changed file is too large for one model call, the app splits that file review segment into Hunk Review Partitions while preserving the file as the coverage unit. Review Budget is a segmentation limit, not a coverage limit.

To control cost, v1 does not run every available review profile automatically. The user confirms active profiles before review starts, and Review Profile selection rules only determine which profiles are suggested or applicable.

The Session Overview Pass is only used for a Large Change Set. In v1, a Large Change Set is any change set with more than 10 changed files or more than 800 modified lines. These thresholds are fixed in v1 and not user-configurable. The pass produces a concise internal Session Overview covering touched files, apparent change intent, affected areas, and general risks. It must not create curatable or publishable Review Feedback.

If a Review Pass returns invalid JSON or feedback outside the Change Set, the app shows the failure and retries that pass at most once. If the retry fails, that pass remains failed. A session with any uncovered Change Set segment becomes an Incomplete Review Session.

Each Review Pass Output must report a Review Pass Status:

- `completed`: the pass completed the review segment within its guardrails and produced any supported feedback.
- `completed_with_limited_context`: the pass produced usable feedback but reached exploration guardrails or lacked enough context to claim full confidence for the segment.
- `incomplete`: the pass could not produce usable coverage for the segment.

Review Pass Status is an observable output contract, not a claim about the model's internal reasoning. The app validates status against exploration usage, missing context, feedback evidence, and output metadata. A pass that exhausts guardrails without enough evidence must return `completed_with_limited_context` or `incomplete`, not confident unsupported feedback.

Review Quality Checks run after each pass and before feedback enters the session as curatable Review Feedback. These checks are deterministic in v1: they validate that inline feedback maps to the reviewed Change Set, required metadata is present, quoted code matches the selected location, severity and state values are valid, and the output follows the Review Pass Output contract. They do not run a second model verification pass, initiate new Repository Exploration, or run tests, linters, typecheck, scanners, builds, or project validation commands. Semantic quality is handled by the original Review Pass output fields, including confidence, evidence, suggested action, and stated limitations, plus human curation.

Repository Exploration may run for as many Exploration Rounds as the reviewer model needs, but only within hard Exploration Guardrails. v1 guardrails must include at least maximum exploration requests, maximum files inspected, maximum bytes or tokens added to review context, and maximum elapsed time per review pass. If guardrails are reached, the review pass must continue with the gathered context or fail as incomplete rather than looping.

Repository Exploration Scope includes any non-ignored file in the reviewed repository, not only files in the Change Set or nearby references. Exploration respects `.gitignore` and other configured ignore files by default. Sensitive files, such as environment files, private keys, certificates, dumps, and likely secret-bearing files, must not enter model context. v1 does not support manual overrides for ignored or sensitive files. Exploration still cannot modify files, execute project behavior, or bypass Repository Exploration Budget, Exploration Guardrails, Sensitive File Guardrail, Ignore Boundary, and Repository Exploration Log.

An Incomplete Review Session can expose valid feedback for curation, but must not be represented as a complete review. Publication of individual feedback can be allowed with clear incomplete-state warning; batch publication must make the incomplete state explicit before proceeding.

## Stale Sessions and Rerun

A Review Session is bound to the Change Set snapshot it reviewed. If the current repository or pull request state no longer matches that snapshot, the session becomes stale.

Stale Review Sessions must be rerun before publication. Rerun creates a new Review Session tied to the new snapshot. The previous session remains in Review History and can be used as reference only; it does not automatically transfer Accepted or Published state to new feedback.

## Feedback Model

Review Feedback can be:

- Inline Feedback: anchored to a Code Location inside the Change Set.
- Review Summary: not anchored to a specific line.

Feedback generated by a pass with `completed_with_limited_context` remains curatable and publishable, but the UI must show a Limited Context Indicator in the feedback detail and publication summary. The indicator informs the Maintainer/Reviewer without automatically blocking publication.

Feedback severities are:

- blocking
- important
- suggestion
- question
- nitpick

Feedback states are:

- draft
- accepted
- edited
- dismissed
- published

Accepted means both the feedback text and code location are approved by the user. Published Feedback cannot be published again.

## Review Pass Output Contract

Each Review Pass returns a fixed, validated structured result. The exact schema can evolve during implementation, but v1 feedback items must include:

- `id`
- `type`
- `severity`
- `profileId`
- `passId`
- `body`
- `suggestedAction`
- `confidence`
- `state`
- `modelProvider`
- `model`
- `createdAt`
- `evidence`
- `limitations`

Inline feedback must include:

- `codeLocation`
- `quotedCode`

Summary feedback must include:

- `relatedFiles`

Evidence stores references only, not full source snapshots. The Change Set snapshot is stored separately in Review History.

v1 does not include a separate model-based verification pass or automatic feedback deduplication. All Review Feedback that passes deterministic Review Quality Checks enters curation, even when another feedback item appears similar.

## Publication

Publication uses `gh` only in v1. The domain and application layers do not know about `gh`; they call publication ports. A local adapter maps platform-neutral Code Location and Review Feedback into GitHub-specific publication payloads.

Batch publication publishes only Accepted and Edited feedback. Draft, Dismissed, and already Published feedback are never included.

Before batch publication, the user can run a Human-Tone Rewrite over the batch. The rewrite can improve clarity, consistency, politeness, and reviewer voice, but it must preserve the approved meaning, severity, and code location of each feedback item. Rewritten feedback returns to the user for final approval before publication.

Before batch publication, the UI shows:

- target pull request
- total comments
- inline vs summary count
- severities included
- limited-context feedback count
- incomplete-session warning if applicable

Partial publication failures leave failed items in their previous accepted or edited state with a visible error.

## Architecture

The implementation follows lightweight DDD and hexagonal architecture.

Domain/application code owns:

- Review Session lifecycle
- Change Set and Change Source concepts
- Review Feedback states and severities
- Review Profile selection and scope rules
- Review Pass planning
- Repository Exploration budgets, guardrails, and logs
- coverage and stale-session rules
- publication decisions

Ports include:

- `GitProvider`
- `PullRequestProvider`
- `ModelProvider`
- `McpProvider`
- `ProfileStore`
- `ReviewHistoryStore`
- `Publisher`

Adapters include:

- Tauri filesystem/process adapter
- local Git adapter
- repository exploration adapter
- `gh` pull request and publisher adapter
- Ollama model adapter
- LM Studio model adapter
- MCP adapter
- future GitHub API or web backend adapters

React UI calls application use cases and renders state. It does not contain review rules, publication mapping, model-provider logic, or GitHub-specific behavior.
