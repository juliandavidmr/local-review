import {
	ArrowsClockwiseIcon,
	CheckCircle,
	WarningCircle,
	XCircle,
} from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import type { GhCliStatus } from "@/adapters/tauri-local-review-api";

type GhStatusControlProps = {
	status: GhCliStatus | null;
	onRefresh: () => void | Promise<void>;
};

export function GhStatusControl({ status, onRefresh }: GhStatusControlProps) {
	const icon = !status ? (
		<WarningCircle className="size-5 text-muted-foreground" />
	) : status.installed && status.authenticated ? (
		<CheckCircle className="size-5 text-foreground" />
	) : status.installed ? (
		<WarningCircle className="size-5 text-muted-foreground" />
	) : (
		<XCircle className="size-5 text-destructive" />
	);
	const label = !status
		? "Checking gh CLI"
		: status.installed && status.authenticated
			? "gh CLI ready"
			: status.installed
				? "gh CLI not authenticated"
				: "gh CLI not installed";

	return (
		<section className="flex flex-col gap-3 border border-border bg-background p-4 md:flex-row md:items-center md:justify-between">
			<div className="flex items-start gap-3">
				{icon}
				<div>
					<h2 className="text-sm font-semibold">{label}</h2>
					<p className="mt-1 text-xs text-muted-foreground">
						{status?.message ??
							"Checking whether pull request publication can use GitHub CLI."}
					</p>
				</div>
			</div>
			<Button onClick={onRefresh} size="sm" variant="outline">
				<ArrowsClockwiseIcon className="size-4" />
				Refresh
			</Button>
		</section>
	);
}
