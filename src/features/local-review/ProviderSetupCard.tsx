import { ArrowsClockwiseIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import type { ModelProviderSettings } from "@/domain";

type ProviderSetupCardProps = {
	provider: ModelProviderSettings;
	models: readonly { displayName: string; modelId: string }[];
	status?: { ok: boolean; message: string };
	isLoading: boolean;
	onBaseUrlChange: (providerId: string, baseUrl: string) => void;
	onModelSelect: (providerId: string, selectedModelId: string) => void;
	onRefresh: (provider: ModelProviderSettings) => void;
};

export function ProviderSetupCard({
	provider,
	models,
	status,
	isLoading,
	onBaseUrlChange,
	onModelSelect,
	onRefresh,
}: ProviderSetupCardProps) {
	const modelOptions = models.map((model) => ({
		label: model.displayName,
		value: model.modelId,
	}));
	const isLmStudio = provider.kind === "lm_studio";

	return (
		<div className="border border-border p-3">
			<div>
				<p className="font-medium">{provider.name}</p>
				<p className="mt-1 text-xs text-muted-foreground">
					{isLmStudio
						? "LM Studio local OpenAI-compatible server"
						: "Ollama local API endpoint"}
				</p>
			</div>
			<div className="mt-3 space-y-2">
				<Label htmlFor={`${provider.id}-setup-url`}>Base URL</Label>
				<Input
					id={`${provider.id}-setup-url`}
					onChange={(event) => onBaseUrlChange(provider.id, event.target.value)}
					value={provider.baseUrl}
				/>
			</div>
			<div className="mt-3 flex flex-col gap-2 md:flex-row md:items-end">
				<div className="w-full space-y-2">
					<Label>Model</Label>
					<Select
						onValueChange={(modelId) => onModelSelect(provider.id, modelId)}
						value={provider.selectedModelId ?? ""}
					>
						<SelectTrigger>
							<SelectValue placeholder="Select model" />
						</SelectTrigger>
						<SelectContent>
							{modelOptions.map((model) => (
								<SelectItem key={model.value} value={model.value}>
									{model.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
				<div className="flex items-end">
					<Button
						className="w-full md:w-auto"
						disabled={isLoading}
						onClick={() => onRefresh(provider)}
						size="sm"
						variant="outline"
					>
						<ArrowsClockwiseIcon className="size-4" />
						{isLoading
							? "Checking..."
							: isLmStudio
								? "Test LM Studio"
								: "Load models"}
					</Button>
				</div>
			</div>
			{status ? (
				<p
					className={
						status.ok
							? "mt-3 text-xs text-muted-foreground"
							: "mt-3 text-xs text-destructive"
					}
				>
					{status.ok ? "Connected" : "Unavailable"}: {status.message}
				</p>
			) : null}
		</div>
	);
}
