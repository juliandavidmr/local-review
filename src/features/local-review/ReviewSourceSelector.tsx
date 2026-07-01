import type { ReviewChangeSourceKind } from "@/adapters/tauri-local-review-api";

type ReviewSourceSelectorProps = {
	value: ReviewChangeSourceKind;
	onChange: (value: ReviewChangeSourceKind) => void;
};

const reviewSourceOptions: Array<{
	value: ReviewChangeSourceKind;
	label: string;
	description: string;
}> = [
	{
		value: "current_branch",
		label: "Current branch",
		description: "Diff the current branch against its upstream or main base.",
	},
	{
		value: "compare_refs",
		label: "Compare refs",
		description: "Manually set the base and head refs, such as develop...HEAD.",
	},
	{
		value: "staged_changes",
		label: "Staged changes",
		description: "Review only changes already staged with git add.",
	},
	{
		value: "unstaged_changes",
		label: "Unstaged changes",
		description: "Review local working tree changes that are not staged.",
	},
];

export function ReviewSourceSelector({
	value,
	onChange,
}: ReviewSourceSelectorProps) {
	return (
		<div className="grid gap-3 md:grid-cols-4">
			{reviewSourceOptions.map((option) => (
				<button
					className={
						value === option.value
							? "border border-foreground bg-background p-3 text-left"
							: "border border-border bg-background p-3 text-left hover:bg-muted"
					}
					key={option.value}
					onClick={() => onChange(option.value)}
					type="button"
				>
					<span className="block text-sm font-medium">{option.label}</span>
					<span className="mt-1 block text-xs text-muted-foreground">
						{option.description}
					</span>
				</button>
			))}
		</div>
	);
}
