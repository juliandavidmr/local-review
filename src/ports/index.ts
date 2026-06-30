import type {
  ChangeSetSnapshot,
  ChangeSource,
  CodeLocation,
  EvidenceReference,
  ExplorationRequestType,
  ReviewFeedback,
  ReviewIntent,
  ReviewPassOutput,
  ReviewProfile,
  ReviewSession,
  ProviderConnectionStatus,
  ProviderSettings,
  ModelProviderSettings,
  SuggestedProfile,
} from "../domain"
import type { ReviewBudget, ReviewPlan, PlannedReviewPass } from "../domain/review-plan"
import type {
  ExplorationRound,
  RepositoryExplorationBudget,
} from "../domain/exploration"

export interface ModelProvider {
  readonly listModels: () => Promise<readonly ModelDescriptor[]>
  readonly runReviewPass: (input: RunReviewPassInput) => Promise<ReviewPassOutput>
  readonly rewriteForHumanTone: (input: HumanToneRewriteInput) => Promise<readonly ReviewFeedback[]>
}

export interface ConfigurableModelProvider extends ModelProvider {
  readonly settings: ModelProviderSettings
  readonly checkConnection: () => Promise<ProviderConnectionStatus>
}

export interface ModelDescriptor {
  readonly providerId: string
  readonly modelId: string
  readonly displayName: string
  readonly available: boolean
}

export interface RunReviewPassInput {
  readonly pass: PlannedReviewPass
  readonly plan: ReviewPlan
  readonly changeSet: ChangeSetSnapshot
  readonly profile?: ReviewProfile
  readonly sessionOverview?: SessionOverview
  readonly sessionInstructions?: string
}

export interface SessionOverview {
  readonly touchedFiles: readonly string[]
  readonly apparentIntent: string
  readonly affectedAreas: readonly string[]
  readonly generalRisks: readonly string[]
}

export interface HumanToneRewriteInput {
  readonly feedback: readonly ReviewFeedback[]
  readonly intent: ReviewIntent
}

export interface McpProvider {
  readonly listEnabledSources: () => Promise<readonly DecisionSupportSource[]>
  readonly requestContext: (input: McpContextRequest) => Promise<McpContextResult>
}

export interface ProviderSettingsStore {
  readonly loadSettings: () => Promise<ProviderSettings>
  readonly saveSettings: (settings: ProviderSettings) => Promise<void>
}

export interface DecisionSupportSource {
  readonly id: string
  readonly name: string
  readonly description?: string
}

export interface McpContextRequest {
  readonly sourceId: string
  readonly prompt: string
}

export interface McpContextResult {
  readonly sourceId: string
  readonly evidence: readonly EvidenceReference[]
  readonly summary: string
}

export interface RepositoryExplorer {
  readonly explore: (input: RepositoryExplorationRequest) => Promise<ExplorationRound>
}

export interface RepositoryExplorationRequest {
  readonly repositoryPath: string
  readonly requestType: ExplorationRequestType
  readonly request: string
  readonly budget: RepositoryExplorationBudget
}

export interface ProfileStore {
  readonly listProfiles: () => Promise<readonly ReviewProfile[]>
  readonly getProfile: (profileId: string) => Promise<ReviewProfile | undefined>
  readonly saveProfile: (profile: ReviewProfile) => Promise<void>
  readonly deleteProfile: (profileId: string) => Promise<void>
  readonly suggestProfiles: (input: SuggestProfilesInput) => Promise<readonly SuggestedProfile[]>
}

export interface SuggestProfilesInput {
  readonly repositoryPath: string
  readonly changeSet: ChangeSetSnapshot
  readonly manuallySelectedProfileIds?: readonly string[]
}

export interface ReviewHistoryStore {
  readonly saveSession: (session: ReviewSession) => Promise<void>
  readonly getSession: (sessionId: string) => Promise<ReviewSession | undefined>
  readonly listSessions: (repositoryPath?: string) => Promise<readonly ReviewSession[]>
}

export interface Publisher {
  readonly publish: (input: PublishReviewInput) => Promise<PublicationResult>
  readonly mapCodeLocation: (location: CodeLocation) => Promise<PublicationMapping>
}

export interface PublishReviewInput {
  readonly target: PublicationTarget
  readonly feedback: readonly ReviewFeedback[]
}

export interface PublicationTarget {
  readonly kind: "pull_request"
  readonly owner: string
  readonly repository: string
  readonly number: number
}

export interface PublicationMapping {
  readonly platform: string
  readonly path: string
  readonly position?: number
  readonly line?: number
  readonly side?: "LEFT" | "RIGHT"
}

export interface PublicationResult {
  readonly publishedFeedbackIds: readonly string[]
  readonly failed: readonly PublicationFailure[]
}

export interface PublicationFailure {
  readonly feedbackId: string
  readonly message: string
}

export interface ExecutionCapacityEstimator {
  readonly estimateReviewBudget: () => Promise<ReviewBudget>
}
