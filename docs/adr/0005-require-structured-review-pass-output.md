# Require Structured Review Pass Output

Review passes must return a fixed, validated structured output instead of freeform Markdown. The UI, review history, publication mapping, filtering, editing, and metrics all depend on reliable feedback metadata that can be validated before entering the review session.

**Considered Options**

- Let models return Markdown and parse comments heuristically.
- Require JSON or equivalent structured output matching a stable schema.

**Consequences**

Each model adapter must enforce or repair output into the review pass output schema before domain/application code accepts it. Invalid output is a pass failure or retry condition, not partially trusted review feedback.
