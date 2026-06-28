import { useEffect, useRef, useState } from "react"
import { ArrowCounterClockwise, StopCircle } from "@phosphor-icons/react"
import {
  buildChangeSet,
  cancelReviewSession,
  type ChangeSetSnapshot,
  loadProfiles,
  loadProviderSettings,
  openRepository,
  runReviewSession,
  saveProviderSettings,
  saveProfile,
  type RepositoryDescriptor,
  type ReviewWorkspaceSession,
} from "@/adapters/tauri-local-review-api"
import { WorkspaceShell } from "@/components/layout/WorkspaceShell"
import { Button } from "@/components/ui/button"
import { defaultProviderSettings, type ProviderSettings } from "@/domain"
import type { ReviewProfileItem } from "@/domain/workspace-view"

import { ExecutionStatus } from "./ExecutionStatus"
import { FeedbackWorkspace } from "./FeedbackWorkspace"
import { InitialSetupScreen } from "./InitialSetupScreen"
import { ProfileManager } from "./ProfileManager"
import { ProviderSettingsPanel } from "./ProviderSettingsPanel"
import { PublicationSummary } from "./PublicationSummary"
import { SetupOverview } from "./SetupOverview"

export function LocalReviewWorkspace() {
  const [session, setSession] = useState<ReviewWorkspaceSession | null>(null)
  const [profiles, setProfiles] = useState<ReviewProfileItem[]>([])
  const [providerSettings, setProviderSettings] = useState<ProviderSettings>(
    defaultProviderSettings,
  )
  const [loading, setLoading] = useState(true)
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const activeReviewId = useRef<string | null>(null)

  useEffect(() => {
    async function loadInitialState() {
      try {
        const [storedProfiles, storedSettings] = await Promise.all([
          loadProfiles(),
          loadProviderSettings(),
        ])
        setProfiles(storedProfiles)
        setProviderSettings(storedSettings)
      } catch (unknownError) {
        setError(errorMessage(unknownError))
      } finally {
        setLoading(false)
      }
    }

    void loadInitialState()
  }, [])

  function updateProfile(
    profileId: string,
    update: (profile: (typeof profiles)[number]) => (typeof profiles)[number],
  ) {
    setProfiles((current) =>
      current.map((profile) => {
        const nextProfile = profile.id === profileId ? update(profile) : profile
        if (nextProfile.id === profileId) {
          void saveProfile(nextProfile)
        }
        return nextProfile
      }),
    )
  }

  if (loading) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-muted/40 p-6">
        <div className="border border-border bg-card p-6 text-sm text-muted-foreground">
          Loading local review configuration...
        </div>
      </main>
    )
  }

  if (!session) {
    return (
      <InitialSetupScreen
        error={error}
        initialProfiles={profiles}
        isRunning={running}
        onComplete={async (setup) => {
          setRunning(true)
          setError(null)
          let startedReviewId: string | null = null
          try {
            const repository = await openRepository(setup.repositoryPath)
            const changeSet = await buildChangeSet(
              repository.path,
              setup.reviewSourceKind,
            )

            if (changeSet.files.length === 0) {
              throw new Error(
                "The selected review source produced 0 changed files. Choose Current branch for committed branch changes, Staged changes for git add changes, or Unstaged changes for local working tree edits.",
              )
            }

            const savedSettings = await saveProviderSettings(
              setup.providerSettings,
            )
            const profilesWithRepositoryScope = setup.profiles.map((profile) =>
              profile.scopeKind === "repository"
                ? { ...profile, scope: repository.path }
                : profile,
            )

            for (const profile of profilesWithRepositoryScope) {
              await saveProfile(profile)
            }

            const reviewId = createReviewId()
            startedReviewId = reviewId
            activeReviewId.current = reviewId
            setProfiles(profilesWithRepositoryScope)
            setProviderSettings(savedSettings)
            setSession(
              createRunningSession({
                changeSet,
                profiles: profilesWithRepositoryScope,
                providerSettings: savedSettings,
                repository,
              }),
            )

            const nextSession = await runReviewSession({
              reviewId,
              repository,
              changeSet,
              profiles: profilesWithRepositoryScope,
              providerSettings: savedSettings,
            })

            if (activeReviewId.current === reviewId) {
              setSession(nextSession)
            }
          } catch (unknownError) {
            setError(errorMessage(unknownError))
          } finally {
            if (startedReviewId && activeReviewId.current === startedReviewId) {
              activeReviewId.current = null
            }
            setRunning(false)
          }
        }}
        providerSettings={providerSettings}
      />
    )
  }

  return (
    <WorkspaceShell
      actions={
        <>
          {running ? (
            <Button onClick={stopActiveReview} variant="destructive">
              <StopCircle className="size-4" />
              Stop review
            </Button>
          ) : null}
          <Button disabled={running} onClick={() => setSession(null)} variant="outline">
            <ArrowCounterClockwise className="size-4" />
            New review
          </Button>
        </>
      }
      subtitle="Open a repository, review a change set, curate generated feedback, and publish through gh."
      title="Review Workspace"
    >
      <div className="space-y-5">
        <ProviderSettingsPanel
          onChange={(settings) => {
            setProviderSettings(settings)
            if (session) {
              setSession({ ...session, providerSettings: settings })
            }
          }}
          settings={providerSettings}
        />
        <ProfileManager
          onCreateProfile={(profile) =>
            setProfiles((current) => [profile, ...current])
          }
          onDeleteProfile={(profileId) =>
            setProfiles((current) =>
              current.filter((profile) => profile.id !== profileId),
            )
          }
          onToggleDefault={(profileId, enabledByDefault) =>
            updateProfile(profileId, (profile) => ({
              ...profile,
              enabledByDefault,
            }))
          }
          onToggleSelected={(profileId, selected) =>
            updateProfile(profileId, (profile) => ({
              ...profile,
              selected,
            }))
          }
          profiles={profiles}
          repositoryPath={session.repository.path}
        />
        <SetupOverview session={session} />
        <ExecutionStatus execution={session.execution} />
        <FeedbackWorkspace feedback={session.feedback} />
        <PublicationSummary publication={session.publication} />
      </div>
    </WorkspaceShell>
  )

  async function stopActiveReview() {
    const reviewId = activeReviewId.current
    if (!reviewId) return

    const confirmed = window.confirm(
      "Stop the current review? The current model pass may finish in the background, but no more passes will be started.",
    )
    if (!confirmed) return

    activeReviewId.current = null
    setRunning(false)
    try {
      await cancelReviewSession(reviewId)
    } catch (unknownError) {
      setError(errorMessage(unknownError))
    }
    setSession((current) =>
      current
        ? {
            ...current,
            execution: {
              ...current.execution,
              status: "cancelled",
            },
            publication: {
              ...current.publication,
              incompleteSession: true,
            },
          }
        : current,
    )
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function createReviewId(): string {
  return `review-${Date.now()}-${Math.random().toString(36).slice(2)}`
}

function createRunningSession(input: {
  repository: RepositoryDescriptor
  changeSet: ChangeSetSnapshot
  profiles: ReviewProfileItem[]
  providerSettings: ProviderSettings
}): ReviewWorkspaceSession {
  const activeProfiles = input.profiles.filter((profile) => profile.selected)
  const reviewableFiles = input.changeSet.files.filter((file) => !file.isGenerated)
  const modifiedLines = input.changeSet.files.reduce(
    (total, file) => total + file.additions + file.deletions,
    0,
  )

  return {
    repository: {
      name: input.repository.name,
      path: input.repository.path,
      branch: input.repository.currentBranch ?? "detached",
      headSha: input.repository.headSha,
    },
    changeSource: {
      kind: "Preparing review",
      target: "Selected change source",
      intent: "Running model review passes",
      snapshot: `${input.changeSet.files.length} files, ${modifiedLines} modified lines`,
    },
    changeSet: input.changeSet,
    profiles: activeProfiles,
    providerSettings: input.providerSettings,
    execution: {
      status: "running",
      completedPasses: 0,
      totalPasses: reviewableFiles.length * activeProfiles.length,
      changedFiles: input.changeSet.files.length,
      modifiedLines,
      explorationRequests: 0,
      guardrailHits: 0,
    },
    feedback: [],
    publication: {
      target: "gh pull request publication not selected",
      totalComments: 0,
      inlineComments: 0,
      summaryComments: 0,
      limitedContextCount: 0,
      incompleteSession: false,
    },
  }
}
