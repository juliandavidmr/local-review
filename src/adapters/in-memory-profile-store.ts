import {
  suggestReviewProfiles,
  type ChangeSetSnapshot,
  type ReviewProfile,
  type SuggestedProfile,
} from "../domain"
import type { ProfileStore, SuggestProfilesInput } from "../ports"

export class InMemoryProfileStore implements ProfileStore {
  private readonly profiles = new Map<string, ReviewProfile>()

  constructor(initialProfiles: readonly ReviewProfile[] = []) {
    for (const profile of initialProfiles) {
      this.profiles.set(profile.id, profile)
    }
  }

  async listProfiles(): Promise<readonly ReviewProfile[]> {
    return Array.from(this.profiles.values()).sort((left, right) =>
      left.name.localeCompare(right.name),
    )
  }

  async getProfile(profileId: string): Promise<ReviewProfile | undefined> {
    return this.profiles.get(profileId)
  }

  async saveProfile(profile: ReviewProfile): Promise<void> {
    this.profiles.set(profile.id, profile)
  }

  async deleteProfile(profileId: string): Promise<void> {
    this.profiles.delete(profileId)
  }

  async suggestProfiles(input: SuggestProfilesInput): Promise<readonly SuggestedProfile[]> {
    return suggestProfilesFromChangeSet(
      Array.from(this.profiles.values()),
      input.repositoryPath,
      input.changeSet,
      input.manuallySelectedProfileIds,
    )
  }
}

function suggestProfilesFromChangeSet(
  profiles: readonly ReviewProfile[],
  repositoryPath: string,
  changeSet: ChangeSetSnapshot,
  manuallySelectedProfileIds: readonly string[] = [],
): readonly SuggestedProfile[] {
  return suggestReviewProfiles(
    profiles,
    repositoryPath,
    changeSet.files,
    manuallySelectedProfileIds,
  )
}
