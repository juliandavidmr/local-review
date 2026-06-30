import type {
	ReviewProfileItem,
	ReviewProfileScopeKind,
} from "@/domain/workspace-view";

export type ProfileDraft = {
	name: string;
	prompt: string;
	scopeKind: ReviewProfileScopeKind;
};

export const emptyProfileDraft: ProfileDraft = {
	name: "",
	prompt: "",
	scopeKind: "global",
};

export function createReviewProfile(draft: ProfileDraft): ReviewProfileItem {
	const name = draft.name.trim();
	const prompt = draft.prompt.trim();

	return {
		id: createProfileId(name),
		name,
		scope: scopeLabel(draft.scopeKind),
		scopeKind: draft.scopeKind,
		selected: true,
		enabledByDefault: draft.scopeKind === "global",
		criteria: [name],
		fileGlobs: ["*"],
		prompt,
	};
}

export function profileToDraft(profile: ReviewProfileItem): ProfileDraft {
	return {
		name: profile.name,
		prompt: profile.prompt,
		scopeKind: profile.scopeKind,
	};
}

export function updateReviewProfile(
	profile: ReviewProfileItem,
	draft: ProfileDraft,
): ReviewProfileItem {
	const name = draft.name.trim();

	return {
		...profile,
		name,
		scope: scopeLabel(draft.scopeKind),
		scopeKind: draft.scopeKind,
		enabledByDefault: draft.scopeKind === "global",
		criteria: [name],
		prompt: draft.prompt.trim(),
	};
}

export function scopeLabel(scopeKind: ReviewProfileScopeKind): string {
	switch (scopeKind) {
		case "global":
			return "Global";
		case "repository":
			return "Repository path";
		case "folder":
			return "Folder path";
	}
}

function createProfileId(name: string): string {
	const slug = name
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-+|-+$/g, "");

	return `${slug || "profile"}-${Date.now()}`;
}
