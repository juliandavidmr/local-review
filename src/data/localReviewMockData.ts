export type ReviewFeedbackState =
  | "draft"
  | "accepted"
  | "edited"
  | "dismissed"
  | "published"

export type ReviewSeverity =
  | "blocking"
  | "important"
  | "suggestion"
  | "question"
  | "nitpick"

export type ReviewFeedbackType = "inline" | "summary"

export type ReviewFeedbackItem = {
  id: string
  title: string
  type: ReviewFeedbackType
  severity: ReviewSeverity
  state: ReviewFeedbackState
  profileId: string
  profileName: string
  file: string
  line?: number
  body: string
  editableComment: string
  suggestedAction: string
  confidence: "high" | "medium" | "low"
  limitedContext: boolean
  quotedCode?: string
  evidence: string[]
  limitations: string[]
}

export type ReviewSessionMock = {
  repository: {
    name: string
    path: string
    branch: string
  }
  changeSource: {
    kind: string
    target: string
    intent: string
    snapshot: string
  }
  profiles: Array<{
    id: string
    name: string
    scope: string
    selected: boolean
  }>
  execution: {
    status: "running" | "completed" | "incomplete"
    completedPasses: number
    totalPasses: number
    changedFiles: number
    modifiedLines: number
    explorationRequests: number
    guardrailHits: number
  }
  feedback: ReviewFeedbackItem[]
  publication: {
    target: string
    totalComments: number
    inlineComments: number
    summaryComments: number
    limitedContextCount: number
    incompleteSession: boolean
  }
}

export const localReviewMockSession: ReviewSessionMock = {
  repository: {
    name: "local-review",
    path: "~/work/local-review",
    branch: "feature/review-workspace",
  },
  changeSource: {
    kind: "Pull request via gh",
    target: "PR #42 into main",
    intent: "Reviewing someone else's changes",
    snapshot: "19 files, 642 modified lines",
  },
  profiles: [
    {
      id: "correctness",
      name: "Correctness",
      scope: "Repository path",
      selected: true,
    },
    {
      id: "architecture",
      name: "Architecture",
      scope: "Global default",
      selected: true,
    },
    {
      id: "accessibility",
      name: "Accessibility",
      scope: "Folder path",
      selected: false,
    },
  ],
  execution: {
    status: "running",
    completedPasses: 17,
    totalPasses: 24,
    changedFiles: 19,
    modifiedLines: 642,
    explorationRequests: 38,
    guardrailHits: 1,
  },
  feedback: [
    {
      id: "fb-001",
      title: "Persist edited comments before batch publish",
      type: "inline",
      severity: "blocking",
      state: "accepted",
      profileId: "correctness",
      profileName: "Correctness",
      file: "src/features/publication/publish-review.ts",
      line: 84,
      body: "The batch publisher reads the original generated body after the user edits a comment, so approved wording can be lost at publication time.",
      editableComment:
        "Please publish the curated comment body here. Batch publish should use the accepted or edited text, not the original generated text.",
      suggestedAction:
        "Load the curated feedback text from session state before mapping comments to the publisher payload.",
      confidence: "high",
      limitedContext: false,
      quotedCode: "body: feedback.body,",
      evidence: [
        "Feedback state stores editableComment separately from body.",
        "Publication summary includes edited feedback in the batch.",
      ],
      limitations: ["Did not execute gh publication in the mock session."],
    },
    {
      id: "fb-002",
      title: "Show incomplete coverage before publishing",
      type: "summary",
      severity: "important",
      state: "edited",
      profileId: "architecture",
      profileName: "Architecture",
      file: "Review session",
      body: "The session can continue after a failed pass, but the publication flow needs a visible incomplete-session warning before batch publication.",
      editableComment:
        "This looks shippable with one condition: the publication step should make incomplete coverage explicit before publishing accepted comments.",
      suggestedAction:
        "Carry the incomplete session flag into the publication summary and require acknowledgement before batch publish.",
      confidence: "medium",
      limitedContext: true,
      evidence: [
        "One pass reached an exploration guardrail.",
        "Publication policy allows individual feedback with a warning.",
      ],
      limitations: [
        "Could not inspect final publisher adapter behavior in this pass.",
      ],
    },
    {
      id: "fb-003",
      title: "Keep profile suggestions scoped",
      type: "inline",
      severity: "suggestion",
      state: "draft",
      profileId: "architecture",
      profileName: "Architecture",
      file: "src/features/profiles/profile-picker.ts",
      line: 31,
      body: "The picker visually groups all global profiles with selected repository profiles, which makes the suggested priority harder to scan.",
      editableComment:
        "Consider keeping manually selected, folder-scoped, repo-scoped, and global profiles visually distinct so the selection priority is clear.",
      suggestedAction:
        "Render profile scope groups in priority order and keep unselected global profiles at the end.",
      confidence: "medium",
      limitedContext: false,
      quotedCode: "profiles.map((profile) => (",
      evidence: ["Profile selection priority is documented in MVP v1."],
      limitations: ["Only reviewed the profile picker surface."],
    },
    {
      id: "fb-004",
      title: "Validate quoted code with exact location text",
      type: "inline",
      severity: "important",
      state: "draft",
      profileId: "correctness",
      profileName: "Correctness",
      file: "src/domain/quality-checks.ts",
      line: 57,
      body: "The quality check accepts a quote when it appears anywhere in the selected text, which can pass even when the code location is imprecise.",
      editableComment:
        "Can we compare the quoted code against the exact selected line range? Substring matching can hide a bad location when similar code appears nearby.",
      suggestedAction:
        "Normalize whitespace and compare against the exact location text returned by the change set helper.",
      confidence: "high",
      limitedContext: false,
      quotedCode: "locationText.includes(feedback.quotedCode)",
      evidence: [
        "Inline feedback must map to the reviewed Change Set.",
        "The same helper is used before feedback enters curation.",
      ],
      limitations: ["Did not inspect generated diff edge cases for renamed files."],
    },
    {
      id: "fb-005",
      title: "Avoid publishing already published feedback",
      type: "inline",
      severity: "blocking",
      state: "accepted",
      profileId: "correctness",
      profileName: "Correctness",
      file: "src/domain/publication.ts",
      line: 22,
      body: "The batch eligibility helper should continue excluding published feedback so a rerun cannot accidentally post the same approved comment twice.",
      editableComment:
        "Please keep already published feedback out of the batch. The UI can display it, but the publisher should never receive it again.",
      suggestedAction:
        "Keep the publishable predicate centralized and use it in both the summary and publisher use case.",
      confidence: "high",
      limitedContext: false,
      quotedCode: "feedback.state === \"accepted\" || feedback.state === \"edited\"",
      evidence: [
        "MVP v1 says Published Feedback cannot be published again.",
        "Publication summary includes accepted and edited feedback only.",
      ],
      limitations: [],
    },
    {
      id: "fb-006",
      title: "Surface stale session state before curation actions",
      type: "summary",
      severity: "important",
      state: "draft",
      profileId: "architecture",
      profileName: "Architecture",
      file: "Review session",
      body: "The app has stale-session rules, but the workspace mock does not yet show how stale state disables or warns on publish actions.",
      editableComment:
        "When the reviewed snapshot no longer matches the repository, publish actions should be visually gated until the session is rerun.",
      suggestedAction:
        "Add a stale-session banner and pass the stale status into publication controls.",
      confidence: "medium",
      limitedContext: true,
      evidence: [
        "Review sessions are bound to a Change Set snapshot.",
        "Stale sessions must be rerun before publication.",
      ],
      limitations: ["UI is still mock-driven, so stale detection was not executed live."],
    },
    {
      id: "fb-007",
      title: "Keep repository exploration read-only",
      type: "inline",
      severity: "important",
      state: "edited",
      profileId: "architecture",
      profileName: "Architecture",
      file: "src/adapters/guarded-repository-explorer.ts",
      line: 38,
      body: "The guarded explorer currently refuses sensitive paths, which is good. Preserve the read-only boundary when replacing the mock implementation with real filesystem access.",
      editableComment:
        "When this becomes a real explorer, it should still only read/search/parse and must not execute project behavior or modify files.",
      suggestedAction:
        "Keep exploration methods explicit by request type and avoid exposing a general command runner through this adapter.",
      confidence: "medium",
      limitedContext: false,
      quotedCode: "Repository exploration guard accepted the request",
      evidence: [
        "Architecture v1 forbids running tests, linters, builds, or project validation during exploration.",
        "Sensitive file guardrails are part of the exploration boundary.",
      ],
      limitations: ["Real filesystem adapter is not implemented yet."],
    },
    {
      id: "fb-008",
      title: "Show limited-context feedback count in filters",
      type: "inline",
      severity: "suggestion",
      state: "draft",
      profileId: "accessibility",
      profileName: "Accessibility",
      file: "src/features/local-review/FeedbackWorkspace.tsx",
      line: 73,
      body: "Limited-context feedback is visible in the detail panel, but the list filters do not provide a quick way to isolate those items.",
      editableComment:
        "Consider adding a limited-context filter so reviewers can quickly audit comments produced near guardrail limits.",
      suggestedAction:
        "Add a boolean limited-context filter next to severity and state filters.",
      confidence: "low",
      limitedContext: false,
      quotedCode: "const matchesType = typeFilter === \"all\" || item.type === typeFilter",
      evidence: [
        "Limited Context Indicator is required in feedback detail and publication summary.",
      ],
      limitations: ["Did not review final visual treatment for the indicator."],
    },
    {
      id: "fb-009",
      title: "Group publication failures by feedback item",
      type: "summary",
      severity: "question",
      state: "draft",
      profileId: "correctness",
      profileName: "Correctness",
      file: "Publication flow",
      body: "Partial publication failures should leave failed items accepted or edited, but the mock does not yet show per-item failure recovery.",
      editableComment:
        "How will the UI show which comments failed and which remote comments were created successfully?",
      suggestedAction:
        "Model publication results as per-feedback outcomes and render retry actions for failed items only.",
      confidence: "medium",
      limitedContext: true,
      evidence: [
        "MVP v1 requires partial publication failures to preserve previous accepted or edited state.",
      ],
      limitations: ["Publisher adapter is still mock-only."],
    },
    {
      id: "fb-010",
      title: "Keep review history outside reviewed repositories",
      type: "inline",
      severity: "important",
      state: "accepted",
      profileId: "architecture",
      profileName: "Architecture",
      file: "src/adapters/in-memory-review-history-store.ts",
      line: 12,
      body: "The in-memory store is fine for the mock, but the real persistence adapter should target the app-owned configuration directory rather than the reviewed repository.",
      editableComment:
        "When replacing this store, please write session history under the app config directory, not inside the repository being reviewed.",
      suggestedAction:
        "Introduce a Tauri-backed ReviewHistoryStore that resolves an application-owned path such as ~/.local-review/.",
      confidence: "high",
      limitedContext: false,
      quotedCode: "private readonly sessions = new Map<string, ReviewSession>()",
      evidence: [
        "ADR 0001 stores review profiles and history outside repositories.",
      ],
      limitations: [],
    },
    {
      id: "fb-011",
      title: "Do not combine incompatible profile criteria",
      type: "summary",
      severity: "suggestion",
      state: "dismissed",
      profileId: "architecture",
      profileName: "Architecture",
      file: "Review plan",
      body: "The plan creates profile-specific passes, which keeps feedback traceable to each review profile. That should stay explicit even if future batching optimizes model calls.",
      editableComment:
        "This is mostly a design note: keep profile attribution visible if pass batching is introduced later.",
      suggestedAction:
        "Only optimize execution after preserving profileId and passId attribution in ReviewPassOutput.",
      confidence: "low",
      limitedContext: false,
      evidence: [
        "ADR 0006 favors profile-specific review passes for traceability.",
      ],
      limitations: ["Dismissed as non-actionable for the current slice."],
    },
    {
      id: "fb-012",
      title: "Handle empty feedback lists gracefully",
      type: "inline",
      severity: "nitpick",
      state: "published",
      profileId: "accessibility",
      profileName: "Accessibility",
      file: "src/features/local-review/FeedbackWorkspace.tsx",
      line: 157,
      body: "The empty state exists, but it only appears in the detail area. The list panel could also make it clear when filters hide all feedback.",
      editableComment:
        "Tiny polish: show an empty list state below the filters when no feedback matches the current filter set.",
      suggestedAction:
        "Render a compact empty state in the feedback list when filteredFeedback is empty.",
      confidence: "medium",
      limitedContext: false,
      quotedCode: "No feedback selected.",
      evidence: ["Filters can combine state, severity, profile, type, and text."],
      limitations: [],
    },
    {
      id: "fb-013",
      title: "Differentiate MCP evidence from repository evidence",
      type: "summary",
      severity: "question",
      state: "draft",
      profileId: "correctness",
      profileName: "Correctness",
      file: "Evidence model",
      body: "Evidence references include MCP and repository context, but the UI mock renders them as plain strings. Reviewers may need to know which evidence came from autonomous MCP use.",
      editableComment:
        "Can the detail panel label MCP evidence separately from repository exploration evidence?",
      suggestedAction:
        "Carry evidence kind into the presentation model and render source badges in the Evidence panel.",
      confidence: "medium",
      limitedContext: true,
      evidence: [
        "MCP Audit Log must be visible during or after review.",
        "Evidence stores references rather than full source snapshots.",
      ],
      limitations: ["Presentation model currently flattens evidence to strings."],
    },
    {
      id: "fb-014",
      title: "Keep large-change thresholds fixed for v1",
      type: "inline",
      severity: "nitpick",
      state: "draft",
      profileId: "architecture",
      profileName: "Architecture",
      file: "src/domain/change-set.ts",
      line: 74,
      body: "The large-change threshold is correctly fixed in code for v1. Avoid exposing this as a user setting until the architecture explicitly allows it.",
      editableComment:
        "Please keep these thresholds fixed for v1; making them configurable now would add product surface without a documented need.",
      suggestedAction:
        "Leave the constants internal and document them near the planning rule.",
      confidence: "high",
      limitedContext: false,
      quotedCode: "return changeSet.files.length > 10 || countModifiedLines(changeSet) > 800",
      evidence: [
        "MVP v1 fixes Large Change Set thresholds at >10 files or >800 modified lines.",
      ],
      limitations: [],
    },
    {
      id: "fb-015",
      title: "Make profile import failures recoverable",
      type: "summary",
      severity: "suggestion",
      state: "edited",
      profileId: "accessibility",
      profileName: "Accessibility",
      file: "Profile import",
      body: "Imported agent definitions are sources, not canonical profile storage. The UI should eventually explain failed imports without implying anything was written to the reviewed repository.",
      editableComment:
        "If a Claude Code or opencode agent cannot be imported, show a recoverable error and keep the reviewed repository untouched.",
      suggestedAction:
        "Route import errors through profile-store specific UI, not through repository setup failure.",
      confidence: "medium",
      limitedContext: false,
      evidence: [
        "ADR 0001 treats imported agent definitions as sources for application-owned Review Profiles.",
      ],
      limitations: ["Profile import UI is not implemented in the mock."],
    },
  ],
  publication: {
    target: "PR #42",
    totalComments: 6,
    inlineComments: 4,
    summaryComments: 2,
    limitedContextCount: 2,
    incompleteSession: true,
  },
}
