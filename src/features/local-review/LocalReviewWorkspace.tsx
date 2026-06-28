import { useMemo, useState } from "react"
import { WorkspaceShell } from "@/components/layout/WorkspaceShell"
import { localReviewMockSession } from "@/data/localReviewMockData"

import { ExecutionStatus } from "./ExecutionStatus"
import { FeedbackWorkspace } from "./FeedbackWorkspace"
import { ProfileManager } from "./ProfileManager"
import { PublicationSummary } from "./PublicationSummary"
import { SetupOverview } from "./SetupOverview"

export function LocalReviewWorkspace() {
  const [profiles, setProfiles] = useState(localReviewMockSession.profiles)
  const session = useMemo(
    () => ({
      ...localReviewMockSession,
      profiles,
    }),
    [profiles],
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

  return (
    <WorkspaceShell
      subtitle="Open a repository, review a change set, curate generated feedback, and publish through gh."
      title="Review Workspace"
    >
      <div className="space-y-5">
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
