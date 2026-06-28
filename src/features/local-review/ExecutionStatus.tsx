import { Progress } from "@/components/ui/progress"
import type { ReviewSessionMock } from "@/data/localReviewMockData"

type ExecutionStatusProps = {
  execution: ReviewSessionMock["execution"]
}

export function ExecutionStatus({ execution }: ExecutionStatusProps) {
  const completedPercent = Math.round(
    (execution.completedPasses / execution.totalPasses) * 100,
  )

  return (
    <section className="border border-border bg-card p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Execution status
          </p>
          <h2 className="mt-1 text-lg font-semibold capitalize">
            {execution.status}
          </h2>
        </div>
        <div className="text-right text-sm">
          <p className="font-medium">
            {execution.completedPasses} of {execution.totalPasses} passes
          </p>
          <p className="text-muted-foreground">{completedPercent}% complete</p>
        </div>
      </div>

      <Progress className="mt-4" value={completedPercent} />

      <div className="mt-4 grid gap-3 text-sm sm:grid-cols-4">
        <Metric label="Changed files" value={execution.changedFiles} />
        <Metric label="Modified lines" value={execution.modifiedLines} />
        <Metric
          label="Exploration requests"
          value={execution.explorationRequests}
        />
        <Metric label="Guardrail hits" value={execution.guardrailHits} />
      </div>
    </section>
  )
}

type MetricProps = {
  label: string
  value: number
}

function Metric({ label, value }: MetricProps) {
  return (
    <div className="border border-border p-3">
      <p className="text-xl font-semibold">{value}</p>
      <p className="text-xs text-muted-foreground">{label}</p>
    </div>
  )
}
