import { useEffect, useMemo, useState, type ReactNode } from "react";
import { ArrowCounterClockwise, Trash } from "@phosphor-icons/react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import type {
	ReviewFeedbackItem,
	ReviewFeedbackState,
	ReviewSeverity,
} from "@/domain/workspace-view";
import type { GhCliStatus } from "@/adapters/tauri-local-review-api";

type FeedbackWorkspaceProps = {
	feedback: ReviewFeedbackItem[];
	ghStatus: GhCliStatus | null;
	isRunning?: boolean;
	repositoryPath: string;
	onFeedbackChange?: (feedback: ReviewFeedbackItem) => void;
	onPublishFeedback?: (feedback: ReviewFeedbackItem) => void;
	onDeleteFeedback?: (feedbackId: string) => void;
};

const stateOptions: Array<"all" | ReviewFeedbackState> = [
	"all",
	"draft",
	"accepted",
	"edited",
	"dismissed",
	"published",
];

const severityOptions: Array<"all" | ReviewSeverity> = [
	"all",
	"blocking",
	"important",
	"suggestion",
	"question",
	"nitpick",
];

export function FeedbackWorkspace({
	feedback,
	ghStatus,
	isRunning = false,
	repositoryPath,
	onFeedbackChange,
	onPublishFeedback,
	onDeleteFeedback,
}: FeedbackWorkspaceProps) {
	const [stateFilter, setStateFilter] =
		useState<(typeof stateOptions)[number]>("all");
	const [severityFilter, setSeverityFilter] =
		useState<(typeof severityOptions)[number]>("all");
	const [profileFilter, setProfileFilter] = useState("all");
	const [typeFilter, setTypeFilter] = useState<"all" | "inline" | "summary">(
		"all",
	);
	const [query, setQuery] = useState("");

	const profileOptions = useMemo(
		() => Array.from(new Set(feedback.map((item) => item.profileName))),
		[feedback],
	);

	const filteredFeedback = useMemo(() => {
		const normalizedQuery = query.trim().toLowerCase();

		return feedback.filter((item) => {
			const matchesState = stateFilter === "all" || item.state === stateFilter;
			const matchesSeverity =
				severityFilter === "all" || item.severity === severityFilter;
			const matchesProfile =
				profileFilter === "all" || item.profileName === profileFilter;
			const matchesType = typeFilter === "all" || item.type === typeFilter;
			const matchesQuery =
				normalizedQuery.length === 0 ||
				[item.title, item.file, item.body, item.profileName].some((value) =>
					value.toLowerCase().includes(normalizedQuery),
				);

			return (
				matchesState &&
				matchesSeverity &&
				matchesProfile &&
				matchesType &&
				matchesQuery
			);
		});
	}, [feedback, profileFilter, query, severityFilter, stateFilter, typeFilter]);

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
						<FeedbackCard
							feedback={item}
							ghStatus={ghStatus}
							index={index + 1}
							key={item.id}
							onFeedbackChange={onFeedbackChange}
							onDeleteFeedback={onDeleteFeedback}
							onPublishFeedback={onPublishFeedback}
							repositoryPath={repositoryPath}
						/>
					))}
				</div>
			) : (
				<div className="p-6">
					<p className="text-sm text-muted-foreground">
						{isRunning
							? "No feedback yet. Comments will appear here as soon as each review pass finds them."
							: "No feedback matches the current filters."}
					</p>
				</div>
			)}
		</section>
	);
}

type FilterSelectProps<T extends string> = {
	label: string;
	value: T;
	options: T[];
	onChange: (value: T) => void;
};

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
	);
}

type FeedbackCardProps = {
	feedback: ReviewFeedbackItem;
	ghStatus: GhCliStatus | null;
	index: number;
	repositoryPath: string;
	onFeedbackChange?: (feedback: ReviewFeedbackItem) => void;
	onPublishFeedback?: (feedback: ReviewFeedbackItem) => void;
	onDeleteFeedback?: (feedbackId: string) => void;
};

function FeedbackCard({
	feedback,
	ghStatus,
	index,
	repositoryPath,
	onFeedbackChange,
	onPublishFeedback,
	onDeleteFeedback,
}: FeedbackCardProps) {
	const [editableComment, setEditableComment] = useState(
		feedback.editableComment,
	);
	const originalComment = feedback.body;
	const hasEditedComment = editableComment !== originalComment;
	const canEdit = Boolean(onFeedbackChange) && feedback.state !== "published";
	const canPublish =
		Boolean(onPublishFeedback) &&
		Boolean(repositoryPath) &&
		feedback.state !== "published" &&
		feedback.type === "inline" &&
		Boolean(feedback.line || feedback.codeLocation) &&
		Boolean(ghStatus?.installed && ghStatus.authenticated);
	const publishTooltip =
		!ghStatus?.installed || !ghStatus.authenticated
			? "Enabled when gh is installed and authenticated."
			: feedback.type !== "inline" || !(feedback.line || feedback.codeLocation)
				? "Only inline feedback with a file and line can be published."
				: "Publish this inline comment to the current pull request.";
	const hasQuotedCode = Boolean(feedback.quotedCode?.trim());
	const limitations = feedback.limitations.filter(
		(item) => item.trim().length > 0,
	);

	useEffect(() => {
		setEditableComment(feedback.editableComment);
	}, [feedback.editableComment, feedback.id]);

	function persistEditableComment(nextComment: string) {
		if (nextComment === feedback.editableComment) return;

		onFeedbackChange?.({
			...feedback,
			editableComment: nextComment,
			state: nextFeedbackStateForComment(
				feedback,
				nextComment,
				originalComment,
			),
		});
	}

	function resetEditableComment() {
		setEditableComment(originalComment);
		persistEditableComment(originalComment);
	}

	function currentFeedback(): ReviewFeedbackItem {
		return {
			...feedback,
			editableComment,
			state: nextFeedbackStateForComment(feedback, editableComment, originalComment),
		};
	}

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
				<div className="grid gap-5 grid-cols-1">
					<DetailBlock className="col-span-1" title="Editable comment">
						<div className="flex items-start gap-2">
							<Textarea
								className="min-h-28 text-sm leading-6"
								disabled={!canEdit}
								onBlur={(event) =>
									persistEditableComment(event.currentTarget.value)
								}
								onChange={(event) => setEditableComment(event.target.value)}
								value={editableComment}
							/>
							<Button
								aria-label="Reset editable comment"
								disabled={!canEdit || !hasEditedComment}
								onClick={resetEditableComment}
								size="icon"
								title="Reset editable comment"
								type="button"
								variant="outline"
							>
								<ArrowCounterClockwise className="size-4" />
							</Button>
						</div>
					</DetailBlock>

					<DetailBlock title="Suggested action">
						<p>{feedback.suggestedAction}</p>
					</DetailBlock>

					<DetailBlock title="Evidence">
						<ul className="space-y-3">
							{feedback.evidence.map((item) => (
								<li key={item}>
									<pre className="overflow-x-auto border border-border bg-muted p-3 text-xs leading-5">
										<code>{item}</code>
									</pre>
								</li>
							))}
						</ul>
					</DetailBlock>

					{hasQuotedCode ? (
						<DetailBlock title="Code context">
							<pre className="overflow-x-auto border border-border bg-muted p-3 text-xs leading-5">
								<code>{feedback.quotedCode}</code>
							</pre>
						</DetailBlock>
					) : null}

					{limitations.length > 0 ? (
						<DetailBlock title="Limitations">
							<ul className="space-y-2">
								{limitations.map((item) => (
									<li className="text-sm" key={item}>
										{item}
									</li>
								))}
							</ul>
						</DetailBlock>
					) : null}
				</div>

				<div className="mt-5 flex flex-wrap justify-end gap-2">
					<TooltipProvider>
						<Tooltip>
							<TooltipTrigger asChild>
								<span>
									<Button
										disabled={!canPublish}
										onClick={() => onPublishFeedback?.(currentFeedback())}
										size="sm"
										type="button"
									>
										Accept
									</Button>
								</span>
							</TooltipTrigger>
							<TooltipContent>{publishTooltip}</TooltipContent>
						</Tooltip>
					</TooltipProvider>
					<Button
						aria-label="Delete feedback"
						disabled={!onDeleteFeedback}
						onClick={() => onDeleteFeedback?.(feedback.id)}
						size="icon-sm"
						title="Delete feedback"
						type="button"
						variant="destructive"
					>
						<Trash className="size-4" />
					</Button>
				</div>
			</div>
		</article>
	);
}

function nextFeedbackStateForComment(
	feedback: ReviewFeedbackItem,
	nextComment: string,
	originalComment: string,
): ReviewFeedbackState {
	if (feedback.state === "published") return feedback.state;
	if (nextComment !== originalComment) return "edited";
	if (feedback.state === "edited") return "draft";
	return feedback.state;
}

type DetailBlockProps = {
	title: string;
	children: ReactNode;
	className?: string;
};

function DetailBlock({ title, children, className }: DetailBlockProps) {
	return (
		<div className={className}>
			<h4 className="text-xs font-medium uppercase text-muted-foreground">
				{title}
			</h4>
			<div className="mt-2 text-sm leading-6">{children}</div>
		</div>
	);
}
