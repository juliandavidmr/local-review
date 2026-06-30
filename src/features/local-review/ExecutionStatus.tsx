import { Progress } from "@/components/ui/progress";
import type { ReviewWorkspaceView } from "@/domain/workspace-view";

type ExecutionStatusProps = {
	execution: ReviewWorkspaceView["execution"];
};

export function ExecutionStatus({ execution }: ExecutionStatusProps) {
	const processedPasses = execution.completedPasses + execution.guardrailHits;
	const completedPercent =
		execution.totalPasses > 0
			? Math.round((processedPasses / execution.totalPasses) * 100)
			: 0;
	const hasNoChangedFiles =
		execution.status === "completed" &&
		execution.changedFiles === 0 &&
		execution.modifiedLines === 0;
	const hasNoPasses =
		execution.status === "completed" && execution.totalPasses === 0;
	const isRunning = execution.status === "running";
	const isCancelled = execution.status === "cancelled";

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
						{processedPasses} of {execution.totalPasses} passes processed
					</p>
					{execution.guardrailHits > 0 ? (
						<p className="text-destructive">
							{execution.guardrailHits} pass failures
						</p>
					) : null}
					<p className="text-muted-foreground">{completedPercent}% complete</p>
				</div>
			</div>

			<Progress className="mt-4" value={completedPercent} />

			{isRunning ? (
				<div className="mt-4 border border-border bg-muted/40 p-3 text-sm">
					<p className="font-medium">Review is running.</p>
					<p className="mt-1 text-muted-foreground">
						The workspace is open while model passes run. Feedback appears as
						soon as each pass returns usable comments.
					</p>
				</div>
			) : isCancelled ? (
				<div className="mt-4 border border-border bg-muted/40 p-3 text-sm">
					<p className="font-medium">Review was stopped.</p>
					<p className="mt-1 text-muted-foreground">
						No more review passes will be started for this session.
					</p>
				</div>
			) : hasNoChangedFiles ? (
				<div className="mt-4 border border-border bg-muted/40 p-3 text-sm">
					<p className="font-medium">No changes were available to review.</p>
					<p className="mt-1 text-muted-foreground">
						The selected repository working tree produced 0 changed files, so no
						model passes were started. Make a local change, select a repository
						with pending changes, or start a new review from the header.
					</p>
				</div>
			) : hasNoPasses ? (
				<div className="mt-4 border border-border bg-muted/40 p-3 text-sm">
					<p className="font-medium">No review passes were scheduled.</p>
					<p className="mt-1 text-muted-foreground">
						Check that at least one active profile applies to the selected
						change set.
					</p>
				</div>
			) : null}

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
	);
}

type MetricProps = {
	label: string;
	value: number;
};

function Metric({ label, value }: MetricProps) {
	return (
		<div className="border border-border p-3">
			<p className="text-xl font-semibold">{value}</p>
			<p className="text-xs text-muted-foreground">{label}</p>
		</div>
	);
}
