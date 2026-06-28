import { useMemo, useState } from "react"
import { Plus, Trash } from "@phosphor-icons/react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
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
import type {
  ReviewProfileItem,
  ReviewProfileScopeKind,
} from "@/data/localReviewMockData"

type ProfileManagerProps = {
  profiles: ReviewProfileItem[]
  repositoryPath: string
  onCreateProfile: (profile: ReviewProfileItem) => void
  onDeleteProfile: (profileId: string) => void
  onToggleDefault: (profileId: string, enabledByDefault: boolean) => void
  onToggleSelected: (profileId: string, selected: boolean) => void
}

type ProfileDraft = {
  name: string
  scopeKind: ReviewProfileScopeKind
  criteria: string
  fileGlobs: string
  prompt: string
  enabledByDefault: boolean
  selected: boolean
}

const emptyDraft: ProfileDraft = {
  name: "",
  scopeKind: "global",
  criteria: "",
  fileGlobs: "",
  prompt: "",
  enabledByDefault: false,
  selected: true,
}

export function ProfileManager({
  profiles,
  repositoryPath,
  onCreateProfile,
  onDeleteProfile,
  onToggleDefault,
  onToggleSelected,
}: ProfileManagerProps) {
  const [open, setOpen] = useState(false)
  const [draft, setDraft] = useState<ProfileDraft>(emptyDraft)
  const activeCount = profiles.filter((profile) => profile.selected).length
  const defaultCount = profiles.filter((profile) => profile.enabledByDefault).length

  const canCreate = draft.name.trim().length > 0 && draft.prompt.trim().length > 0
  const sortedProfiles = useMemo(
    () =>
      [...profiles].sort((left, right) => {
        if (left.selected !== right.selected) return left.selected ? -1 : 1
        return left.name.localeCompare(right.name)
      }),
    [profiles],
  )

  function submitProfile() {
    if (!canCreate) return

    onCreateProfile({
      id: createMockProfileId(draft.name),
      name: draft.name.trim(),
      scope: scopeLabel(draft.scopeKind),
      scopeKind: draft.scopeKind,
      selected: draft.selected,
      enabledByDefault: draft.enabledByDefault,
      criteria: splitLines(draft.criteria),
      fileGlobs: splitLines(draft.fileGlobs),
      prompt: draft.prompt.trim(),
    })
    setDraft(emptyDraft)
    setOpen(false)
  }

  return (
    <section className="border border-border bg-card">
      <div className="flex flex-col gap-4 border-b border-border p-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Review profiles
          </p>
          <h2 className="mt-1 text-lg font-semibold">
            {activeCount} active profiles
          </h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Profiles are application-owned and scoped outside reviewed repositories.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="outline">{profiles.length} total</Badge>
          <Badge variant="secondary">{defaultCount} default</Badge>
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
              <Button size="sm">
                <Plus className="size-4" />
                Create profile
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-2xl">
              <DialogHeader>
                <DialogTitle>Create review profile</DialogTitle>
                <DialogDescription>
                  The profile is stored by the application and scoped to where it should apply.
                </DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="profile-name">Name</Label>
                  <Input
                    id="profile-name"
                    onChange={(event) =>
                      setDraft((current) => ({ ...current, name: event.target.value }))
                    }
                    value={draft.name}
                  />
                </div>
                <div className="space-y-2">
                  <Label>Scope</Label>
                  <Select
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        scopeKind: value as ReviewProfileScopeKind,
                      }))
                    }
                    value={draft.scopeKind}
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
                </div>
                <div className="space-y-2">
                  <Label htmlFor="profile-criteria">Criteria</Label>
                  <Textarea
                    id="profile-criteria"
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        criteria: event.target.value,
                      }))
                    }
                    placeholder="One criterion per line"
                    value={draft.criteria}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="profile-globs">File globs</Label>
                  <Textarea
                    id="profile-globs"
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        fileGlobs: event.target.value,
                      }))
                    }
                    placeholder="src/**"
                    value={draft.fileGlobs}
                  />
                </div>
                <div className="space-y-2 md:col-span-2">
                  <Label htmlFor="profile-prompt">Prompt</Label>
                  <Textarea
                    className="min-h-28"
                    id="profile-prompt"
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        prompt: event.target.value,
                      }))
                    }
                    value={draft.prompt}
                  />
                </div>
                <ToggleRow
                  checked={draft.selected}
                  label="Activate for this session"
                  onCheckedChange={(selected) =>
                    setDraft((current) => ({ ...current, selected }))
                  }
                />
                <ToggleRow
                  checked={draft.enabledByDefault}
                  label="Global default when applicable"
                  onCheckedChange={(enabledByDefault) =>
                    setDraft((current) => ({ ...current, enabledByDefault }))
                  }
                />
              </div>
              <DialogFooter>
                <Button disabled={!canCreate} onClick={submitProfile}>
                  Create profile
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      <div className="grid gap-3 bg-muted/40 p-4 xl:grid-cols-3">
        {sortedProfiles.map((profile) => (
          <article className="border border-border bg-card p-4" key={profile.id}>
            <div className="flex items-start justify-between gap-3">
              <div>
                <h3 className="font-semibold">{profile.name}</h3>
                <p className="mt-1 text-xs text-muted-foreground">
                  {profile.scope}
                </p>
              </div>
              <Button
                aria-label={`Delete ${profile.name}`}
                onClick={() => onDeleteProfile(profile.id)}
                size="icon-sm"
                variant="ghost"
              >
                <Trash className="size-4" />
              </Button>
            </div>

            <div className="mt-3 flex flex-wrap gap-2">
              <Badge variant={profile.selected ? "secondary" : "outline"}>
                {profile.selected ? "active" : "inactive"}
              </Badge>
              {profile.enabledByDefault ? (
                <Badge variant="outline">default</Badge>
              ) : null}
              <Badge variant="outline">{profile.scopeKind}</Badge>
            </div>

            <p className="mt-3 line-clamp-3 text-sm text-muted-foreground">
              {profile.prompt}
            </p>

            <div className="mt-4 space-y-3 border-t border-border pt-3">
              <ToggleRow
                checked={profile.selected}
                label="Active in session"
                onCheckedChange={(selected) =>
                  onToggleSelected(profile.id, selected)
                }
              />
              <ToggleRow
                checked={profile.enabledByDefault}
                label="Default suggestion"
                onCheckedChange={(enabledByDefault) =>
                  onToggleDefault(profile.id, enabledByDefault)
                }
              />
            </div>
          </article>
        ))}
      </div>

      <div className="border-t border-border px-4 py-3 text-xs text-muted-foreground">
        Repository scope target: <span className="font-mono">{repositoryPath}</span>
      </div>
    </section>
  )
}

type ToggleRowProps = {
  checked: boolean
  label: string
  onCheckedChange: (checked: boolean) => void
}

function ToggleRow({ checked, label, onCheckedChange }: ToggleRowProps) {
  return (
    <label className="flex items-center justify-between gap-3 text-sm">
      <span>{label}</span>
      <Switch checked={checked} onCheckedChange={onCheckedChange} />
    </label>
  )
}

function splitLines(value: string): string[] {
  return value
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
}

function createMockProfileId(name: string): string {
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
