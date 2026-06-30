import { CircleNotch, GitBranch, Sparkle } from "@phosphor-icons/react";
import type React from "react";
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
					<div className="flex items-start gap-3">
						<CircleNotch className="mt-0.5 size-5 animate-spin text-foreground" />
						<div className="min-w-0 flex-1">
							<p className="font-medium">Review is running.</p>
							<p className="mt-1 text-muted-foreground">
								Feedback appears as soon as each pass returns usable comments.
							</p>
							<div className="mt-3 grid gap-2">
								<ActivityRow
									icon={<Sparkle className="size-4" />}
									label="Phase"
									value={execution.currentPhase ?? "Waiting for model response"}
								/>
								{execution.currentFile ? (
									<ActivityRow
										icon={<GitBranch className="size-4" />}
										label="File"
										value={execution.currentFile}
									/>
								) : null}
								{execution.currentProfile ? (
									<ActivityRow
										label="Profile"
										value={execution.currentProfile}
									/>
								) : null}
							</div>
						</div>
					</div>
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

type ActivityRowProps = {
	icon?: React.ReactNode;
	label: string;
	value: string;
};

function ActivityRow({ icon, label, value }: ActivityRowProps) {
	return (
		<div className="flex min-w-0 items-center gap-2 text-xs">
			<span className="flex size-4 shrink-0 items-center justify-center text-muted-foreground">
				{icon}
			</span>
			<span className="shrink-0 font-medium text-muted-foreground">{label}</span>
			<span className="truncate text-foreground" title={value}>
				{value}
			</span>
		</div>
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
