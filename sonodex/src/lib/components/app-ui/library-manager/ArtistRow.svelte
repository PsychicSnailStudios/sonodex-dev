<script lang="ts">
	import { Pencil, Trash, CloudDownload } from "lucide-svelte";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { removeArtist } from "$lib/ts/app/libraryManager";
	import { library } from "$lib/ts/library.svelte";
	import type { Artist } from "$lib/ts/types";

	let { artist }: { artist: Artist } = $props();

	const trackCount = $derived(
		library.tracks.filter((t) => {
			try {
				const arr: string[] = t.artists ? JSON.parse(t.artists as string) : [];
				return arr.includes(artist.name);
			} catch {
				return false;
			}
		}).length
	);
</script>

<div class="flex gap-2 p-2 border-2 rounded-md justify-between items-center">
	<div class="flex gap-2 min-w-0 flex-1">
		<ArtworkDisplay uid={artist.uid} size={40} type="artist" />
		<div class="min-w-0 grid">
			<span class="text-sm truncate">{artist.name}</span>
			<div class="text-xs text-muted-foreground truncate">
				{trackCount} {trackCount === 1 ? "track" : "tracks"}
			</div>
		</div>
	</div>

	<div class="flex gap-2 shrink-0">
		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "destructive", size: "icon" })}
				onclick={() => removeArtist(artist.uid)}
			>
				<Trash />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Delete artist record</p></Tooltip.Content>
		</Tooltip.Root>

		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "outline", size: "icon" })}
				onclick={() => openEditModal({ type: "artist", uid: artist.uid })}
			>
				<Pencil />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Edit artist</p></Tooltip.Content>
		</Tooltip.Root>
	</div>
</div>
