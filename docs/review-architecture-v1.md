# Review Architecture v1

Local Review v1 uses an analysis-first review architecture that preserves full change set coverage while controlling compute cost. The review engine plans review work before model execution, but repository exploration happens only when the reviewer model requests it during a review pass.

## Core Flow

1. Build a Change Set snapshot from the selected Change Source.
2. Suggest applicable Review Profiles by Profile Scope.
3. The user confirms active profiles, model/provider settings, and Session Instructions.
4. Plan File Review Segments and Profile-Specific Review Passes.
5. For a Large Change Set, run an internal Session Overview Pass.
6. Run Review Passes until the entire Change Set has valid coverage.
7. Run deterministic Review Quality Checks over each Review Pass Output.
8. Send all valid Review Feedback to Incremental Curation.
9. Publish only Accepted or Edited feedback through the `gh` publisher.
10. Store the Review Session in local Review History.

## Review Planning

v1 uses File Review Segments as the primary segmentation strategy. Each changed file becomes a coverage unit, and the app creates Profile-Specific Review Passes for each selected Review Profile that applies to that file.

If a changed file exceeds one pass budget, the file remains the coverage unit but is internally split into Hunk Review Partitions. This keeps coverage traceable to the file while preventing oversized prompts.

To control cost, v1 does not run every available Review Profile automatically. Review Profile selection rules suggest applicable profiles; the user chooses which profiles are active for the session.

## Large Change Sets

A Large Change Set is any change set with more than 10 changed files or more than 800 modified lines. These thresholds are fixed in v1 and are not user-configurable.

For a Large Change Set, the app runs a Session Overview Pass before segmented review passes. This pass produces a concise internal Session Overview covering touched files, apparent change intent, affected areas, and general risks. It does not produce curatable or publishable Review Feedback.

## Repository Exploration

Repository Exploration is requested by the reviewer model, not performed automatically upfront. The reviewer model starts from the changed file, applicable Review Profile, Session Instructions, and any Session Overview. During a Review Pass, it may request reading, searching, parsing, generated-file detection, or symbol/import context.

Repository Exploration Scope covers any non-ignored file in the reviewed repository. Exploration respects `.gitignore` and other configured ignore files by default. Sensitive files, such as environment files, private keys, certificates, dumps, and likely secret-bearing files, must not enter model context. v1 does not support manual overrides for ignored or sensitive files.

Repository Exploration cannot modify files, execute project behavior, or run tests, linters, typecheck, scanners, builds, or project validation commands.

## Exploration Guardrails

The reviewer model may request as many Exploration Rounds as it needs, but only within hard Exploration Guardrails. v1 guardrails must include:

- maximum exploration requests
- maximum files inspected
- maximum bytes or tokens added to review context
- maximum elapsed time per Review Pass

If guardrails are reached, the Review Pass must continue with gathered context or return an incomplete status rather than loop.

Every Exploration Round is recorded in the Repository Exploration Log, including the request, accessed paths, result summary, and budget usage.

## Review Pass Output

Each Review Pass returns structured output. v1 feedback items include:

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

Inline feedback includes `codeLocation` and `quotedCode`. Summary feedback includes `relatedFiles`.

Each Review Pass Output reports a Review Pass Status:

- `completed`
- `completed_with_limited_context`
- `incomplete`

Review Pass Status is an observable output contract, not a claim about hidden model reasoning. The app validates status against exploration usage, missing context, feedback evidence, and output metadata.

## Quality Checks

v1 does not use a separate model-based Verification Pass. Review Quality Checks are deterministic only. They validate:

- output schema
- required metadata
- valid feedback type, state, and severity
- inline feedback maps to the Change Set
- `quotedCode` matches the selected location
- Review Pass Status is consistent with exploration usage and required metadata

Semantic quality relies on structured pass fields, especially confidence, evidence, suggested action, and limitations, plus human curation.

v1 does not automatically deduplicate Review Feedback. All feedback that passes deterministic Review Quality Checks enters curation, even when another feedback item appears similar.

## Limited Context

Feedback produced by a `completed_with_limited_context` pass remains curatable and publishable. The UI must show a Limited Context Indicator in the feedback detail and publication summary. The indicator informs the Maintainer/Reviewer without automatically blocking publication.

## Curation and Publication

Review Feedback enters curation as draft. The user may accept, edit, dismiss, or leave it as draft. Accepted means both the feedback text and code location are approved by the user.

Batch publication includes only Accepted and Edited feedback. Draft, Dismissed, and already Published feedback are never included. Before batch publication, the UI shows the target pull request, total comments, inline vs summary count, severities included, limited-context feedback count, and incomplete-session warning if applicable.

Publication uses `gh` only in v1 through a publisher adapter. Domain and application logic stay platform-neutral.
