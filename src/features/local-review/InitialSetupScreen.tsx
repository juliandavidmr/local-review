import { useEffect, useMemo, useRef, useState } from "react"
import { ArrowsClockwise, FolderOpen, Plus } from "@phosphor-icons/react"

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
import { Textarea } from "@/components/ui/textarea"
import {
  selectSingleModelProvider,
  updateModelProviderSettings,
  type ModelProviderSettings,
  type ProviderSettings,
} from "@/domain"
import type { ReviewChangeSourceKind } from "@/adapters/tauri-local-review-api"
import type {
  ReviewProfileItem,
  ReviewProfileScopeKind,
} from "@/domain/workspace-view"

import { useProviderModelProbe } from "./useProviderModelProbe"

type InitialSetupScreenProps = {
  error?: string | null
  initialProfiles: ReviewProfileItem[]
  isRunning?: boolean
  providerSettings: ProviderSettings
  onComplete: (setup: {
    repositoryPath: string
    reviewSourceKind: ReviewChangeSourceKind
    profiles: ReviewProfileItem[]
    providerSettings: ProviderSettings
  }) => void | Promise<void>
}

type ProfileDraft = {
  name: string
  prompt: string
  scopeKind: ReviewProfileScopeKind
}

const quickModels = {
  ollama: ["llama3.1", "qwen2.5-coder", "deepseek-coder"],
  "lm-studio": ["local-model", "qwen2.5-coder-instruct", "openai/gpt-oss-20b"],
}

const reviewSourceOptions: Array<{
  value: ReviewChangeSourceKind
  label: string
  description: string
}> = [
  {
    value: "current_branch",
    label: "Current branch",
    description: "Diff the current branch against its upstream or main base.",
  },
  {
    value: "staged_changes",
    label: "Staged changes",
    description: "Review only changes already staged with git add.",
  },
  {
    value: "unstaged_changes",
    label: "Unstaged changes",
    description: "Review local working tree changes that are not staged.",
  },
]

export function InitialSetupScreen({
  error,
  initialProfiles,
  isRunning = false,
  providerSettings,
  onComplete,
}: InitialSetupScreenProps) {
  const [repositoryPath, setRepositoryPath] = useState("")
  const [profiles, setProfiles] = useState(initialProfiles)
  const [settings, setSettings] = useState(() =>
    selectSingleModelProvider(providerSettings, "lm-studio"),
  )
  const [reviewSourceKind, setReviewSourceKind] =
    useState<ReviewChangeSourceKind>("current_branch")
  const [profileDraft, setProfileDraft] = useState<ProfileDraft>({
    name: "",
    prompt: "",
    scopeKind: "global",
  })
  const { loadingProviderId, modelsByProvider, refreshProvider, statuses } =
    useProviderModelProbe(settings, setSettings)
  const autoTestedLmStudio = useRef(false)
  const activeProfiles = profiles.filter((profile) => profile.selected)
  const selectedProvider = settings.modelProviders.find(
    (provider) => provider.enabled && provider.selectedModelId,
  )
  const activeProvider = settings.modelProviders.find((provider) => provider.enabled)
  const activeProviderId = activeProvider?.id ?? "lm-studio"
  const canStart =
    repositoryPath.trim().length > 0 &&
    Boolean(selectedProvider) &&
    activeProfiles.length > 0 &&
    !isRunning

  const setupItems = useMemo(
    () => [
      {
        label: "Repository",
        done: repositoryPath.trim().length > 0,
        detail: repositoryPath || "No folder selected",
      },
      {
        label: "Model provider",
        done: Boolean(selectedProvider),
        detail: selectedProvider
          ? `${selectedProvider.name} / ${selectedProvider.selectedModelId}`
          : "Choose a provider and model",
      },
      {
        label: "Review source",
        done: true,
        detail:
          reviewSourceOptions.find((option) => option.value === reviewSourceKind)
            ?.label ?? "Unstaged changes",
      },
      {
        label: "Profiles",
        done: activeProfiles.length > 0,
        detail: `${activeProfiles.length} active`,
      },
    ],
    [activeProfiles.length, repositoryPath, reviewSourceKind, selectedProvider],
  )

  useEffect(() => {
    if (autoTestedLmStudio.current) return

    const lmStudio = settings.modelProviders.find(
      (provider) => provider.id === "lm-studio" && provider.enabled,
    )
    if (!lmStudio) return

    autoTestedLmStudio.current = true
    void refreshProvider(lmStudio)
  }, [])

  async function chooseRepositoryFolder() {
    try {
      const dialog = await import("@tauri-apps/plugin-dialog")
      const selected = await dialog.open({
        directory: true,
        multiple: false,
        title: "Select Git repository",
      })

      if (typeof selected === "string") {
        setRepositoryPath(selected)
      }
    } catch {
      // Browser preview fallback keeps the manual path input usable.
    }
  }

  function selectProvider(providerId: string, selectedModelId: string) {
    setSettings((current) =>
      selectSingleModelProvider(current, providerId, selectedModelId),
    )
  }

  function selectProviderType(providerId: string) {
    setSettings((current) => selectSingleModelProvider(current, providerId))
  }

  function updateProviderBaseUrl(providerId: string, baseUrl: string) {
    setSettings((current) =>
      updateModelProviderSettings(current, providerId, (provider) => ({
        ...provider,
        baseUrl,
      })),
    )
  }

  function createProfile() {
    if (!profileDraft.name.trim() || !profileDraft.prompt.trim()) return

    setProfiles((current) => [
      {
        id: createProfileId(profileDraft.name),
        name: profileDraft.name.trim(),
        scope: scopeLabel(profileDraft.scopeKind),
        scopeKind: profileDraft.scopeKind,
        selected: true,
        enabledByDefault: profileDraft.scopeKind === "global",
        criteria: [profileDraft.name.trim()],
        fileGlobs: ["*"],
        prompt: profileDraft.prompt.trim(),
      },
      ...current,
    ])
    setProfileDraft({
      name: "",
      prompt: "",
      scopeKind: "global",
    })
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-muted/40 p-6">
      <section className="w-full max-w-5xl border border-border bg-card shadow-sm">
        <div className="border-b border-border p-6">
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Local Review setup
          </p>
          <h1 className="mt-2 text-2xl font-semibold">Start a review session</h1>
          <p className="mt-2 max-w-2xl text-sm text-muted-foreground">
            Select a local repository, choose a local model provider, and activate review profiles before generating feedback.
          </p>
        </div>

        <div className="grid gap-6 p-6 xl:grid-cols-3">
          <div className="space-y-3">
            {setupItems.map((item) => (
              <div className="border border-border bg-background p-3" key={item.label}>
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-medium">{item.label}</p>
                  <span className={item.done ? "text-xs text-foreground" : "text-xs text-muted-foreground"}>
                    {item.done ? "Ready" : "Missing"}
                  </span>
                </div>
                <p className="mt-1 break-all text-xs text-muted-foreground">
                  {item.detail}
                </p>
              </div>
            ))}
          </div>

          <div className="space-y-5 xl:col-span-2">
            <SetupBlock title="Repository">
              <div className="grid gap-2 md:grid-cols-3">
                <Input
                  className="md:col-span-2"
                  onChange={(event) => setRepositoryPath(event.target.value)}
                  placeholder="/Users/name/project"
                  value={repositoryPath}
                />
                <Button onClick={chooseRepositoryFolder} variant="outline">
                  <FolderOpen className="size-4" />
                  Select folder
                </Button>
              </div>
            </SetupBlock>

            <SetupBlock title="Provider and model">
              <div className="space-y-3">
                <div className="space-y-2">
                  <Label>Provider type</Label>
                  <Select onValueChange={selectProviderType} value={activeProviderId}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {settings.modelProviders.map((provider) => (
                        <SelectItem key={provider.id} value={provider.id}>
                          {provider.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {activeProvider ? (
                  <ProviderSetupCard
                    isLoading={loadingProviderId === activeProvider.id}
                    models={modelsByProvider[activeProvider.id] ?? []}
                    onBaseUrlChange={updateProviderBaseUrl}
                    onModelSelect={selectProvider}
                    onRefresh={refreshProvider}
                    provider={activeProvider}
                    status={statuses[activeProvider.id]}
                  />
                ) : null}
              </div>
            </SetupBlock>

            <SetupBlock title="Review source">
              <div className="grid gap-3 md:grid-cols-3">
                {reviewSourceOptions.map((option) => (
                  <button
                    className={
                      reviewSourceKind === option.value
                        ? "border border-foreground bg-background p-3 text-left"
                        : "border border-border bg-background p-3 text-left hover:bg-muted"
                    }
                    key={option.value}
                    onClick={() => setReviewSourceKind(option.value)}
                    type="button"
                  >
                    <span className="block text-sm font-medium">{option.label}</span>
                    <span className="mt-1 block text-xs text-muted-foreground">
                      {option.description}
                    </span>
                  </button>
                ))}
              </div>
            </SetupBlock>

            <SetupBlock title="Review profiles">
              <div className="grid gap-3 md:grid-cols-2">
                <div className="space-y-3">
                  {profiles.map((profile) => (
                    <label
                      className="flex items-start justify-between gap-3 border border-border p-3"
                      key={profile.id}
                    >
                      <span>
                        <span className="block text-sm font-medium">
                          {profile.name}
                        </span>
                        <span className="mt-1 block text-xs text-muted-foreground">
                          {profile.scope}
                        </span>
                      </span>
                      <Switch
                        checked={profile.selected}
                        onCheckedChange={(selected) =>
                          setProfiles((current) =>
                            current.map((candidate) =>
                              candidate.id === profile.id
                                ? { ...candidate, selected }
                                : candidate,
                            ),
                          )
                        }
                      />
                    </label>
                  ))}
                </div>
                <div className="space-y-3 border border-border p-3">
                  <p className="text-sm font-medium">Create manual profile</p>
                  <Input
                    onChange={(event) =>
                      setProfileDraft((current) => ({
                        ...current,
                        name: event.target.value,
                      }))
                    }
                    placeholder="Security"
                    value={profileDraft.name}
                  />
                  <Select
                    onValueChange={(scopeKind) =>
                      setProfileDraft((current) => ({
                        ...current,
                        scopeKind: scopeKind as ReviewProfileScopeKind,
                      }))
                    }
                    value={profileDraft.scopeKind}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="global">Global</SelectItem>
                      <SelectItem value="repository">Repository path</SelectItem>
                      <SelectItem value="folder">Folder path</SelectItem>
                    </SelectContent>
                  </Select>
                  <Textarea
                    className="min-h-24"
                    onChange={(event) =>
                      setProfileDraft((current) => ({
                        ...current,
                        prompt: event.target.value,
                      }))
                    }
                    placeholder="Review for..."
                    value={profileDraft.prompt}
                  />
                  <Button className="w-full" onClick={createProfile} variant="outline">
                    <Plus className="size-4" />
                    Add profile
                  </Button>
                </div>
              </div>
            </SetupBlock>
          </div>
        </div>

        <div className="flex items-center justify-end gap-3 border-t border-border p-6">
          {error ? (
            <p className="mr-auto max-w-xl text-sm text-destructive">{error}</p>
          ) : null}
          <Button
            disabled={!canStart}
            onClick={() =>
              onComplete({
                repositoryPath: repositoryPath.trim(),
                reviewSourceKind,
                profiles,
                providerSettings: settings,
              })
            }
          >
            {isRunning ? "Running review..." : "Start review workspace"}
          </Button>
        </div>
      </section>
    </main>
  )
}

type ProviderSetupCardProps = {
  provider: ModelProviderSettings
  models: readonly { displayName: string; modelId: string }[]
  status?: { ok: boolean; message: string }
  isLoading: boolean
  onBaseUrlChange: (providerId: string, baseUrl: string) => void
  onModelSelect: (providerId: string, selectedModelId: string) => void
  onRefresh: (provider: ModelProviderSettings) => void
}

function ProviderSetupCard({
  provider,
  models,
  status,
  isLoading,
  onBaseUrlChange,
  onModelSelect,
  onRefresh,
}: ProviderSetupCardProps) {
  const presetModels = quickModels[provider.id as keyof typeof quickModels] ?? []
  const modelOptions = [
    ...models.map((model) => ({
      label: model.displayName,
      value: model.modelId,
    })),
    ...presetModels
      .filter(
        (modelId) => !models.some((model) => model.modelId === modelId),
      )
      .map((modelId) => ({ label: modelId, value: modelId })),
  ]
  const isLmStudio = provider.kind === "lm_studio"

  return (
    <div className="border border-border p-3">
      <div>
        <p className="font-medium">{provider.name}</p>
        <p className="mt-1 text-xs text-muted-foreground">
          {isLmStudio
            ? "LM Studio local OpenAI-compatible server"
            : "Ollama local API endpoint"}
        </p>
      </div>
      <div className="mt-3 space-y-2">
        <Label htmlFor={`${provider.id}-setup-url`}>Base URL</Label>
        <Input
          id={`${provider.id}-setup-url`}
          onChange={(event) =>
            onBaseUrlChange(provider.id, event.target.value)
          }
          value={provider.baseUrl}
        />
      </div>
      <div className="mt-3 flex flex-col gap-2 md:flex-row md:items-end">
        <div className="w-full space-y-2">
          <Label>Model</Label>
          <Select
            onValueChange={(modelId) => onModelSelect(provider.id, modelId)}
            value={provider.selectedModelId ?? ""}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select model" />
            </SelectTrigger>
            <SelectContent>
              {modelOptions.map((model) => (
                <SelectItem key={model.value} value={model.value}>
                  {model.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex items-end">
          <Button
            className="w-full md:w-auto"
            disabled={isLoading}
            onClick={() => onRefresh(provider)}
            size="sm"
            variant="outline"
          >
            <ArrowsClockwise className="size-4" />
            {isLoading ? "Checking..." : isLmStudio ? "Test LM Studio" : "Load models"}
          </Button>
        </div>
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
    </div>
  )
}

type SetupBlockProps = {
  title: string
  children: React.ReactNode
}

function SetupBlock({ title, children }: SetupBlockProps) {
  return (
    <section>
      <h2 className="mb-2 text-sm font-semibold">{title}</h2>
      {children}
    </section>
  )
}

function createProfileId(name: string): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")

  return `${slug || "profile"}-${Date.now()}`
}

function scopeLabel(scopeKind: ReviewProfileScopeKind): string {
  switch (scopeKind) {
    case "global":
      return "Global"
    case "repository":
      return "Repository path"
    case "folder":
      return "Folder path"
  }
}
