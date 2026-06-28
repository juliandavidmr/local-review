import {
  defaultProviderSettings,
  type ProviderSettings,
} from "../domain"
import type { ProviderSettingsStore } from "../ports"

export class InMemoryProviderSettingsStore implements ProviderSettingsStore {
  private settings: ProviderSettings

  constructor(initialSettings: ProviderSettings = defaultProviderSettings) {
    this.settings = initialSettings
  }

  async loadSettings(): Promise<ProviderSettings> {
    return this.settings
  }

  async saveSettings(settings: ProviderSettings): Promise<void> {
    this.settings = settings
  }
}
