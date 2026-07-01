import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";

type CompareRefsFieldsProps = {
	baseRef: string;
	branches: string[];
	disabled?: boolean;
	headRef: string;
	onBaseRefChange: (value: string) => void;
	onHeadRefChange: (value: string) => void;
};

export function CompareRefsFields({
	baseRef,
	branches,
	disabled = false,
	headRef,
	onBaseRefChange,
	onHeadRefChange,
}: CompareRefsFieldsProps) {
	return (
		<div className="mt-4 grid gap-3 md:grid-cols-2">
			<div className="space-y-2">
				<Label htmlFor="base-ref">Base ref</Label>
				<Select
					disabled={disabled || branches.length === 0}
					onValueChange={onBaseRefChange}
					value={baseRef}
				>
					<SelectTrigger id="base-ref">
						<SelectValue placeholder="Select base branch" />
					</SelectTrigger>
					<SelectContent>
						{branches.map((branch) => (
							<SelectItem key={branch} value={branch}>
								{branch}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				<p className="text-xs text-muted-foreground">
					Use the branch where your current branch started, for example
					develop or origin/develop.
				</p>
			</div>
			<div className="space-y-2">
				<Label htmlFor="head-ref">Head ref</Label>
				<Select
					disabled={disabled || branches.length === 0}
					onValueChange={onHeadRefChange}
					value={headRef}
				>
					<SelectTrigger id="head-ref">
						<SelectValue placeholder="Select head branch" />
					</SelectTrigger>
					<SelectContent>
						{branches.map((branch) => (
							<SelectItem key={branch} value={branch}>
								{branch}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				<p className="text-xs text-muted-foreground">
					Use the current branch or another branch to review.
				</p>
			</div>
		</div>
	);
}
