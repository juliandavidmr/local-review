import { useEffect, useRef, useState } from "react";
import { FolderOpen } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import {
	selectSingleModelProvider,
	updateModelProviderSettings,
	type ProviderSettings,
} from "@/domain";
import type {
	GhCliStatus,
	ReviewChangeSourceKind,
} from "@/adapters/tauri-local-review-api";
import type { ReviewProfileItem } from "@/domain/workspace-view";

import { GhStatusControl } from "./GhStatusControl";
import { ProviderSetupCard } from "./ProviderSetupCard";
import { ReviewProfilesSetup } from "./ReviewProfilesSetup";
import { ReviewSourceSelector } from "./ReviewSourceSelector";
import { SetupBlock } from "./SetupBlock";
import { useProviderModelProbe } from "./useProviderModelProbe";

type InitialSetupScreenProps = {
	error?: string | null;
	initialProfiles: ReviewProfileItem[];
	isRunning?: boolean;
	ghStatus: GhCliStatus | null;
	providerSettings: ProviderSettings;
	onRefreshGhStatus: () => void | Promise<void>;
	onComplete: (setup: {
		repositoryPath: string;
		reviewSourceKind: ReviewChangeSourceKind;
		profiles: ReviewProfileItem[];
		providerSettings: ProviderSettings;
	}) => void | Promise<void>;
};

export function InitialSetupScreen({
	error,
	initialProfiles,
	isRunning = false,
	ghStatus,
	providerSettings,
	onRefreshGhStatus,
	onComplete,
}: InitialSetupScreenProps) {
	const [repositoryPath, setRepositoryPath] = useState("");
	const [profiles, setProfiles] = useState(initialProfiles);
	const [settings, setSettings] = useState(() =>
		selectSingleModelProvider(providerSettings, "lm-studio"),
	);
	const [reviewSourceKind, setReviewSourceKind] =
		useState<ReviewChangeSourceKind>("current_branch");
	const { loadingProviderId, modelsByProvider, refreshProvider, statuses } =
		useProviderModelProbe(settings, setSettings);
	const autoTestedLmStudio = useRef(false);
	const activeProfiles = profiles.filter((profile) => profile.selected);
	const selectedProvider = settings.modelProviders.find(
		(provider) => provider.enabled && provider.selectedModelId,
	);
	const activeProvider = settings.modelProviders.find(
		(provider) => provider.enabled,
	);
	const activeProviderId = activeProvider?.id ?? "lm-studio";
	const canStart =
		repositoryPath.trim().length > 0 &&
		Boolean(selectedProvider) &&
		activeProfiles.length > 0 &&
		!isRunning;

	useEffect(() => {
		if (autoTestedLmStudio.current) return;

		const lmStudio = settings.modelProviders.find(
			(provider) => provider.id === "lm-studio" && provider.enabled,
		);
		if (!lmStudio) return;

		autoTestedLmStudio.current = true;
		void refreshProvider(lmStudio);
	}, []);

	async function chooseRepositoryFolder() {
		try {
			const dialog = await import("@tauri-apps/plugin-dialog");
			const selected = await dialog.open({
				directory: true,
				multiple: false,
				title: "Select Git repository",
			});

			if (typeof selected === "string") {
				setRepositoryPath(selected);
			}
		} catch {
			// Browser preview fallback keeps the manual path input usable.
		}
	}

	function selectProvider(providerId: string, selectedModelId: string) {
		setSettings((current) =>
			selectSingleModelProvider(current, providerId, selectedModelId),
		);
	}

	function selectProviderType(providerId: string) {
		setSettings((current) => selectSingleModelProvider(current, providerId));
	}

	function updateProviderBaseUrl(providerId: string, baseUrl: string) {
		setSettings((current) =>
			updateModelProviderSettings(current, providerId, (provider) => ({
				...provider,
				baseUrl,
			})),
		);
	}

	return (
		<main className="flex min-h-screen items-center justify-center bg-muted/40 p-6">
			<section className="w-full max-w-xl border border-border bg-card shadow-sm">
				<div className="border-b border-border p-6">
					<p className="text-xs font-medium uppercase text-muted-foreground">
						Local Review setup
					</p>
					<h1 className="mt-2 text-2xl font-semibold">
						Start a review session
					</h1>
					<p className="mt-2 max-w-2xl text-sm text-muted-foreground">
						Select a local repository, choose a local model provider, and
						activate review profiles before generating feedback.
					</p>
				</div>

				<div className="space-y-5 p-6">
					<GhStatusControl onRefresh={onRefreshGhStatus} status={ghStatus} />

					<SetupBlock title="Repository">
						<div className="grid gap-2 md:grid-cols-3">
							<Input
								className="md:col-span-2"
								onChange={(event) => setRepositoryPath(event.target.value)}
								placeholder="/Users/name/project"
								value={repositoryPath}
							/>
							<Button onClick={chooseRepositoryFolder} variant="outline">
								<FolderOpen className="size-4" />
								Select folder
							</Button>
						</div>
					</SetupBlock>

					<SetupBlock title="Provider and model">
						<div className="space-y-3">
							<div className="space-y-2">
								<Label>Provider type</Label>
								<Select
									onValueChange={selectProviderType}
									value={activeProviderId}
								>
									<SelectTrigger>
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{settings.modelProviders.map((provider) => (
											<SelectItem key={provider.id} value={provider.id}>
												{provider.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</div>

							{activeProvider ? (
								<ProviderSetupCard
									isLoading={loadingProviderId === activeProvider.id}
									models={modelsByProvider[activeProvider.id] ?? []}
									onBaseUrlChange={updateProviderBaseUrl}
									onModelSelect={selectProvider}
									onRefresh={refreshProvider}
									provider={activeProvider}
									status={statuses[activeProvider.id]}
								/>
							) : null}
						</div>
					</SetupBlock>

					<SetupBlock title="Review source">
						<ReviewSourceSelector
							onChange={setReviewSourceKind}
							value={reviewSourceKind}
						/>
					</SetupBlock>

					<SetupBlock title="Review profiles">
						<ReviewProfilesSetup
							onProfilesChange={setProfiles}
							profiles={profiles}
						/>
					</SetupBlock>
				</div>

				<div className="flex items-center justify-end gap-3 border-t border-border p-6">
					{error ? (
						<p className="mr-auto max-w-xl text-sm text-destructive">{error}</p>
					) : null}
					<Button
						disabled={!canStart}
						onClick={() =>
							onComplete({
								repositoryPath: repositoryPath.trim(),
								reviewSourceKind,
								profiles,
								providerSettings: settings,
							})
						}
					>
						{isRunning ? "Running review..." : "Start review workspace"}
					</Button>
				</div>
			</section>
		</main>
	);
}
