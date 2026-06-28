import { ArrowsClockwise, Plugs } from "@phosphor-icons/react"

import { saveProviderSettings } from "@/adapters/tauri-local-review-api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import {
  selectSingleModelProvider,
  updateMcpSourceSettings,
  updateModelProviderSettings,
  type ModelProviderSettings,
  type ProviderSettings,
} from "@/domain"

import { useProviderModelProbe } from "./useProviderModelProbe"

type ProviderSettingsPanelProps = {
  settings: ProviderSettings
  onChange: (settings: ProviderSettings) => void
}

export function ProviderSettingsPanel({
  settings,
  onChange,
}: ProviderSettingsPanelProps) {
  const { loadingProviderId, modelsByProvider, refreshProvider, statuses } =
    useProviderModelProbe(settings, onChange, { persistSettings: true })
  const selectedModelProvider = settings.modelProviders.find(
    (provider) => provider.enabled,
  )
  const enabledMcpSources = settings.mcpSources.filter((source) => source.enabled).length

  function updateModelProvider(
    providerId: string,
    patch: Partial<ModelProviderSettings>,
  ) {
    const nextSettings = updateModelProviderSettings(
      settings,
      providerId,
      (provider) => ({
        ...provider,
        ...patch,
      }),
    )
    onChange(nextSettings)
    void saveProviderSettings(nextSettings)
  }

  return (
    <section className="border border-border bg-card">
      <div className="flex flex-col gap-4 border-b border-border p-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Providers
          </p>
          <h2 className="mt-1 text-lg font-semibold">
            {selectedModelProvider ? selectedModelProvider.name : "No provider selected"}
          </h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Configure local model providers, MCP sources, and execution capacity.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="secondary">{enabledMcpSources} MCP sources</Badge>
          <Badge variant="outline">
            {settings.execution.maxParallelReviewPasses} parallel passes
          </Badge>
        </div>
      </div>

      <div className="grid gap-4 bg-muted/40 p-4 xl:grid-cols-2">
        {settings.modelProviders.map((provider) => {
          const status = statuses[provider.id]
          const models = modelsByProvider[provider.id] ?? []
          const isLoading = loadingProviderId === provider.id
          const isLmStudio = provider.kind === "lm_studio"
          const modelOptions =
            provider.selectedModelId &&
            !models.some((model) => model.modelId === provider.selectedModelId)
              ? [
                  {
                    available: true,
                    displayName: provider.selectedModelId,
                    modelId: provider.selectedModelId,
                    providerId: provider.id,
                  },
                  ...models,
                ]
              : models

          return (
            <article className="border border-border bg-card p-4" key={provider.id}>
              <div className="flex items-start justify-between gap-3">
                <div>
                  <h3 className="font-semibold">{provider.name}</h3>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {provider.kind === "ollama" ? "Ollama local API" : "OpenAI-compatible LM Studio API"}
                  </p>
                </div>
                <Switch
                  checked={provider.enabled}
                  onCheckedChange={(enabled) =>
                    void saveProviderSettings(
                      enabled
                        ? selectSingleModelProvider(settings, provider.id)
                        : updateModelProviderSettings(
                            settings,
                            provider.id,
                            (currentProvider) => ({
                              ...currentProvider,
                              enabled: false,
                              selectedModelId: undefined,
                              useForHumanToneRewrite: false,
                            }),
                          ),
                    ).then(onChange)
                  }
                />
              </div>

              <div className="mt-4 grid gap-3 md:grid-cols-2">
                <div className="space-y-2 md:col-span-2">
                  <Label htmlFor={`${provider.id}-base-url`}>Base URL</Label>
                  <Input
                    id={`${provider.id}-base-url`}
                    onChange={(event) =>
                      updateModelProvider(provider.id, {
                        baseUrl: event.target.value,
                      })
                    }
                    value={provider.baseUrl}
                  />
                </div>
                <div className="space-y-2">
                  <Label>Model</Label>
                  <Select
                    disabled={modelOptions.length === 0 || isLoading}
                    onValueChange={(selectedModelId) =>
                      void saveProviderSettings(
                        selectSingleModelProvider(
                          settings,
                          provider.id,
                          selectedModelId,
                        ),
                      ).then(onChange)
                    }
                    value={provider.selectedModelId ?? ""}
                  >
                    <SelectTrigger>
                      <SelectValue
                        placeholder={
                          isLoading ? "Loading models" : "No model loaded"
                        }
                      />
                    </SelectTrigger>
                    <SelectContent>
                      {modelOptions.map((model) => (
                        <SelectItem key={model.modelId} value={model.modelId}>
                          {model.displayName}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-end">
                  <Button
                    className="w-full"
                    onClick={() => refreshProvider(provider)}
                    disabled={isLoading}
                    size="sm"
                    variant="outline"
                  >
                    <ArrowsClockwise className="size-4" />
                    {isLoading
                      ? "Checking..."
                      : isLmStudio
                        ? "Test LM Studio"
                        : "Test and load models"}
                  </Button>
                </div>
              </div>

              <div className="mt-4 flex items-center justify-between gap-3 border-t border-border pt-3 text-sm">
                <span>Use for human-tone rewrite</span>
                <Switch
                  checked={Boolean(provider.useForHumanToneRewrite)}
                  onCheckedChange={(useForHumanToneRewrite) =>
                    updateModelProvider(provider.id, {
                      useForHumanToneRewrite,
                    })
                  }
                />
              </div>

              {status ? (
                <p
                  className={
                    status.ok
                      ? "mt-3 text-xs text-muted-foreground"
                      : "mt-3 text-xs text-destructive"
                  }
                >
                  {status.ok ? "Connected" : "Unavailable"}: {status.message}
                </p>
              ) : null}
            </article>
          )
        })}
      </div>

      <div className="grid gap-4 border-t border-border p-4 xl:grid-cols-2">
        <section>
          <div className="mb-3 flex items-center gap-2">
            <Plugs className="size-4" />
            <h3 className="text-sm font-semibold">MCP sources</h3>
          </div>
          <div className="space-y-2">
            {settings.mcpSources.map((source) => (
              <label
                className="flex items-start justify-between gap-3 border border-border p-3 text-sm"
                key={source.id}
              >
                <span>
                  <span className="block font-medium">{source.name}</span>
                  {source.description ? (
                    <span className="mt-1 block text-xs text-muted-foreground">
                      {source.description}
                    </span>
                  ) : null}
                </span>
                <Switch
                  checked={source.enabled}
                  onCheckedChange={(enabled) => {
                    const nextSettings = updateMcpSourceSettings(
                      settings,
                      source.id,
                      (current) => ({
                        ...current,
                        enabled,
                      }),
                    )
                    onChange(nextSettings)
                    void saveProviderSettings(nextSettings)
                  }}
                />
              </label>
            ))}
          </div>
        </section>

        <section>
          <h3 className="text-sm font-semibold">Execution capacity</h3>
          <div className="mt-3 space-y-4 border border-border p-3">
            <div className="space-y-2">
              <Label htmlFor="parallel-passes">Max parallel review passes</Label>
              <Input
                id="parallel-passes"
                min={1}
                onChange={(event) => {
                  const nextSettings = {
                    ...settings,
                    execution: {
                      ...settings.execution,
                      maxParallelReviewPasses: Math.max(
                        1,
                        Number(event.target.value) || 1,
                      ),
                    },
                  }
                  onChange(nextSettings)
                  void saveProviderSettings(nextSettings)
                }}
                type="number"
                value={settings.execution.maxParallelReviewPasses}
              />
            </div>
            <label className="flex items-center justify-between gap-3 text-sm">
              <span>Adaptive parallelism</span>
              <Switch
                checked={settings.execution.adaptiveParallelismEnabled}
                onCheckedChange={(adaptiveParallelismEnabled) => {
                  const nextSettings = {
                    ...settings,
                    execution: {
                      ...settings.execution,
                      adaptiveParallelismEnabled,
                    },
                  }
                  onChange(nextSettings)
                  void saveProviderSettings(nextSettings)
                }}
              />
            </label>
          </div>
        </section>
      </div>
    </section>
  )
}
