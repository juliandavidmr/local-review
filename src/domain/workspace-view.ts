import type { ProviderSettings } from "./provider-settings"

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

export type ReviewProfileScopeKind = "global" | "repository" | "folder"

export type ReviewProfileItem = {
  id: string
  name: string
  scope: string
  scopeKind: ReviewProfileScopeKind
  selected: boolean
  enabledByDefault: boolean
  criteria: string[]
  fileGlobs: string[]
  prompt: string
}

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
  confidence: "high" | "medium" | "low" | string
  limitedContext: boolean
  quotedCode?: string
  evidence: string[]
  limitations: string[]
  codeLocation?: {
    filePath: string
    startLine: number
    endLine: number
    side: string
  }
}

export type ReviewWorkspaceView = {
  repository: {
    name: string
    path: string
    branch: string
    headSha?: string
  }
  changeSource: {
    kind: string
    target: string
    intent: string
    snapshot: string
  }
  profiles: ReviewProfileItem[]
  providerSettings: ProviderSettings
  execution: {
    status: "running" | "completed" | "incomplete" | string
    completedPasses: number
    totalPasses: number
    changedFiles: number
    modifiedLines: number
    explorationRequests: number
    guardrailHits: number
    currentFile?: string
    currentProfile?: string
    currentPhase?: string
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
