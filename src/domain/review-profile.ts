import type { ChangedFile } from "./change-set"

export type ProfileScope =
  | { readonly kind: "global" }
  | { readonly kind: "repository"; readonly repositoryPath: string }
  | {
      readonly kind: "folder"
      readonly repositoryPath: string
      readonly folderPath: string
    }

export interface ReviewRule {
  readonly id: string
  readonly title: string
  readonly body: string
}

export interface ReviewProfile {
  readonly id: string
  readonly name: string
  readonly scope: ProfileScope
  readonly criteria: readonly string[]
  readonly rules: readonly ReviewRule[]
  readonly prompt: string
  readonly enabledByDefault?: boolean
  readonly fileGlobs?: readonly string[]
}

export interface SuggestedProfile {
  readonly profile: ReviewProfile
  readonly selected: boolean
  readonly reason: "manual" | "folder_scope" | "repository_scope" | "global_default" | "global_available"
  readonly priority: number
}

export function profileAppliesToFile(
  profile: ReviewProfile,
  repositoryPath: string,
  file: Pick<ChangedFile, "path">,
): boolean {
  if (!profileAppliesToRepository(profile, repositoryPath)) return false
  if (!profile.fileGlobs?.length) return true

  return profile.fileGlobs.some((glob) => pathMatchesSimpleGlob(file.path, glob))
}

export function suggestReviewProfiles(
  profiles: readonly ReviewProfile[],
  repositoryPath: string,
  changedFiles: readonly Pick<ChangedFile, "path">[],
  manuallySelectedProfileIds: readonly string[] = [],
): readonly SuggestedProfile[] {
  const manualIds = new Set(manuallySelectedProfileIds)

  return profiles
    .filter((profile) =>
      changedFiles.some((file) => profileAppliesToFile(profile, repositoryPath, file)),
    )
    .map((profile) => {
      const manual = manualIds.has(profile.id)
      const scopedReason = profileSuggestionReason(profile, repositoryPath)

      return {
        profile,
        selected: manual || scopedReason !== "global_available",
        reason: manual ? "manual" : scopedReason,
        priority: manual ? 0 : profileSuggestionPriority(scopedReason),
      }
    })
    .sort((left, right) => left.priority - right.priority || left.profile.name.localeCompare(right.profile.name))
}

function profileAppliesToRepository(
  profile: ReviewProfile,
  repositoryPath: string,
): boolean {
  if (profile.scope.kind === "global") return true
  return normalizePath(profile.scope.repositoryPath) === normalizePath(repositoryPath)
}

function profileSuggestionReason(
  profile: ReviewProfile,
  repositoryPath: string,
): SuggestedProfile["reason"] {
  if (profile.scope.kind === "folder") return "folder_scope"
  if (profile.scope.kind === "repository") return "repository_scope"
  if (profile.enabledByDefault && profileAppliesToRepository(profile, repositoryPath)) {
    return "global_default"
  }

  return "global_available"
}

function profileSuggestionPriority(reason: SuggestedProfile["reason"]): number {
  switch (reason) {
    case "manual":
      return 0
    case "folder_scope":
      return 1
    case "repository_scope":
      return 2
    case "global_default":
      return 3
    case "global_available":
      return 4
  }
}

function pathMatchesSimpleGlob(path: string, glob: string): boolean {
  if (glob === "*") return true
  if (glob.endsWith("/**")) return path.startsWith(glob.slice(0, -3))
  if (glob.startsWith("**/*.")) return path.endsWith(glob.slice(4))
  if (glob.startsWith("*.")) return path.endsWith(glob.slice(1))
  return path === glob
}

function normalizePath(path: string): string {
  return path.replace(/\/+$/, "")
}
