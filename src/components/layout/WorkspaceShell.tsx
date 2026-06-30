import type { ReactNode } from "react";

type WorkspaceShellProps = {
	title: string;
	subtitle: string;
	actions?: ReactNode;
	children: ReactNode;
};

export function WorkspaceShell({
	title,
	subtitle,
	actions,
	children,
}: WorkspaceShellProps) {
	return (
		<main className="min-h-screen bg-background text-foreground">
			<div className="border-b border-border bg-card">
				<div className="mx-auto flex max-w-screen-2xl items-center justify-between px-6 py-4">
					<div>
						<p className="text-xs font-medium uppercase text-muted-foreground">
							Local Review
						</p>
						<h1 className="mt-1 text-xl font-semibold">{title}</h1>
						<p className="mt-1 text-sm text-muted-foreground">{subtitle}</p>
					</div>
					{actions ? (
						<div className="flex shrink-0 items-center gap-2">{actions}</div>
					) : null}
				</div>
			</div>
			<div className="mx-auto max-w-screen-2xl px-6 py-6">{children}</div>
		</main>
	);
}
