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
  ],
  publication: {
    target: "PR #42",
    totalComments: 2,
    inlineComments: 1,
    summaryComments: 1,
    limitedContextCount: 1,
    incompleteSession: true,
  },
}
