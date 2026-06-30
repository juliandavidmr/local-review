import { useMemo } from "react";

import { Badge } from "@/components/ui/badge";
import type { ReviewProfileItem } from "@/domain/workspace-view";

type ProfileManagerProps = {
	profiles: ReviewProfileItem[];
	repositoryPath: string;
};

export function ProfileManager({
	profiles,
	repositoryPath,
}: ProfileManagerProps) {
	const sortedProfiles = useMemo(
		() =>
			[...profiles].sort((left, right) => {
				if (left.scopeKind !== right.scopeKind) {
					return left.scopeKind.localeCompare(right.scopeKind);
				}
				return left.name.localeCompare(right.name);
			}),
		[profiles],
	);

	return (
		<section className="border border-border bg-card">
			<div className="flex flex-col gap-4 border-b border-border p-4 lg:flex-row lg:items-start lg:justify-between">
				<div>
					<p className="text-xs font-medium uppercase text-muted-foreground">
						Review profiles
					</p>
					<h2 className="mt-1 text-lg font-semibold">
						{profiles.length} review profiles
					</h2>
					<p className="mt-1 text-sm text-muted-foreground">
						Application-owned review guidance available for this workspace.
					</p>
				</div>
				<div className="flex flex-wrap items-center gap-2">
					<Badge variant="outline">{profiles.length} total</Badge>
				</div>
			</div>

			<div className="grid gap-3 bg-muted/40 p-4 xl:grid-cols-3">
				{sortedProfiles.map((profile) => (
					<article
						className="border border-border bg-card p-4"
						key={profile.id}
					>
						<div className="flex items-start justify-between gap-3">
							<div>
								<h3 className="font-semibold">{profile.name}</h3>
								<p className="mt-1 text-xs text-muted-foreground">
									{profile.scope}
								</p>
							</div>
						</div>

						<div className="mt-3 flex flex-wrap gap-2">
							<Badge variant="outline">{profile.scopeKind}</Badge>
						</div>

						<p className="mt-3 line-clamp-3 text-sm text-muted-foreground">
							{stripDescription(profile.prompt)}
						</p>
					</article>
				))}
			</div>

			<div className="border-t border-border px-4 py-3 text-xs text-muted-foreground">
				Repository scope target:{" "}
				<span className="font-mono">{repositoryPath}</span>
			</div>
		</section>
	);
}

function stripDescription(value: string): string {
	return value
		.split("\n")
		.map((line) => line.trim())
		.filter(Boolean)
		.join(" ")
		.replace(/[`*_>#-]+/g, "")
		.replace(/\s+/g, " ")
		.trim();
}
