import { useEffect, useState } from "react"
import { ArrowCounterClockwise } from "@phosphor-icons/react"
import {
  buildWorkingTreeChangeSet,
  loadProfiles,
  loadProviderSettings,
  openRepository,
  runReviewSession,
  saveProviderSettings,
  saveProfile,
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
          try {
            const repository = await openRepository(setup.repositoryPath)
            const changeSet = await buildWorkingTreeChangeSet(repository.path)
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

            const nextSession = await runReviewSession({
              repository,
              changeSet,
              profiles: profilesWithRepositoryScope,
              providerSettings: savedSettings,
            })

            setProfiles(profilesWithRepositoryScope)
            setProviderSettings(savedSettings)
            setSession(nextSession)
          } catch (unknownError) {
            setError(errorMessage(unknownError))
          } finally {
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
        <Button onClick={() => setSession(null)} variant="outline">
          <ArrowCounterClockwise className="size-4" />
          New review
        </Button>
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
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
