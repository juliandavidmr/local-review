import { PencilSimpleIcon, PlusIcon } from "@phosphor-icons/react";
import { useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import type { ReviewProfileItem } from "@/domain/workspace-view";

import { ProfileEditorDialog } from "./ProfileEditorDialog";
import {
	createReviewProfile,
	emptyProfileDraft,
	profileToDraft,
	type ProfileDraft,
	updateReviewProfile,
} from "./profileDraft";

type ReviewProfilesSetupProps = {
	profiles: ReviewProfileItem[];
	onProfilesChange: (profiles: ReviewProfileItem[]) => void;
};

type ProfileDialogState =
	| { mode: "create"; profileId?: never }
	| { mode: "edit"; profileId: string };

export function ReviewProfilesSetup({
	profiles,
	onProfilesChange,
}: ReviewProfilesSetupProps) {
	const [dialogState, setDialogState] = useState<ProfileDialogState | null>(
		null,
	);
	const editingProfile = profiles.find(
		(profile) => profile.id === dialogState?.profileId,
	);
	const dialogDraft = useMemo(() => {
		if (dialogState?.mode !== "edit" || !editingProfile) {
			return emptyProfileDraft;
		}

		return profileToDraft(editingProfile);
	}, [dialogState, editingProfile]);

	function updateProfileSelection(profileId: string, selected: boolean) {
		onProfilesChange(
			profiles.map((profile) =>
				profile.id === profileId ? { ...profile, selected } : profile,
			),
		);
	}

	function saveProfile(draft: ProfileDraft) {
		if (dialogState?.mode === "edit" && editingProfile) {
			onProfilesChange(
				profiles.map((profile) =>
					profile.id === editingProfile.id
						? updateReviewProfile(profile, draft)
						: profile,
				),
			);
			return;
		}

		onProfilesChange([createReviewProfile(draft), ...profiles]);
	}

	return (
		<div className="grid gap-3 grid-cols-1">
			<div className="space-y-3">
				{profiles.map((profile) => (
					<div
						className="flex items-center justify-between gap-3 border border-border p-3"
						key={profile.id}
					>
						<label className="flex min-w-0 flex-1 items-start justify-between gap-3">
							<span className="min-w-0">
								<span className="block truncate text-sm font-medium">
									{profile.name}
								</span>
								<span className="mt-1 block text-xs text-muted-foreground">
									{profile.scope}
								</span>
							</span>
						</label>
						<div className="flex shrink-0 items-center gap-2">
							<Button
								aria-label={`Edit ${profile.name}`}
								onClick={() =>
									setDialogState({ mode: "edit", profileId: profile.id })
								}
								size="icon-sm"
								type="button"
								variant="ghost"
							>
								<PencilSimpleIcon className="size-4" />
							</Button>
							<Switch
								checked={profile.selected}
								onCheckedChange={(selected) =>
									updateProfileSelection(profile.id, selected)
								}
							/>
						</div>
					</div>
				))}
			</div>

			<Button
				className="w-full"
				onClick={() => setDialogState({ mode: "create" })}
				type="button"
				variant="outline"
			>
				<PlusIcon className="size-4" />
				Add profile
			</Button>

			<ProfileEditorDialog
				draft={dialogDraft}
				mode={dialogState?.mode ?? "create"}
				onOpenChange={(open) => {
					if (!open) setDialogState(null);
				}}
				onSubmit={saveProfile}
				open={dialogState !== null}
			/>
		</div>
	);
}
