import { useMemo, useState } from "react"
import { WorkspaceShell } from "@/components/layout/WorkspaceShell"
import { localReviewMockSession } from "@/data/localReviewMockData"
import type { ProviderSettings } from "@/domain"

import { ExecutionStatus } from "./ExecutionStatus"
import { FeedbackWorkspace } from "./FeedbackWorkspace"
import { InitialSetupScreen } from "./InitialSetupScreen"
import { ProfileManager } from "./ProfileManager"
import { ProviderSettingsPanel } from "./ProviderSettingsPanel"
import { PublicationSummary } from "./PublicationSummary"
import { SetupOverview } from "./SetupOverview"

export function LocalReviewWorkspace() {
  const [setupComplete, setSetupComplete] = useState(false)
  const [repositoryPath, setRepositoryPath] = useState("")
  const [profiles, setProfiles] = useState(
    localReviewMockSession.profiles.map((profile) => ({
      ...profile,
      selected: false,
    })),
  )
  const [providerSettings, setProviderSettings] = useState(
    localReviewMockSession.providerSettings,
  )
  const session = useMemo(
    () => ({
      ...localReviewMockSession,
      repository: {
        ...localReviewMockSession.repository,
        path: repositoryPath || localReviewMockSession.repository.path,
      },
      profiles,
      providerSettings,
      execution: {
        ...localReviewMockSession.execution,
        totalPasses:
          providerSettings.execution.maxParallelReviewPasses *
          localReviewMockSession.execution.changedFiles,
      },
    }),
    [profiles, providerSettings, repositoryPath],
  )

  function updateProfile(
    profileId: string,
    update: (profile: (typeof profiles)[number]) => (typeof profiles)[number],
  ) {
    setProfiles((current) =>
      current.map((profile) =>
        profile.id === profileId ? update(profile) : profile,
      ),
    )
  }

  if (!setupComplete) {
    return (
      <InitialSetupScreen
        initialProfiles={profiles}
        onComplete={(setup) => {
          setRepositoryPath(setup.repositoryPath)
          setProfiles(setup.profiles)
          setProviderSettings(setup.providerSettings as ProviderSettings)
          setSetupComplete(true)
        }}
        providerSettings={providerSettings}
      />
    )
  }

  return (
    <WorkspaceShell
      subtitle="Open a repository, review a change set, curate generated feedback, and publish through gh."
      title="Review Workspace"
    >
      <div className="space-y-5">
        <ProviderSettingsPanel
          onChange={setProviderSettings}
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
