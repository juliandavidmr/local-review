# Local Review

A local-first code review tool for preparing, inspecting, and publishing review feedback on Git repositories.

## Language

**Developer**:
A person who wants feedback on changes before or during review.
_Avoid_: Coder, author

**Maintainer/Reviewer**:
A person responsible for evaluating changes and deciding which feedback should be published.
_Avoid_: Auditor, approver

**Review Intent**:
The user's reason for starting a review session, either preparing their own changes for review or evaluating someone else's changes.
_Avoid_: Mode, persona flow

**Review Session**:
A complete review effort for a repository and a selected set of changes, including generated observations, user decisions, and publication outcome.
_Avoid_: Job, run, scan

**Incomplete Review Session**:
A review session that produced usable feedback but did not achieve valid review coverage for the entire change set.
_Avoid_: Complete review, successful review

**Stale Review Session**:
A review session whose reviewed change set snapshot no longer matches the current repository or pull request state and must be rerun before publication.
_Avoid_: Current review, revalidated review

**Change Set**:
The explicit files and hunks selected for review in a review session. Context outside the change set can inform the review, but review feedback is anchored to the change set.
_Avoid_: Diff, patch, scope

**Change Source**:
The origin used to build a change set, such as a pull request, working tree, commit, or comparison between refs.
_Avoid_: Review target, input mode

**Review Budget**:
The per-pass processing limit used to split a review session into manageable parts while still covering the entire change set.
_Avoid_: Coverage limit, skipped scope

**Review Plan**:
The planned breakdown of a review session into prioritized review segments, initial context budgets, and review passes before expensive model evaluation begins.
_Avoid_: Prompt plan, execution script

**Risk Signal**:
A cheap indicator that a part of the change set may deserve more review attention, such as sensitive paths, risky code areas, unusually large changes, or missing related tests.
_Avoid_: Severity, finding, model suspicion

**Context Budget**:
The amount of review context a review pass may spend when the reviewer model requests additional repository exploration for a review segment.
_Avoid_: Token limit, model context window

**Context Bundle**:
The repository information available to a review pass for one review segment, beginning with the change set and expanding only through reviewer model-requested repository exploration.
_Avoid_: Full repository context, prompt dump

**Repository Exploration**:
Reviewer model-requested reading, searching, parsing, and lightweight inspection of repository files used to expand review context without modifying files or executing project behavior.
_Avoid_: Test run, validation command, build step

**Repository Exploration Scope**:
The repository-owned file space available for reviewer model-requested repository exploration during a review session.
_Avoid_: Change set only, nearby files only

**Ignore Boundary**:
The repository ignore rules that repository exploration respects by default, including `.gitignore` and other configured ignore files.
_Avoid_: Optional filter, hidden file preference

**Sensitive File Guardrail**:
A hard exploration guardrail that prevents sensitive files from entering model context by default.
_Avoid_: Best-effort privacy reminder, prompt-only secret handling

**Repository Exploration Budget**:
The per-session or per-pass limit on reviewer model-requested repository exploration.
_Avoid_: Unlimited tool access, hidden exploration

**Exploration Round**:
One reviewer model request for additional repository context and the resulting repository exploration response during a review pass.
_Avoid_: Tool loop, model turn

**Exploration Guardrail**:
A hard limit that prevents repository exploration from looping indefinitely or degrading review quality, such as request count, file count, byte count, token count, or elapsed time.
_Avoid_: Soft preference, prompt reminder

**Review Context**:
The repository information, project guidance, and enabled helper resources used to understand a change set during a review session.
_Avoid_: Prompt context, knowledge base

**Session Instructions**:
Additional user-provided guidance that applies only to one review session.
_Avoid_: Review profile, permanent rule

**Decision Support Source**:
An enabled resource, such as an MCP, that can provide relevant context or evidence when a review session needs it to evaluate a change set.
_Avoid_: Mandatory review step, passive context

**Autonomous MCP Use**:
The reviewer model's ability to call enabled MCPs during a review session whenever it decides they are needed.
_Avoid_: Manual MCP approval, fixed MCP step

**MCP Audit Log**:
A visible record of MCP calls made during a review session and the context they contributed.
_Avoid_: Permission prompt, hidden trace

**Repository Exploration Log**:
A visible record of repository exploration requested by the reviewer model during a review session and the context it contributed.
_Avoid_: Hidden file access, prompt transcript

**Review History**:
The local record of past review sessions, including their change set snapshot, review feedback states, publication outcome, metrics, and MCP audit log.
_Avoid_: Repository snapshot, cloud history

**Review Feedback**:
A review observation tied to the change set, with enough location, evidence, severity, and suggested action for a user to decide whether to keep it. Review feedback moves from draft to accepted, edited, dismissed, or published; accepted feedback has both its text and code location approved.
_Avoid_: Finding, issue, autofix

**Code Location**:
The platform-neutral position in a change set where inline feedback applies.
_Avoid_: GitHub line, review comment position

**Publication Mapping**:
The adapter-specific translation from review feedback and code location to the target platform's publication format.
_Avoid_: Code location, feedback location

**Inline Feedback**:
Review feedback anchored to a specific line or hunk in the change set.
_Avoid_: Line comment, code annotation

**Review Summary**:
Review feedback that summarizes the review session without being anchored to a specific line.
_Avoid_: General comment, report

**Feedback Severity**:
The decision weight assigned to review feedback: blocking, important, suggestion, question, or nitpick.
_Avoid_: Priority, risk level

**Review Profile**:
A reusable review configuration that defines the evaluative intent, criteria, rules, prompts, and context priorities for a review session. A review profile is stored as one Markdown file with minimal YAML frontmatter.
_Avoid_: Prompt, checklist

**Claude Agent Definition**:
A Claude Code agent file that can be used as a source for a review profile when it represents a review-oriented workflow.
_Avoid_: Review profile, required profile format

**Imported Agent Definition**:
An agent definition from Claude Code, opencode, or another local coding tool that can be imported into a review profile.
_Avoid_: Review profile, source of truth

**Review Rule**:
A reusable human-defined criterion that can be included in review profiles.
_Avoid_: Prompt, policy

**Review Pass**:
An individual evaluation run within a review session, usually tied to a review profile, model, or portion of the change set. Review passes produce review feedback for user curation.
_Avoid_: Profile run, model call

**Review Segment**:
A coherent portion of a change set selected for one or more review passes, usually grouped by file, folder, risk area, or related behavior.
_Avoid_: Chunk, token slice

**File Review Segment**:
A review segment whose primary boundary is one changed file from the change set.
_Avoid_: Logical area segment, feature segment

**Hunk Review Partition**:
A pass-sized partition inside a file review segment used when one changed file exceeds the review budget.
_Avoid_: Separate file segment, independent review scope

**Profile-Specific Review Pass**:
A review pass that evaluates one review segment using one selected review profile.
_Avoid_: Combined profile pass, mixed criteria pass

**Review Pass Output**:
The fixed, validated structured result returned by a review pass, including review feedback and required metadata.
_Avoid_: Freeform response, markdown output

**Review Pass Status**:
The observable completion state reported by a review pass output: completed, completed with limited context, or incomplete.
_Avoid_: Model certainty, hidden reasoning state

**Limited Context Completion**:
A review pass status where the pass produced usable feedback but reached exploration guardrails or otherwise lacked enough context to claim full confidence for the review segment.
_Avoid_: Complete review, failed review

**Limited Context Indicator**:
A visible UI and publication-review signal that review feedback came from a pass completed with limited context.
_Avoid_: Hidden caveat, publication blocker

**Review Quality Check**:
A post-pass validation step that checks structured output quality, including code location validity and whether feedback follows the required output contract.
_Avoid_: Test run, linter

**Session Overview Pass**:
A review pass that builds internal understanding of the entire change set before segmented review passes inspect the details.
_Avoid_: Summary only, skipped review

**Session Overview**:
A concise internal map of a large change set used to orient later review passes without producing review feedback.
_Avoid_: Review summary, publishable feedback

**Large Change Set**:
A change set large enough to require a session overview before segmented review passes begin.
_Avoid_: Complex change, risky change

**Execution Capacity**:
The local machine and model provider capacity available for running review passes without saturating the user's environment.
_Avoid_: Worker count, thread pool

**Model Provider**:
A local service that exposes models for review passes.
_Avoid_: Backend, cloud provider

**Model**:
A concrete language model available through a model provider.
_Avoid_: Provider, engine

**Publication Choice**:
The user's decision to publish accepted review feedback individually or as a complete batch after curating the review session.
_Avoid_: Auto-posting, sync

**Human-Tone Rewrite**:
An optional pre-publication rewrite that makes accepted feedback read like a coherent human review without changing the user's approved meaning.
_Avoid_: New review, autofix

**Published Feedback**:
Review feedback that has already been sent to the target review platform and cannot be published again.
_Avoid_: Republishable feedback, editable remote comment

**Incremental Curation**:
The user's ability to inspect, edit, accept, or dismiss review feedback while review passes are still running.
_Avoid_: Live publishing, final-only review

**Incremental Rerun**:
A rerun that reviews only the review segments affected by a changed change set snapshot while preserving previous session history as reference.
_Avoid_: Resume, cache hit, partial review

**Review Workspace**:
The main review interface, with a filterable feedback list on the left and selected feedback detail with code context on the right.
_Avoid_: Dashboard, report view

**User Library**:
A personal library of review profiles, prompts, and review rules stored outside any reviewed repository in the application's own configuration.
_Avoid_: Repository library, imported agent definition

**Repository Library**:
A repository-owned source of review guidance that can inform a review session but does not own or store the application's review profiles.
_Avoid_: Profile store, source of truth

**Profile Scope**:
The applicability boundary of a review profile, either global or associated with a specific repository or folder path.
_Avoid_: Profile location, saved repo config
