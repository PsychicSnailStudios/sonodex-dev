<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { open } from "@tauri-apps/plugin-dialog";
	import { Upload, Clipboard, Trash2, ImageIcon } from "lucide-svelte";
	import Button from "$lib/components/ui/button/button.svelte";

	let {
		entityType,
		entityId,
		onchange,
	} = $props<{
		entityType: "track" | "album" | "artist" | "playlist";
		entityId: number;
		onchange?: (path: string | null) => void;
	}>();

	let blobUrl = $state<string | null>(null);
	let loading = $state(true);

	const artworkCommand: Record<string, string> = {
		track: "get_track_artwork",
		album: "get_album_artwork",
		artist: "get_artist_profile_art",
		playlist: "get_playlist_artwork",
	};

	async function loadArtwork() {
		loading = true;
		try {
			const bytes = await invoke<number[] | null>(artworkCommand[entityType], { id: entityId });
			if (bytes) {
				const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
				blobUrl = URL.createObjectURL(blob);
			} else {
				blobUrl = null;
			}
		} catch {
			blobUrl = null;
		}
		loading = false;
	}

	async function handleUpload() {
		const selected = await open({
			filters: [{ name: "Image", extensions: ["jpg", "jpeg", "png", "webp"] }],
			multiple: false,
		});
		if (selected && typeof selected === "string") {
			onchange?.(selected);
			blobUrl = `asset://localhost/${encodeURIComponent(selected)}`;
		}
	}

	async function handlePaste() {
		try {
			const items = await navigator.clipboard.read();
			for (const item of items) {
				const imageType = item.types.find((t) => t.startsWith("image/"));
				if (imageType) {
					const blob = await item.getType(imageType);
					blobUrl = URL.createObjectURL(blob);
					const reader = new FileReader();
					reader.onload = () => {
						onchange?.("__paste__");
					};
					reader.readAsArrayBuffer(blob);
					return;
				}
			}
		} catch {
		}
	}

	function handleClear() {
		blobUrl = null;
		onchange?.(null);
	}

	$effect(() => {
		if (entityId) loadArtwork();
	});
</script>

<div class="flex gap-4 items-start">
	<div class="relative shrink-0 w-32 h-32 rounded-md border bg-muted overflow-hidden flex items-center justify-center">
		{#if loading}
			<div class="w-6 h-6 rounded-full border-2 border-primary border-t-transparent animate-spin"></div>
		{:else if blobUrl}
			<img src={blobUrl} alt="Artwork" class="w-full h-full object-cover" />
		{:else}
			<ImageIcon class="w-10 h-10 text-muted-foreground" />
		{/if}
	</div>

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
