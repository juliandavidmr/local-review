import { useState } from "react"

import {
  checkProviderConnection,
  listProviderModels,
  saveProviderSettings,
} from "@/adapters/tauri-local-review-api"
import {
  selectSingleModelProvider,
  type ModelProviderSettings,
  type ProviderConnectionStatus,
  type ProviderSettings,
} from "@/domain"
import type { ModelDescriptor } from "@/ports"

type ProbeOptions = {
  persistSettings?: boolean
}

type RefreshOptions = {
  selectFirstModel?: boolean
}

export function useProviderModelProbe(
  settings: ProviderSettings,
  onSettingsChange: (settings: ProviderSettings) => void,
  options: ProbeOptions = {},
) {
  const [modelsByProvider, setModelsByProvider] = useState<
    Record<string, readonly ModelDescriptor[]>
  >({})
  const [statuses, setStatuses] = useState<Record<string, ProviderConnectionStatus>>(
    {},
  )
  const [loadingProviderId, setLoadingProviderId] = useState<string | null>(null)

  async function refreshProvider(
    provider: ModelProviderSettings,
    refreshOptions: RefreshOptions = {},
  ) {
    setLoadingProviderId(provider.id)

    try {
      const status = await checkProviderConnection(provider)
      const models = status.ok ? await listProviderModels(provider) : []

      setStatuses((current) => ({
        ...current,
        [provider.id]: status,
      }))
      setModelsByProvider((current) => ({
        ...current,
        [provider.id]: models,
      }))

      if (
        status.ok &&
        refreshOptions.selectFirstModel !== false &&
        !provider.selectedModelId &&
        models[0]
      ) {
        const nextSettings = selectSingleModelProvider(
          settings,
          provider.id,
          models[0].modelId,
        )
        onSettingsChange(
          options.persistSettings
            ? await saveProviderSettings(nextSettings)
            : nextSettings,
        )
      }

      return { models, status }
    } catch (error) {
      const status = {
        providerId: provider.id,
        ok: false,
        message:
          error instanceof Error
            ? error.message
            : "Provider connection could not be checked.",
      }

      setStatuses((current) => ({
        ...current,
        [provider.id]: status,
      }))
      setModelsByProvider((current) => ({
        ...current,
        [provider.id]: [],
      }))

      return { models: [], status }
    } finally {
      setLoadingProviderId((current) => (current === provider.id ? null : current))
    }
  }

  return {
    loadingProviderId,
    modelsByProvider,
    refreshProvider,
    statuses,
  }
}
