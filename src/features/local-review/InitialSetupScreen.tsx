import { useMemo, useState } from "react"
import { FolderOpen, Plus } from "@phosphor-icons/react"

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
  type ProviderSettings,
} from "@/domain"
import type {
  ReviewProfileItem,
  ReviewProfileScopeKind,
} from "@/data/localReviewMockData"

type InitialSetupScreenProps = {
  initialProfiles: ReviewProfileItem[]
  providerSettings: ProviderSettings
  onComplete: (setup: {
    repositoryPath: string
    profiles: ReviewProfileItem[]
    providerSettings: ProviderSettings
  }) => void
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

export function InitialSetupScreen({
  initialProfiles,
  providerSettings,
  onComplete,
}: InitialSetupScreenProps) {
  const [repositoryPath, setRepositoryPath] = useState("")
  const [profiles, setProfiles] = useState(initialProfiles)
  const [settings, setSettings] = useState(providerSettings)
  const [profileDraft, setProfileDraft] = useState<ProfileDraft>({
    name: "",
    prompt: "",
    scopeKind: "global",
  })
  const activeProfiles = profiles.filter((profile) => profile.selected)
  const selectedProvider = settings.modelProviders.find(
    (provider) => provider.enabled && provider.selectedModelId,
  )
  const canStart = repositoryPath.trim().length > 0 && Boolean(selectedProvider) && activeProfiles.length > 0

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
        label: "Profiles",
        done: activeProfiles.length > 0,
        detail: `${activeProfiles.length} active`,
      },
    ],
    [activeProfiles.length, repositoryPath, selectedProvider],
  )

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
              <div className="grid gap-3 md:grid-cols-2">
                {settings.modelProviders.map((provider) => (
                  <div className="border border-border p-3" key={provider.id}>
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="font-medium">{provider.name}</p>
                        <p className="mt-1 text-xs text-muted-foreground">
                          Pre-filled local endpoint
                        </p>
                      </div>
                      <Switch
                        checked={provider.enabled}
                        onCheckedChange={(enabled) =>
                          setSettings((current) =>
                            enabled
                              ? selectSingleModelProvider(current, provider.id)
                              : updateModelProviderSettings(
                                  current,
                                  provider.id,
                                  (currentProvider) => ({
                                    ...currentProvider,
                                    enabled: false,
                                    selectedModelId: undefined,
                                    useForHumanToneRewrite: false,
                                  }),
                                ),
                          )
                        }
                      />
                    </div>
                    <div className="mt-3 space-y-2">
                      <Label htmlFor={`${provider.id}-setup-url`}>Base URL</Label>
                      <Input
                        id={`${provider.id}-setup-url`}
                        onChange={(event) =>
                          updateProviderBaseUrl(provider.id, event.target.value)
                        }
                        value={provider.baseUrl}
                      />
                    </div>
                    <div className="mt-3 space-y-2">
                      <Label>Quick model</Label>
                      <Select
                        onValueChange={(modelId) =>
                          selectProvider(provider.id, modelId)
                        }
                        value={provider.selectedModelId ?? ""}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select model" />
                        </SelectTrigger>
                        <SelectContent>
                          {quickModels[provider.id as keyof typeof quickModels].map(
                            (modelId) => (
                              <SelectItem key={modelId} value={modelId}>
                                {modelId}
                              </SelectItem>
                            ),
                          )}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
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
          <Button
            disabled={!canStart}
            onClick={() =>
              onComplete({
                repositoryPath: repositoryPath.trim(),
                profiles,
                providerSettings: settings,
              })
            }
          >
            Start review workspace
          </Button>
        </div>
      </section>
    </main>
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
