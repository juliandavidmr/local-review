import { useEffect, useRef, useState } from "react";
import { ArrowCounterClockwise, StopCircle } from "@phosphor-icons/react";
import {
	buildChangeSet,
	cancelReviewSession,
	checkGhCliStatus,
	deleteReviewFeedback,
	type ChangeSetSnapshot,
	type GhCliStatus,
	loadLatestReviewSession,
	loadProfiles,
	loadProviderSettings,
	listenReviewProgress,
	openRepository,
	publishReviewFeedback,
	runReviewSession,
	saveReviewSession,
	saveProviderSettings,
	saveProfile,
	updateReviewFeedback,
	type RepositoryDescriptor,
	type ReviewWorkspaceSession,
} from "@/adapters/tauri-local-review-api";
import { WorkspaceShell } from "@/components/layout/WorkspaceShell";
import { Button } from "@/components/ui/button";
import { defaultProviderSettings, type ProviderSettings } from "@/domain";
import type {
	ReviewFeedbackItem,
	ReviewProfileItem,
	ReviewWorkspaceView,
} from "@/domain/workspace-view";

import { ExecutionStatus } from "./ExecutionStatus";
import { FeedbackWorkspace } from "./FeedbackWorkspace";
import { InitialSetupScreen } from "./InitialSetupScreen";
import { SetupOverview } from "./SetupOverview";

export function LocalReviewWorkspace() {
	const [session, setSession] = useState<ReviewWorkspaceSession | null>(null);
	const [profiles, setProfiles] = useState<ReviewProfileItem[]>([]);
	const [providerSettings, setProviderSettings] = useState<ProviderSettings>(
		defaultProviderSettings,
	);
	const [ghStatus, setGhStatus] = useState<GhCliStatus | null>(null);
	const [loading, setLoading] = useState(true);
	const [running, setRunning] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const activeReviewId = useRef<string | null>(null);

	useEffect(() => {
		async function loadInitialState() {
			try {
				const [storedProfiles, storedSettings, latestSession] =
					await Promise.all([
						loadProfiles(),
						loadProviderSettings(),
						loadLatestReviewSession(),
					]);
				setProfiles(storedProfiles);
				setProviderSettings(storedSettings);
				setSession(latestSession);
				void refreshGhStatus();
			} catch (unknownError) {
				setError(errorMessage(unknownError));
			} finally {
				setLoading(false);
			}
		}

		void loadInitialState();
	}, []);

	if (loading) {
		return (
			<main className="flex min-h-screen items-center justify-center bg-muted/40 p-6">
				<div className="border border-border bg-card p-6 text-sm text-muted-foreground">
					Loading local review configuration...
				</div>
			</main>
		);
	}

	if (!session) {
		return (
			<InitialSetupScreen
				error={error}
				ghStatus={ghStatus}
				initialProfiles={profiles}
				isRunning={running}
				onComplete={async (setup) => {
					setRunning(true);
					setError(null);
					let startedReviewId: string | null = null;
					let unlistenProgress: (() => void) | null = null;
					try {
						const repository = await openRepository(setup.repositoryPath);
						const changeSet = await buildChangeSet(
							repository.path,
							setup.reviewSourceKind,
						);

						if (changeSet.files.length === 0) {
							throw new Error(
								"The selected review source produced 0 changed files. Choose Current branch for committed branch changes, Staged changes for git add changes, or Unstaged changes for local working tree edits.",
							);
						}

						const savedSettings = await saveProviderSettings(
							setup.providerSettings,
						);
						const profilesWithRepositoryScope = setup.profiles.map((profile) =>
							profile.scopeKind === "repository"
								? { ...profile, scope: repository.path }
								: profile,
						);

						for (const profile of profilesWithRepositoryScope) {
							await saveProfile(profile);
						}

						const reviewId = createReviewId();
						startedReviewId = reviewId;
						activeReviewId.current = reviewId;
						setProfiles(profilesWithRepositoryScope);
						setProviderSettings(savedSettings);
						const runningSession = createRunningSession({
							changeSet,
							profiles: profilesWithRepositoryScope,
							providerSettings: savedSettings,
							repository,
						});
						setSession(runningSession);
						await saveReviewSession(runningSession);
						unlistenProgress = await listenReviewProgress((progress) => {
							if (activeReviewId.current !== progress.reviewId) return;

							setSession((current) => {
								if (!current) return current;

								const nextSession = applyReviewProgress(
									current,
									progress.execution,
									progress.feedback,
								);
								void saveReviewSession(nextSession);
								return nextSession;
							});
						});

						const nextSession = await runReviewSession({
							reviewId,
							repository,
							changeSet,
							profiles: profilesWithRepositoryScope,
							providerSettings: savedSettings,
						});

						if (activeReviewId.current === reviewId) {
							setSession((current) => {
								const mergedSession = current
									? preserveCuratedFeedback(current, nextSession)
									: nextSession;
								void saveReviewSession(mergedSession);
								return mergedSession;
							});
						}
					} catch (unknownError) {
						const message = errorMessage(unknownError);
						setError(message);
						setSession((current) => {
							if (!current || !startedReviewId) return current;

							const failedSession: ReviewWorkspaceSession = {
								...current,
								execution: {
									...current.execution,
									status: "incomplete",
								},
								publication: {
									...current.publication,
									incompleteSession: true,
								},
							};
							void saveReviewSession(failedSession);
							return failedSession;
						});
					} finally {
						unlistenProgress?.();
						if (startedReviewId && activeReviewId.current === startedReviewId) {
							activeReviewId.current = null;
						}
						setRunning(false);
					}
				}}
				onRefreshGhStatus={refreshGhStatus}
				providerSettings={providerSettings}
			/>
		);
	}

	return (
		<WorkspaceShell
			actions={
				<>
					{running ? (
						<Button onClick={stopActiveReview} variant="destructive">
							<StopCircle className="size-4" />
							Stop review
						</Button>
					) : null}
					<Button
						disabled={running}
						onClick={() => setSession(null)}
						variant="outline"
					>
						<ArrowCounterClockwise className="size-4" />
						New review
					</Button>
				</>
			}
			subtitle="Open a repository, review a change set, curate generated feedback, and publish through gh."
			title="Review Workspace"
		>
			<div className="space-y-5">
				<SelectedProviderSummary providerSettings={session.providerSettings} />
				<SetupOverview session={session} />
				<ExecutionStatus execution={session.execution} />
				<FeedbackWorkspace
					feedback={session.feedback}
					ghStatus={ghStatus}
					isRunning={running}
					onDeleteFeedback={deleteFeedback}
					onFeedbackChange={persistFeedbackChange}
					onPublishFeedback={publishFeedback}
					repositoryPath={session.repository.path}
				/>
			</div>
		</WorkspaceShell>
	);

	async function persistFeedbackChange(nextFeedback: ReviewFeedbackItem) {
		const currentSession = session;
		if (!currentSession) return;

		setSession((current) =>
			current
				? {
						...current,
						feedback: current.feedback.map((item) =>
							item.id === nextFeedback.id ? nextFeedback : item,
						),
					}
				: current,
		);

		try {
			await updateReviewFeedback({
				sessionId: currentSession.changeSet.id,
				feedbackId: nextFeedback.id,
				feedback: nextFeedback,
			});
		} catch (unknownError) {
			setError(errorMessage(unknownError));
		}
	}

	async function publishFeedback(feedback: ReviewFeedbackItem) {
		const currentSession = session;
		if (!currentSession) return;

		try {
			await publishReviewFeedback({
				repositoryPath: currentSession.repository.path,
				feedback,
			});
			await persistFeedbackChange({
				...feedback,
				state: "published",
			});
		} catch (unknownError) {
			setError(errorMessage(unknownError));
		}
	}

	async function deleteFeedback(feedbackId: string) {
		const currentSession = session;
		if (!currentSession) return;

		setSession((current) =>
			current
				? {
						...current,
						feedback: current.feedback.filter((item) => item.id !== feedbackId),
						publication: {
							...current.publication,
							totalComments: Math.max(
								0,
								current.publication.totalComments - 1,
							),
						},
					}
				: current,
		);

		try {
			const nextSession = await deleteReviewFeedback({
				sessionId: currentSession.changeSet.id,
				feedbackId,
			});
			setSession(nextSession);
		} catch (unknownError) {
			setError(errorMessage(unknownError));
			setSession(currentSession);
		}
	}

	async function refreshGhStatus() {
		try {
			setGhStatus(await checkGhCliStatus());
		} catch (unknownError) {
			setGhStatus({
				installed: false,
				authenticated: false,
				message: errorMessage(unknownError),
			});
		}
	}

	async function stopActiveReview() {
		const reviewId = activeReviewId.current;
		if (!reviewId) return;

		const confirmed = window.confirm(
			"Stop the current review? The current model pass may finish in the background, but no more passes will be started.",
		);
		if (!confirmed) return;

		activeReviewId.current = null;
		setRunning(false);
		try {
			await cancelReviewSession(reviewId);
		} catch (unknownError) {
			setError(errorMessage(unknownError));
		}
		setSession((current) =>
			current
				? {
						...current,
						execution: {
							...current.execution,
							status: "cancelled",
						},
						publication: {
							...current.publication,
							incompleteSession: true,
						},
					}
				: current,
		);
	}
}

function applyReviewProgress(
	session: ReviewWorkspaceSession,
	execution: ReviewWorkspaceView["execution"],
	feedback: ReviewFeedbackItem[],
): ReviewWorkspaceSession {
	const nextFeedback = mergeFeedback(session.feedback, feedback);
	const inlineComments = nextFeedback.filter(
		(item) => item.type === "inline",
	).length;
	const summaryComments = nextFeedback.length - inlineComments;
	const limitedContextCount = nextFeedback.filter(
		(item) => item.limitedContext,
	).length;

	return {
		...session,
		execution: {
			...session.execution,
			status: execution.status,
			completedPasses: execution.completedPasses,
			totalPasses: execution.totalPasses,
			guardrailHits: execution.guardrailHits,
		},
		feedback: nextFeedback,
		publication: {
			...session.publication,
			totalComments: nextFeedback.length,
			inlineComments,
			summaryComments,
			limitedContextCount,
			incompleteSession:
				session.publication.incompleteSession ||
				execution.status === "incomplete" ||
				execution.status === "cancelled",
		},
	};
}

function mergeFeedback(
	currentFeedback: ReviewFeedbackItem[],
	incomingFeedback: ReviewFeedbackItem[],
): ReviewFeedbackItem[] {
	if (incomingFeedback.length === 0) return currentFeedback;

	const existingIds = new Set(currentFeedback.map((item) => item.id));
	const newItems = incomingFeedback.filter((item) => !existingIds.has(item.id));
	return [...currentFeedback, ...newItems];
}

function preserveCuratedFeedback(
	currentSession: ReviewWorkspaceSession,
	nextSession: ReviewWorkspaceSession,
): ReviewWorkspaceSession {
	const curatedById = new Map(
		currentSession.feedback
			.filter((item) => item.state !== "draft")
			.map((item) => [item.id, item]),
	);

	return {
		...nextSession,
		feedback: nextSession.feedback.map(
			(item) => curatedById.get(item.id) ?? item,
		),
	};
}

function SelectedProviderSummary({
	providerSettings,
}: {
	providerSettings: ProviderSettings;
}) {
	const selectedProvider = providerSettings.modelProviders.find(
		(provider) => provider.enabled && provider.selectedModelId,
	);

	return (
		<section className="border border-border bg-card p-4">
			<p className="text-xs font-medium uppercase text-muted-foreground">
				Provider and model
			</p>
			<h2 className="mt-1 text-lg font-semibold">
				{selectedProvider
					? `${selectedProvider.name} / ${selectedProvider.selectedModelId}`
					: "No provider selected"}
			</h2>
		</section>
	);
}

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}

function createReviewId(): string {
	return `review-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function createRunningSession(input: {
	repository: RepositoryDescriptor;
	changeSet: ChangeSetSnapshot;
	profiles: ReviewProfileItem[];
	providerSettings: ProviderSettings;
}): ReviewWorkspaceSession {
	const activeProfiles = input.profiles.filter((profile) => profile.selected);
	const reviewableFiles = input.changeSet.files.filter(
		(file) => !file.isGenerated,
	);
	const modifiedLines = input.changeSet.files.reduce(
		(total, file) => total + file.additions + file.deletions,
		0,
	);

	return {
		repository: {
			name: input.repository.name,
			path: input.repository.path,
			branch: input.repository.currentBranch ?? "detached",
			headSha: input.repository.headSha,
		},
		changeSource: {
			kind: "Preparing review",
			target: "Selected change source",
			intent: "Running model review passes",
			snapshot: `${input.changeSet.files.length} files, ${modifiedLines} modified lines`,
		},
		changeSet: input.changeSet,
		profiles: activeProfiles,
		providerSettings: input.providerSettings,
		execution: {
			status: "running",
			completedPasses: 0,
			totalPasses: reviewableFiles.length * activeProfiles.length,
			changedFiles: input.changeSet.files.length,
			modifiedLines,
			explorationRequests: 0,
			guardrailHits: 0,
		},
		feedback: [],
		publication: {
			target: "gh pull request publication not selected",
			totalComments: 0,
			inlineComments: 0,
			summaryComments: 0,
			limitedContextCount: 0,
			incompleteSession: false,
		},
	};
}
