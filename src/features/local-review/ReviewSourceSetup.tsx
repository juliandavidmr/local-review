import { useEffect, useMemo, useState } from "react";

import {
	listRepositoryBranches,
	type ReviewChangeSourceKind,
} from "@/adapters/tauri-local-review-api";

import { CompareRefsFields } from "./CompareRefsFields";
import { ReviewSourceSelector } from "./ReviewSourceSelector";

type ReviewSourceSetupProps = {
	baseRef: string;
	headRef: string;
	onBaseRefChange: (value: string) => void;
	onHeadRefChange: (value: string) => void;
	onSourceChange: (value: ReviewChangeSourceKind) => void;
	repositoryPath: string;
	sourceKind: ReviewChangeSourceKind;
};

export function ReviewSourceSetup({
	baseRef,
	headRef,
	onBaseRefChange,
	onHeadRefChange,
	onSourceChange,
	repositoryPath,
	sourceKind,
}: ReviewSourceSetupProps) {
	const [branches, setBranches] = useState<string[]>([]);
	const [isLoadingBranches, setIsLoadingBranches] = useState(false);
	const [branchError, setBranchError] = useState<string | null>(null);
	const trimmedRepositoryPath = repositoryPath.trim();
	const branchNamesKey = useMemo(() => branches.join("\n"), [branches]);

	useEffect(() => {
		if (sourceKind !== "compare_refs" || trimmedRepositoryPath.length === 0) {
			setBranches([]);
			setBranchError(null);
			return;
		}

		let isActive = true;
		setIsLoadingBranches(true);
		setBranchError(null);

		listRepositoryBranches(trimmedRepositoryPath)
			.then((nextBranches) => {
				if (!isActive) return;
				setBranches(nextBranches);
			})
			.catch((unknownError) => {
				if (!isActive) return;
				setBranches([]);
				setBranchError(errorMessage(unknownError));
			})
			.finally(() => {
				if (!isActive) return;
				setIsLoadingBranches(false);
			});

		return () => {
			isActive = false;
		};
	}, [sourceKind, trimmedRepositoryPath]);

	useEffect(() => {
		if (sourceKind !== "compare_refs" || branches.length === 0) return;

		const defaultBaseRef = preferredBaseBranch(branches);
		const defaultHeadRef = preferredHeadBranch(branches);

		if (!branches.includes(baseRef) && defaultBaseRef) {
			onBaseRefChange(defaultBaseRef);
		}

		if (!branches.includes(headRef) && defaultHeadRef) {
			onHeadRefChange(defaultHeadRef);
		}
	}, [
		baseRef,
		branchNamesKey,
		branches,
		headRef,
		onBaseRefChange,
		onHeadRefChange,
		sourceKind,
	]);

	return (
		<>
			<ReviewSourceSelector onChange={onSourceChange} value={sourceKind} />
			{sourceKind === "compare_refs" ? (
				<>
					<CompareRefsFields
						baseRef={baseRef}
						branches={branches}
						disabled={isLoadingBranches}
						headRef={headRef}
						onBaseRefChange={onBaseRefChange}
						onHeadRefChange={onHeadRefChange}
					/>
					{isLoadingBranches ? (
						<p className="mt-2 text-xs text-muted-foreground">
							Loading repository branches...
						</p>
					) : null}
					{branchError ? (
						<p className="mt-2 text-xs text-destructive">{branchError}</p>
					) : null}
					{!isLoadingBranches &&
					!branchError &&
					trimmedRepositoryPath.length > 0 &&
					branches.length === 0 ? (
						<p className="mt-2 text-xs text-muted-foreground">
							No local or remote branches were found for this repository.
						</p>
					) : null}
				</>
			) : null}
		</>
	);
}

function preferredBaseBranch(branches: string[]): string | undefined {
	return (
		branches.find((branch) => branch === "develop") ??
		branches.find((branch) => branch === "origin/develop") ??
		branches.find((branch) => branch === "main") ??
		branches.find((branch) => branch === "origin/main") ??
		branches.find((branch) => branch === "master") ??
		branches.find((branch) => branch === "origin/master") ??
		branches[0]
	);
}

function preferredHeadBranch(branches: string[]): string | undefined {
	return branches[0];
}

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}
