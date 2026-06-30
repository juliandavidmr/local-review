import type { ReviewWorkspaceView } from "@/domain/workspace-view";

type SetupOverviewProps = {
	session: ReviewWorkspaceView;
};

export function SetupOverview({ session }: SetupOverviewProps) {
	const selectedProfiles = session.profiles.filter(
		(profile) => profile.selected,
	);

	return (
		<section className="grid gap-4 lg:grid-cols-2">
			<div className="border border-border bg-card p-4">
				<p className="text-xs font-medium uppercase text-muted-foreground">
					Repository
				</p>
				<h2 className="mt-2 text-lg font-semibold">
					{session.repository.name}
				</h2>
				<dl className="mt-4 space-y-2 text-sm">
					<div>
						<dt className="text-muted-foreground">Path</dt>
						<dd className="font-mono text-xs">{session.repository.path}</dd>
					</div>
					<div>
						<dt className="text-muted-foreground">Branch</dt>
						<dd>{session.repository.branch}</dd>
					</div>
				</dl>
			</div>

			<div className="border border-border bg-card p-4">
				<p className="text-xs font-medium uppercase text-muted-foreground">
					Review profiles
				</p>
				<h2 className="mt-2 text-lg font-semibold">
					{selectedProfiles.length} active profiles
				</h2>
				<div className="mt-4 space-y-2">
					{session.profiles.map((profile) => (
						<div key={profile.id}>
							<p className="font-medium">{profile.name}</p>
							<p className="text-xs text-muted-foreground">{profile.scope}</p>
						</div>
					))}
				</div>
			</div>
		</section>
	);
}
