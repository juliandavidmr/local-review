export type ReviewIntent = "prepare_own_changes" | "review_someone_elses_changes"

export type ChangeSourceType = "pull_request" | "working_tree" | "commit" | "compare_refs"

export type ChangeSource =
  | {
      readonly type: "pull_request"
      readonly provider: "github"
      readonly owner: string
      readonly repository: string
      readonly number: number
    }
  | {
      readonly type: "working_tree"
      readonly repositoryPath: string
    }
  | {
      readonly type: "commit"
      readonly repositoryPath: string
      readonly commitSha: string
    }
  | {
      readonly type: "compare_refs"
      readonly repositoryPath: string
      readonly baseRef: string
      readonly headRef: string
    }

export type ChangedFileStatus =
  | "added"
  | "modified"
  | "deleted"
  | "renamed"
  | "copied"

export type ChangeLineKind = "added" | "removed" | "context"

export interface ChangeLine {
  readonly kind: ChangeLineKind
  readonly content: string
  readonly oldLineNumber?: number
  readonly newLineNumber?: number
}

export interface ChangeHunk {
  readonly id: string
  readonly oldStartLine: number
  readonly newStartLine: number
  readonly lines: readonly ChangeLine[]
}

export interface ChangedFile {
  readonly path: string
  readonly previousPath?: string
  readonly status: ChangedFileStatus
  readonly additions: number
  readonly deletions: number
  readonly hunks: readonly ChangeHunk[]
  readonly isGenerated?: boolean
}

export interface ChangeSetSnapshot {
  readonly id: string
  readonly repositoryPath: string
  readonly source: ChangeSource
  readonly baseRef?: string
  readonly headRef?: string
  readonly files: readonly ChangedFile[]
  readonly createdAt: string
  readonly fingerprint: string
}

export type CodeLocationSide = "old" | "new"

export interface CodeLocation {
  readonly filePath: string
  readonly startLine: number
  readonly endLine: number
  readonly side: CodeLocationSide
}

export function countModifiedLines(changeSet: ChangeSetSnapshot): number {
  return changeSet.files.reduce(
    (total, file) => total + file.additions + file.deletions,
    0,
  )
}

export function countFileModifiedLines(file: ChangedFile): number {
  return file.additions + file.deletions
}

export function isLargeChangeSet(changeSet: ChangeSetSnapshot): boolean {
  return changeSet.files.length > 10 || countModifiedLines(changeSet) > 800
}

export function findChangedFile(
  changeSet: ChangeSetSnapshot,
  filePath: string,
): ChangedFile | undefined {
  return changeSet.files.find(
    (file) => file.path === filePath || file.previousPath === filePath,
  )
}

export function locationLineMatches(
  line: ChangeLine,
  lineNumber: number,
  side: CodeLocationSide,
): boolean {
  return side === "new"
    ? line.newLineNumber === lineNumber
    : line.oldLineNumber === lineNumber
}

export function isLocationInChangeSet(
  changeSet: ChangeSetSnapshot,
  location: CodeLocation,
): boolean {
  if (location.startLine > location.endLine) return false

  const file = findChangedFile(changeSet, location.filePath)
  if (!file) return false

  for (let lineNumber = location.startLine; lineNumber <= location.endLine; lineNumber += 1) {
    const hasLine = file.hunks.some((hunk) =>
      hunk.lines.some((line) =>
        locationLineMatches(line, lineNumber, location.side),
      ),
    )

    if (!hasLine) return false
  }

  return true
}

export function getLocationText(
  changeSet: ChangeSetSnapshot,
  location: CodeLocation,
): string | undefined {
  const file = findChangedFile(changeSet, location.filePath)
  if (!file || !isLocationInChangeSet(changeSet, location)) return undefined

  const lines: string[] = []
  for (let lineNumber = location.startLine; lineNumber <= location.endLine; lineNumber += 1) {
    const line = file.hunks
      .flatMap((hunk) => hunk.lines)
      .find((candidate) =>
        locationLineMatches(candidate, lineNumber, location.side),
      )

    if (!line) return undefined
    lines.push(line.content)
  }

  return lines.join("\n")
}
