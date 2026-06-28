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
import { Textarea } from "@/components/ui/textarea"
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
  const [selectedId, setSelectedId] = useState(feedback[0]?.id ?? "")
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

  const selectedFeedback =
    filteredFeedback.find((item) => item.id === selectedId) ??
    filteredFeedback[0] ??
    feedback[0]

  return (
    <section className="grid gap-4 lg:grid-cols-10">
      <div className="border border-border bg-card lg:col-span-3">
        <div className="border-b border-border p-4">
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Feedback
          </p>
          <h2 className="mt-1 text-lg font-semibold">
            {filteredFeedback.length} findings
          </h2>
          <div className="mt-4 space-y-2">
            <Input
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

        <div className="max-h-screen overflow-y-auto">
          {filteredFeedback.map((item) => (
            <button
              className="block w-full border-b border-border p-4 text-left hover:bg-muted"
              key={item.id}
              onClick={() => setSelectedId(item.id)}
              type="button"
            >
              <div className="flex items-center justify-between gap-3">
                <span className="text-xs font-medium uppercase text-muted-foreground">
                  {item.severity}
                </span>
                <span className="text-xs text-muted-foreground">
                  {item.state}
                </span>
              </div>
              <p className="mt-2 text-sm font-medium">{item.title}</p>
              <p className="mt-1 truncate font-mono text-xs text-muted-foreground">
                {item.file}
              </p>
            </button>
          ))}
        </div>
      </div>

      {selectedFeedback ? (
        <FeedbackDetail feedback={selectedFeedback} />
      ) : (
        <div className="border border-border bg-card p-6 lg:col-span-7">
          <p className="text-sm text-muted-foreground">No feedback selected.</p>
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
      <Select onValueChange={(nextValue) => onChange(nextValue as T)} value={value}>
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

type FeedbackDetailProps = {
  feedback: ReviewFeedbackItem
}

function FeedbackDetail({ feedback }: FeedbackDetailProps) {
  return (
    <div className="border border-border bg-card lg:col-span-7">
      <div className="border-b border-border p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="text-xs font-medium uppercase text-muted-foreground">
              {feedback.profileName} profile
            </p>
            <h2 className="mt-1 text-xl font-semibold">{feedback.title}</h2>
            <p className="mt-2 font-mono text-xs text-muted-foreground">
              {feedback.file}
              {feedback.line ? `:${feedback.line}` : ""}
            </p>
          </div>
          <div className="flex gap-2">
            <Badge variant="secondary">{feedback.severity}</Badge>
            <Badge variant="outline">{feedback.state}</Badge>
            {feedback.limitedContext ? (
              <Badge variant="secondary">limited context</Badge>
            ) : null}
          </div>
        </div>
      </div>

      <div className="grid gap-5 p-5 xl:grid-cols-2">
        <div className="space-y-4">
          <DetailBlock title="Generated feedback">
            <p>{feedback.body}</p>
          </DetailBlock>

          <DetailBlock title="Suggested action">
            <p>{feedback.suggestedAction}</p>
          </DetailBlock>

          <DetailBlock title="Code context">
            <div className="border border-border bg-muted p-3 font-mono text-xs">
              {feedback.quotedCode ?? "Summary feedback has no inline quote."}
            </div>
          </DetailBlock>
        </div>

        <div className="space-y-4">
          <DetailBlock title="Editable comment">
            <Textarea
              className="min-h-40 resize-none"
              defaultValue={feedback.editableComment}
            />
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

          <DetailBlock title="Limitations">
            <ul className="space-y-2">
              {feedback.limitations.map((item) => (
                <li className="text-sm" key={item}>
                  {item}
                </li>
              ))}
            </ul>
          </DetailBlock>

          <div className="flex flex-wrap gap-2">
            <Button size="sm">Accept</Button>
            <Button size="sm" variant="secondary">
              Mark edited
            </Button>
            <Button size="sm" variant="outline">
              Dismiss
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}

type DetailBlockProps = {
  title: string
  children: ReactNode
}

function DetailBlock({ title, children }: DetailBlockProps) {
  return (
    <div>
      <h3 className="text-xs font-medium uppercase text-muted-foreground">
        {title}
      </h3>
      <div className="mt-2 text-sm leading-6">{children}</div>
    </div>
  )
}
