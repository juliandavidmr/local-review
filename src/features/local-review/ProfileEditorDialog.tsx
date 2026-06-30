import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type { ReviewProfileScopeKind } from "@/domain/workspace-view";

import { emptyProfileDraft, type ProfileDraft } from "./profileDraft";

type ProfileEditorDialogProps = {
	draft?: ProfileDraft;
	mode: "create" | "edit";
	open: boolean;
	onOpenChange: (open: boolean) => void;
	onSubmit: (draft: ProfileDraft) => void;
};

export function ProfileEditorDialog({
	draft,
	mode,
	open,
	onOpenChange,
	onSubmit,
}: ProfileEditorDialogProps) {
	const [currentDraft, setCurrentDraft] = useState<ProfileDraft>(
		draft ?? emptyProfileDraft,
	);
	const canSubmit =
		currentDraft.name.trim().length > 0 &&
		currentDraft.prompt.trim().length > 0;

	useEffect(() => {
		if (!open) return;

		setCurrentDraft(draft ?? emptyProfileDraft);
	}, [draft, open]);

	function submitProfile(event: React.FormEvent<HTMLFormElement>) {
		event.preventDefault();
		if (!canSubmit) return;

		onSubmit(currentDraft);
		onOpenChange(false);
	}

	return (
		<Dialog onOpenChange={onOpenChange} open={open}>
			<DialogContent className="sm:max-w-lg">
				<form className="space-y-4" onSubmit={submitProfile}>
					<DialogHeader>
						<DialogTitle>
							{mode === "create" ? "Create manual profile" : "Edit profile"}
						</DialogTitle>
						<DialogDescription>
							Define the review guidance and where this profile applies.
						</DialogDescription>
					</DialogHeader>

					<div className="space-y-2">
						<Label htmlFor="profile-name">Name</Label>
						<Input
							id="profile-name"
							onChange={(event) =>
								setCurrentDraft((current) => ({
									...current,
									name: event.target.value,
								}))
							}
							placeholder="Security"
							value={currentDraft.name}
						/>
					</div>

					<div className="space-y-2">
						<Label>Scope</Label>
						<Select
							onValueChange={(scopeKind) =>
								setCurrentDraft((current) => ({
									...current,
									scopeKind: scopeKind as ReviewProfileScopeKind,
								}))
							}
							value={currentDraft.scopeKind}
						>
							<SelectTrigger>
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="global">Global</SelectItem>
								<SelectItem value="repository">Repository path</SelectItem>
								<SelectItem value="folder">Folder path</SelectItem>
							</SelectContent>
						</Select>
					</div>

					<div className="space-y-2">
						<Label htmlFor="profile-prompt">Prompt</Label>
						<Textarea
							className="min-h-32"
							id="profile-prompt"
							onChange={(event) =>
								setCurrentDraft((current) => ({
									...current,
									prompt: event.target.value,
								}))
							}
							placeholder="Review for..."
							value={currentDraft.prompt}
						/>
					</div>

					<DialogFooter>
						<Button
							onClick={() => onOpenChange(false)}
							type="button"
							variant="outline"
						>
							Cancel
						</Button>
						<Button disabled={!canSubmit} type="submit">
							{mode === "create" ? "Add profile" : "Save changes"}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	);
}
