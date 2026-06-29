import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"

import type { ProviderConnectionStatus, ProviderSettings } from "@/domain"
import type {
  ReviewFeedbackItem,
  ReviewProfileItem,
  ReviewWorkspaceView,
} from "@/domain/workspace-view"
import type { ModelDescriptor } from "@/ports"

export type RepositoryDescriptor = {
  path: string
  name: string
  currentBranch?: string
  headSha?: string
}

export type ChangeSetSnapshot = {
  id: string
  repositoryPath: string
  source: unknown
  files: Array<{
    path: string
    additions: number
    deletions: number
    hunks: unknown[]
    isGenerated: boolean
  }>
  createdAt: string
  fingerprint: string
}

export type ReviewChangeSourceKind =
  | "current_branch"
  | "staged_changes"
  | "unstaged_changes"
  | "working_tree"

export type ReviewWorkspaceSession = ReviewWorkspaceView & {
  changeSet: ChangeSetSnapshot
}

export type ReviewProgressEvent = {
  reviewId: string
  execution: ReviewWorkspaceView["execution"]
  feedback: ReviewFeedbackItem[]
}

type RawReviewWorkspaceSession = Omit<
  ReviewWorkspaceSession,
  "repository" | "changeSource"
> & {
  repository: RepositoryDescriptor
  changeSource: string
}

export async function openRepository(
  repositoryPath: string,
): Promise<RepositoryDescriptor> {
  return invoke("open_repository", { repositoryPath })
}

export async function buildWorkingTreeChangeSet(
  repositoryPath: string,
): Promise<ChangeSetSnapshot> {
  return buildChangeSet(repositoryPath, "working_tree")
}

export async function buildChangeSet(
  repositoryPath: string,
  sourceKind: ReviewChangeSourceKind,
): Promise<ChangeSetSnapshot> {
  return invoke("build_change_set", {
    source: {
      type: sourceKind,
      repository_path: repositoryPath,
    },
  })
}

export async function loadProfiles(): Promise<ReviewProfileItem[]> {
  return invoke("load_profiles")
}

export async function saveProfile(
  profile: ReviewProfileItem,
): Promise<ReviewProfileItem[]> {
  return invoke("save_profile", { profile })
}

export async function deleteProfile(
  profileId: string,
): Promise<ReviewProfileItem[]> {
  return invoke("delete_profile", { profileId })
}

export async function loadProviderSettings(): Promise<ProviderSettings> {
  return invoke("load_provider_settings")
}

export async function loadReviewSessions(): Promise<ReviewWorkspaceSession[]> {
  const sessions = await invoke<RawReviewWorkspaceSession[]>("load_review_sessions")
  return sessions.map(toReviewWorkspaceSession)
}

export async function loadLatestReviewSession(): Promise<ReviewWorkspaceSession | null> {
  const session = await invoke<RawReviewWorkspaceSession | null>(
    "load_latest_review_session",
  )
  return session ? toReviewWorkspaceSession(session) : null
}

export async function saveReviewSession(
  session: ReviewWorkspaceSession,
): Promise<ReviewWorkspaceSession> {
  const saved = await invoke<RawReviewWorkspaceSession>("save_review_session", {
    session,
  })
  return toReviewWorkspaceSession(saved)
}

export async function updateReviewFeedback(input: {
  sessionId: string
  feedbackId: string
  feedback: ReviewFeedbackItem
}): Promise<ReviewWorkspaceSession> {
  const saved = await invoke<RawReviewWorkspaceSession>("update_review_feedback", input)
  return toReviewWorkspaceSession(saved)
}

export async function saveProviderSettings(
  settings: ProviderSettings,
): Promise<ProviderSettings> {
  return invoke("save_provider_settings", { settings })
}

export async function listProviderModels(
  provider: ProviderSettings["modelProviders"][number],
): Promise<ModelDescriptor[]> {
  return invoke("list_provider_models", { provider })
}

export async function checkProviderConnection(
  provider: ProviderSettings["modelProviders"][number],
): Promise<ProviderConnectionStatus> {
  return invoke("check_provider_connection", { provider })
}

export async function cancelReviewSession(reviewId: string): Promise<void> {
  return invoke("cancel_review_session", { reviewId })
}

export async function listenReviewProgress(
  onProgress: (event: ReviewProgressEvent) => void,
): Promise<() => void> {
  return listen<ReviewProgressEvent>("review-progress", (event) => {
    onProgress(event.payload)
  })
}

export async function runReviewSession(input: {
  reviewId: string
  repository: RepositoryDescriptor
  changeSet: ChangeSetSnapshot
  profiles: ReviewProfileItem[]
  providerSettings: ProviderSettings
}): Promise<ReviewWorkspaceSession> {
  const session = await invoke<RawReviewWorkspaceSession>(
    "run_review_session",
    input,
  )

  return toReviewWorkspaceSession(session)
}

function toReviewWorkspaceSession(
  session: RawReviewWorkspaceSession,
): ReviewWorkspaceSession {
  return {
    ...session,
    repository: {
      name: session.repository.name,
      path: session.repository.path,
      branch: session.repository.currentBranch ?? "detached",
      headSha: session.repository.headSha,
    },
    changeSource: {
      kind: session.changeSource,
      target: session.changeSource,
      intent: changeSourceIntent(session.changeSource),
      snapshot: `${session.execution.changedFiles} files, ${session.execution.modifiedLines} modified lines`,
    },
  }
}

function changeSourceIntent(changeSource: string): string {
  switch (changeSource) {
    case "Current branch":
      return "Reviewing branch changes against its base"
    case "Staged changes":
      return "Reviewing local changes staged for commit"
    case "Unstaged changes":
      return "Reviewing local unstaged working tree changes"
    default:
      return "Reviewing local changes"
  }
}
