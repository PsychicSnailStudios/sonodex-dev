<script lang="ts">
	import { Pencil, Trash, CloudDownload } from "lucide-svelte";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { removeAlbum, enrichAlbumTracks } from "$lib/ts/app/libraryManager";
	import type { Album } from "$lib/ts/types";

	let { album }: { album: Album } = $props();

	const trackCount = $derived.by(() => {
		try {
			const parsed = album.tracks ? JSON.parse(album.tracks as string) : [];
			return parsed.length;
		} catch {
			return 0;
		}
	});
</script>

<div class="flex gap-2 p-2 border-2 rounded-md justify-between items-center">
	<div class="flex gap-2 min-w-0 flex-1">
		<ArtworkDisplay uid={album.uid} size={40} type="album" />
		<div class="min-w-0 grid">
			<span class="text-sm truncate">{album.title}</span>
			<div class="text-xs text-muted-foreground truncate">
				{album.album_artist ?? "—"}
				{#if album.release_date}
					{" · "}{album.release_date}
				{/if}
				{" · "}{trackCount} {trackCount === 1 ? "track" : "tracks"}
			</div>
		</div>
	</div>

	<div class="flex gap-2 shrink-0">
		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "destructive", size: "icon" })}
				onclick={() => removeAlbum(album.uid)}
			>
				<Trash />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Delete album record</p></Tooltip.Content>
		</Tooltip.Root>

		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "outline", size: "icon" })}
				onclick={() => enrichAlbumTracks(album.uid)}
			>
				<CloudDownload />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Fetch metadata for all tracks</p></Tooltip.Content>
		</Tooltip.Root>

		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "outline", size: "icon" })}
				onclick={() => openEditModal({ type: "album", uid: album.uid })}
			>
				<Pencil />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Edit album</p></Tooltip.Content>
		</Tooltip.Root>
	</div>
</div>
