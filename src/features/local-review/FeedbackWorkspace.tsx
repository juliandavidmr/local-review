import { useMemo, useState, type ReactNode } from "react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import type {
  ReviewFeedbackItem,
  ReviewFeedbackState,
  ReviewSeverity,
} from "@/data/localReviewMockData"

type FeedbackWorkspaceProps = {
  feedback: ReviewFeedbackItem[]
}

const stateOptions: Array<"all" | ReviewFeedbackState> = [
  "all",
  "draft",
  "accepted",
  "edited",
  "dismissed",
  "published",
]

const severityOptions: Array<"all" | ReviewSeverity> = [
  "all",
  "blocking",
  "important",
  "suggestion",
  "question",
  "nitpick",
]

export function FeedbackWorkspace({ feedback }: FeedbackWorkspaceProps) {
  const [stateFilter, setStateFilter] =
    useState<(typeof stateOptions)[number]>("all")
  const [severityFilter, setSeverityFilter] =
    useState<(typeof severityOptions)[number]>("all")
  const [profileFilter, setProfileFilter] = useState("all")
  const [typeFilter, setTypeFilter] = useState<"all" | "inline" | "summary">(
    "all",
  )
  const [query, setQuery] = useState("")

  const profileOptions = useMemo(
    () => Array.from(new Set(feedback.map((item) => item.profileName))),
    [feedback],
  )

  const filteredFeedback = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase()

    return feedback.filter((item) => {
      const matchesState = stateFilter === "all" || item.state === stateFilter
      const matchesSeverity =
        severityFilter === "all" || item.severity === severityFilter
      const matchesProfile =
        profileFilter === "all" || item.profileName === profileFilter
      const matchesType = typeFilter === "all" || item.type === typeFilter
      const matchesQuery =
        normalizedQuery.length === 0 ||
        [item.title, item.file, item.body, item.profileName].some((value) =>
          value.toLowerCase().includes(normalizedQuery),
        )

      return (
        matchesState &&
        matchesSeverity &&
        matchesProfile &&
        matchesType &&
        matchesQuery
      )
    })
  }, [feedback, profileFilter, query, severityFilter, stateFilter, typeFilter])

  return (
    <section className="border border-border bg-card">
      <div className="flex flex-col gap-4 border-b border-border p-4 xl:flex-row xl:items-start xl:justify-between">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Feedback
          </p>
          <h2 className="mt-1 text-lg font-semibold">
            {filteredFeedback.length} findings
          </h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Full feedback comments shown one after another.
          </p>
        </div>

        <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-5">
          <Input
            className="md:col-span-2 xl:col-span-1"
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Filter feedback"
            value={query}
          />
          <FilterSelect
            label="State"
            onChange={setStateFilter}
            options={stateOptions}
            value={stateFilter}
          />
          <FilterSelect
            label="Severity"
            onChange={setSeverityFilter}
            options={severityOptions}
            value={severityFilter}
          />
          <FilterSelect
            label="Profile"
            onChange={setProfileFilter}
            options={["all", ...profileOptions]}
            value={profileFilter}
          />
          <FilterSelect
            label="Type"
            onChange={setTypeFilter}
            options={["all", "inline", "summary"]}
            value={typeFilter}
          />
        </div>
      </div>

      {filteredFeedback.length > 0 ? (
        <div className="space-y-4 bg-muted/40 p-4">
          {filteredFeedback.map((item, index) => (
            <FeedbackCard feedback={item} index={index + 1} key={item.id} />
          ))}
        </div>
      ) : (
        <div className="p-6">
          <p className="text-sm text-muted-foreground">
            No feedback matches the current filters.
          </p>
        </div>
      )}
    </section>
  )
}

type FilterSelectProps<T extends string> = {
  label: string
  value: T
  options: T[]
  onChange: (value: T) => void
}

function FilterSelect<T extends string>({
  label,
  value,
  options,
  onChange,
}: FilterSelectProps<T>) {
  return (
    <div className="space-y-1">
      <p className="text-xs font-medium text-muted-foreground">{label}</p>
      <Select
        onValueChange={(nextValue) => onChange(nextValue as T)}
        value={value}
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option} value={option}>
              {option}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}

type FeedbackCardProps = {
  feedback: ReviewFeedbackItem
  index: number
}

function FeedbackCard({ feedback, index }: FeedbackCardProps) {
  return (
    <article className="border border-border bg-card shadow-sm">
      <div className="border-b border-border bg-muted/30 p-5">
        <div className="mb-3 flex items-center gap-3">
          <span className="border border-border bg-background px-2 py-1 text-xs font-medium text-muted-foreground">
            Feedback #{index}
          </span>
          <span className="text-xs font-medium uppercase text-muted-foreground">
            {feedback.profileName} profile
          </span>
        </div>
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="min-w-0">
            <h3 className="text-xl font-semibold">{feedback.title}</h3>
            <p className="mt-2 break-all font-mono text-xs text-muted-foreground">
              {feedback.file}
              {feedback.line ? `:${feedback.line}` : ""}
            </p>
          </div>
          <div className="flex shrink-0 flex-wrap gap-2">
            <Badge variant="secondary">{feedback.severity}</Badge>
            <Badge variant="outline">{feedback.state}</Badge>
            <Badge variant="outline">{feedback.type}</Badge>
            {feedback.limitedContext ? (
              <Badge variant="secondary">limited context</Badge>
            ) : null}
          </div>
        </div>
      </div>

      <div className="p-5">
        <div className="grid gap-5 xl:grid-cols-2">
          <DetailBlock title="Generated feedback">
            <p>{feedback.body}</p>
          </DetailBlock>

          <DetailBlock title="Editable comment">
            <div className="min-h-24 border border-input bg-background p-3 text-sm leading-6">
              {feedback.editableComment}
            </div>
          </DetailBlock>

          <DetailBlock title="Suggested action">
            <p>{feedback.suggestedAction}</p>
          </DetailBlock>

          <DetailBlock title="Evidence">
            <ul className="space-y-2">
              {feedback.evidence.map((item) => (
                <li className="text-sm" key={item}>
                  {item}
                </li>
              ))}
            </ul>
          </DetailBlock>

          <DetailBlock title="Code context">
            <div className="overflow-x-auto border border-border bg-muted p-3 font-mono text-xs">
              {feedback.quotedCode ?? "Summary feedback has no inline quote."}
            </div>
          </DetailBlock>

          <DetailBlock title="Limitations">
            <ul className="space-y-2">
              {feedback.limitations.map((item) => (
                <li className="text-sm" key={item}>
                  {item}
                </li>
              ))}
            </ul>
          </DetailBlock>
        </div>

        <div className="mt-5 flex flex-wrap justify-end gap-2">
          <Button size="sm">Accept</Button>
          <Button size="sm" variant="secondary">
            Mark edited
          </Button>
          <Button size="sm" variant="outline">
            Dismiss
          </Button>
        </div>
      </div>
    </article>
  )
}

type DetailBlockProps = {
  title: string
  children: ReactNode
}

function DetailBlock({ title, children }: DetailBlockProps) {
  return (
    <div>
      <h4 className="text-xs font-medium uppercase text-muted-foreground">
        {title}
      </h4>
      <div className="mt-2 text-sm leading-6">{children}</div>
    </div>
  )
}
