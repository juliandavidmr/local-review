import type { ProfileScope, ReviewProfile, ReviewRule } from "../domain"
import type { ProfileStore } from "../ports"

export interface ReviewProfileDraft {
  readonly name: string
  readonly scope: ProfileScope
  readonly criteria: readonly string[]
  readonly rules: readonly ReviewRule[]
  readonly prompt: string
  readonly enabledByDefault?: boolean
  readonly fileGlobs?: readonly string[]
}

export async function createReviewProfile(
  store: ProfileStore,
  draft: ReviewProfileDraft,
): Promise<ReviewProfile> {
  const profile: ReviewProfile = {
    ...draft,
    id: createProfileId(draft.name),
  }

  await store.saveProfile(profile)
  return profile
}

export async function updateReviewProfile(
  store: ProfileStore,
  profile: ReviewProfile,
): Promise<ReviewProfile> {
  await store.saveProfile(profile)
  return profile
}

export async function deleteReviewProfile(
  store: ProfileStore,
  profileId: string,
): Promise<void> {
  await store.deleteProfile(profileId)
}

export function setReviewProfileDefault(
  profile: ReviewProfile,
  enabledByDefault: boolean,
): ReviewProfile {
  return {
    ...profile,
    enabledByDefault,
  }
}

function createProfileId(name: string): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")

  return `profile:${slug || "untitled"}:${Date.now()}`
}
