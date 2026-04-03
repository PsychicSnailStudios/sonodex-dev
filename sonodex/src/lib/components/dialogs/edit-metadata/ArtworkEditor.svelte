<script lang="ts">
	
	// APP
	import { open } from "@tauri-apps/plugin-dialog";

	// COMPONENTS
	import { Upload, Clipboard, Trash2 } from "lucide-svelte";
	import Button from "$lib/components/ui/button/button.svelte";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";

	// PROPS
	let {
		entityType,
		entityUid,
		onchange,
	} = $props<{
		entityType: "track" | "album" | "artist" | "playlist";
		entityUid: string;
		onchange?: (path: string | null) => void;
	}>();

	// VARIABLES
	let previewPath = $state<string | null>(null);
	
	// FUNCTIONS
	async function handleUpload() {
		const selected = await open({
			filters: [{ name: "Image", extensions: ["jpg", "jpeg", "png", "webp"] }],
			multiple: false,
		});
		if (selected && typeof selected === "string") {
			previewPath = selected;
			onchange?.(selected);
		}
	}

	async function handlePaste() {
		try {
			const items = await navigator.clipboard.read();
			for (const item of items) {
				const imageType = item.types.find((t) => t.startsWith("image/"));
				if (imageType) {
					const blob = await item.getType(imageType);
					const reader = new FileReader();
					reader.onload = () => {
						onchange?.("__paste__");
					};
					reader.readAsArrayBuffer(blob);
					return;
				}
			}
		} catch {}
	}

	function handleClear() {
		previewPath = null;
		onchange?.(null);
	}
</script>

<div class="flex gap-4 items-start">
	<ArtworkDisplay uid={entityUid} size={128} type={entityType} previewPath={previewPath} />

	<div class="flex flex-col gap-2 justify-center pt-1">
		<Button variant="outline" size="sm" onclick={handleUpload} class="justify-start gap-2">
			<Upload class="w-4 h-4" />
			Upload file
		</Button>
		<Button variant="outline" size="sm" onclick={handlePaste} class="justify-start gap-2">
			<Clipboard class="w-4 h-4" />
			Paste from clipboard
		</Button>
		<Button variant="outline" size="sm" onclick={handleClear} class="justify-start gap-2 text-destructive hover:text-destructive">
			<Trash2 class="w-4 h-4" />
			Remove artwork
		</Button>
	</div>
</div>