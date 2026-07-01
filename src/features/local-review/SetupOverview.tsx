import type { ProviderSettings } from "@/domain";
import type { ReviewWorkspaceView } from "@/domain/workspace-view";

type SetupOverviewProps = {
	providerSettings: ProviderSettings;
	session: ReviewWorkspaceView;
};

export function SetupOverview({ providerSettings, session }: SetupOverviewProps) {
	const selectedProfiles = session.profiles.filter(
		(profile) => profile.selected,
	);
	const selectedProvider = providerSettings.modelProviders.find(
		(provider) => provider.enabled && provider.selectedModelId,
	);
	const providerLabel = selectedProvider
		? `${selectedProvider.name} / ${selectedProvider.selectedModelId}`
		: "No provider selected";

	return (
		<section className="overflow-hidden rounded-lg border border-border bg-card shadow-sm">
			<div className="grid gap-px bg-border/70 sm:grid-cols-[minmax(0,1.05fr)_minmax(0,1fr)]">
				<div className="min-w-0 bg-card p-4">
					<p className="text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
						Provider and model
					</p>
					<p className="mt-2 truncate text-base font-semibold text-foreground">
						{providerLabel}
					</p>
				</div>

				<div className="min-w-0 bg-card p-4">
					<p className="text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
						Repository
					</p>
					<p className="mt-2 truncate text-base font-semibold text-foreground">
						{session.repository.name}
					</p>
				</div>
			</div>

			<div className="grid gap-4 border-t border-border/70 p-4 lg:grid-cols-[minmax(0,1fr)_18rem]">
				<dl className="grid min-w-0 gap-3 sm:grid-cols-2 lg:grid-cols-1">
					<div className="min-w-0">
						<dt className="text-xs text-muted-foreground">Path</dt>
						<dd className="mt-1 truncate font-mono text-sm text-foreground">
							{session.repository.path}
						</dd>
					</div>
					<div className="min-w-0">
						<dt className="text-xs text-muted-foreground">Branch</dt>
						<dd className="mt-1 truncate font-mono text-sm text-foreground">
							{session.repository.branch}
						</dd>
					</div>
				</dl>

				<div className="min-w-0 border-t border-border/70 pt-4 lg:border-l lg:border-t-0 lg:pl-4 lg:pt-0">
					<div className="flex items-center justify-between gap-3">
						<p className="text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
							Review profiles
						</p>
						<span className="rounded-full border border-border bg-muted px-2 py-0.5 text-xs font-medium text-muted-foreground">
							{selectedProfiles.length} active
						</span>
					</div>
					<div className="mt-3 grid gap-2">
						{session.profiles.map((profile) => (
							<div
								className="min-w-0 rounded-md border border-border/70 bg-muted/35 px-3 py-2"
								key={profile.id}
							>
								<p className="truncate text-sm font-semibold text-foreground">
									{profile.name}
								</p>
								<p className="mt-0.5 truncate text-xs text-muted-foreground">
									{profile.scope}
								</p>
							</div>
						))}
					</div>
				</div>
			</div>
		</section>
	);
}
