import type { ReviewSession } from "../domain"
import type { ReviewHistoryStore } from "../ports"

export class InMemoryReviewHistoryStore implements ReviewHistoryStore {
  private readonly sessions = new Map<string, ReviewSession>()

  constructor(initialSessions: readonly ReviewSession[] = []) {
    for (const session of initialSessions) {
      this.sessions.set(session.id, session)
    }
  }

  async saveSession(session: ReviewSession): Promise<void> {
    this.sessions.set(session.id, session)
  }

  async getSession(sessionId: string): Promise<ReviewSession | undefined> {
    return this.sessions.get(sessionId)
  }

  async listSessions(repositoryPath?: string): Promise<readonly ReviewSession[]> {
    const sessions = Array.from(this.sessions.values())
    const filtered = repositoryPath
      ? sessions.filter((session) => session.repositoryPath === repositoryPath)
      : sessions

    return filtered.sort((left, right) => right.updatedAt.localeCompare(left.updatedAt))
  }
}
