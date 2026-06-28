import { WorkspaceShell } from "@/components/layout/WorkspaceShell"
import { localReviewMockSession } from "@/data/localReviewMockData"

import { ExecutionStatus } from "./ExecutionStatus"
import { FeedbackWorkspace } from "./FeedbackWorkspace"
import { PublicationSummary } from "./PublicationSummary"
import { SetupOverview } from "./SetupOverview"

export function LocalReviewWorkspace() {
  return (
    <WorkspaceShell
      subtitle="Open a repository, review a change set, curate generated feedback, and publish through gh."
      title="Review Workspace"
    >
      <div className="space-y-5">
        <SetupOverview session={localReviewMockSession} />
        <ExecutionStatus execution={localReviewMockSession.execution} />
        <FeedbackWorkspace feedback={localReviewMockSession.feedback} />
        <PublicationSummary publication={localReviewMockSession.publication} />
      </div>
    </WorkspaceShell>
  )
}
