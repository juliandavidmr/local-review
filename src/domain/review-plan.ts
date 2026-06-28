import {
  countFileModifiedLines,
  isLargeChangeSet,
  type ChangeSetSnapshot,
  type ChangedFile,
} from "./change-set"
import type { RepositoryExplorationBudget } from "./exploration"
import {
  profileAppliesToFile,
  type ReviewProfile,
} from "./review-profile"

export interface ReviewBudget {
  readonly maxModifiedLinesPerPass: number
  readonly contextTokensPerPass: number
  readonly explorationBudget: RepositoryExplorationBudget
}

export type ReviewSegment =
  | {
      readonly id: string
      readonly kind: "file"
      readonly filePath: string
      readonly hunkIds: readonly string[]
    }
  | {
      readonly id: string
      readonly kind: "hunk_partition"
      readonly filePath: string
      readonly hunkIds: readonly string[]
      readonly partitionIndex: number
    }

export type PlannedReviewPassKind = "session_overview" | "profile_specific"

export interface PlannedReviewPass {
  readonly id: string
  readonly kind: PlannedReviewPassKind
  readonly segmentId?: string
  readonly profileId?: string
  readonly contextBudgetTokens: number
  readonly explorationBudget: RepositoryExplorationBudget
}

export interface ReviewPlan {
  readonly changeSetId: string
  readonly isLargeChangeSet: boolean
  readonly segments: readonly ReviewSegment[]
  readonly passes: readonly PlannedReviewPass[]
  readonly coverageFilePaths: readonly string[]
}

export function planReviewPasses(
  changeSet: ChangeSetSnapshot,
  profiles: readonly ReviewProfile[],
  budget: ReviewBudget,
): ReviewPlan {
  const segments = changeSet.files.flatMap((file) =>
    createSegmentsForFile(file, budget.maxModifiedLinesPerPass),
  )

  const overviewPasses: PlannedReviewPass[] = isLargeChangeSet(changeSet)
    ? [
        {
          id: `${changeSet.id}:overview`,
          kind: "session_overview",
          contextBudgetTokens: budget.contextTokensPerPass,
          explorationBudget: budget.explorationBudget,
        },
      ]
    : []

  const profilePasses = segments.flatMap((segment) => {
    const file = changeSet.files.find((candidate) => candidate.path === segment.filePath)
    if (!file) return []

    return profiles
      .filter((profile) =>
        profileAppliesToFile(profile, changeSet.repositoryPath, file),
      )
      .map((profile): PlannedReviewPass => ({
        id: `${segment.id}:profile:${profile.id}`,
        kind: "profile_specific",
        segmentId: segment.id,
        profileId: profile.id,
        contextBudgetTokens: budget.contextTokensPerPass,
        explorationBudget: budget.explorationBudget,
      }))
  })

  return {
    changeSetId: changeSet.id,
    isLargeChangeSet: isLargeChangeSet(changeSet),
    segments,
    passes: [...overviewPasses, ...profilePasses],
    coverageFilePaths: changeSet.files.map((file) => file.path),
  }
}

function createSegmentsForFile(
  file: ChangedFile,
  maxModifiedLinesPerPass: number,
): readonly ReviewSegment[] {
  const hunkIds = file.hunks.map((hunk) => hunk.id)
  if (countFileModifiedLines(file) <= maxModifiedLinesPerPass || file.hunks.length <= 1) {
    return [
      {
        id: `file:${file.path}`,
        kind: "file",
        filePath: file.path,
        hunkIds,
      },
    ]
  }

  return partitionHunkIds(file, maxModifiedLinesPerPass).map(
    (partition, index): ReviewSegment => ({
      id: `file:${file.path}:partition:${index + 1}`,
      kind: "hunk_partition",
      filePath: file.path,
      hunkIds: partition,
      partitionIndex: index + 1,
    }),
  )
}

function partitionHunkIds(
  file: ChangedFile,
  maxModifiedLinesPerPass: number,
): readonly (readonly string[])[] {
  const partitions: string[][] = []
  let currentPartition: string[] = []
  let currentModifiedLines = 0

  for (const hunk of file.hunks) {
    const hunkModifiedLines = hunk.lines.filter((line) => line.kind !== "context").length
    const wouldOverflow =
      currentPartition.length > 0 &&
      currentModifiedLines + hunkModifiedLines > maxModifiedLinesPerPass

    if (wouldOverflow) {
      partitions.push(currentPartition)
      currentPartition = []
      currentModifiedLines = 0
    }

    currentPartition.push(hunk.id)
    currentModifiedLines += hunkModifiedLines
  }

  if (currentPartition.length > 0) partitions.push(currentPartition)
  return partitions
}
