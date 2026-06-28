import { Button } from "@/components/ui/button"
import type { ReviewSessionMock } from "@/data/localReviewMockData"

type PublicationSummaryProps = {
  publication: ReviewSessionMock["publication"]
}

export function PublicationSummary({ publication }: PublicationSummaryProps) {
  return (
    <section className="border border-border bg-card p-4">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Publication summary
          </p>
          <h2 className="mt-1 text-lg font-semibold">
            Ready for {publication.target}
          </h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Batch publication includes accepted and edited feedback only.
          </p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="outline">
            Human-tone rewrite
          </Button>
          <Button size="sm">Publish batch</Button>
        </div>
      </div>

      <div className="mt-4 grid gap-3 text-sm sm:grid-cols-5">
        <SummaryMetric label="Total" value={publication.totalComments} />
        <SummaryMetric label="Inline" value={publication.inlineComments} />
        <SummaryMetric label="Summary" value={publication.summaryComments} />
        <SummaryMetric
          label="Limited context"
          value={publication.limitedContextCount}
        />
        <SummaryMetric
          label="Incomplete"
          value={publication.incompleteSession ? "Yes" : "No"}
        />
      </div>

      {publication.incompleteSession ? (
        <div className="mt-4 border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
          This session has incomplete coverage. Individual publication can
          proceed with care, and batch publication should require explicit
          acknowledgement.
        </div>
      ) : null}
    </section>
  )
}

type SummaryMetricProps = {
  label: string
  value: number | string
}

function SummaryMetric({ label, value }: SummaryMetricProps) {
  return (
    <div className="border border-border p-3">
      <p className="text-lg font-semibold">{value}</p>
      <p className="text-xs text-muted-foreground">{label}</p>
    </div>
  )
}
