<script lang="ts">
	import { Pencil, Trash, CloudDownload, Loader2 } from "lucide-svelte";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { removeAlbum, enrichAlbum } from "$lib/ts/app/libraryManager";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { getArtistUidFromName } from "$lib/ts/library.svelte";
	import type { Album } from "$lib/ts/util/types";

	let {
		album,
		selected = false,
		onToggle,
		onShiftClick,
	}: {
		album: Album;
		selected?: boolean;
		onToggle?: () => void;
		onShiftClick?: () => void;
	} = $props();

	let enriching = $state(false);

	async function handleEnrich(e: MouseEvent) {
		e.stopPropagation();
		enriching = true;
		try {
			await enrichAlbum(album.uid);
		} finally {
			enriching = false;
		}
	}

	const trackCount = $derived.by(() => {
		try {
			return album.tracks ? JSON.parse(album.tracks as string).length : 0;
		} catch { return 0; }
	});

	const artistList = $derived.by<string[]>(() => {
		try {
			return album.artists ? JSON.parse(album.artists as string) : [];
		} catch { return []; }
	});

	function handleRowClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (target.closest("a, button")) return;
		if (e.shiftKey) {
			onShiftClick?.();
		} else {
			onToggle?.();
		}
	}
</script>

<div class="flex gap-2 items-center">
	<Checkbox
		checked={selected}
		onCheckedChange={(v) => {
			if (v !== selected) onToggle?.();
		}}
	/>
	<div
		class="flex gap-2 p-2 border-2 rounded-md justify-between items-center flex-1 cursor-pointer"
		class:border-primary={selected}
		onclick={handleRowClick}
		onmousedown={(e) => { if (e.shiftKey) e.preventDefault(); }}
		role="row"
		aria-selected={selected}
	>
		<div class="flex gap-2 items-center min-w-0 flex-1">
			<ArtworkDisplay uid={album.uid} size={40} type="album" />
			<div class="min-w-0 grid">
				<button
					onclick={(e) => { e.stopPropagation(); setSelection(album.uid, "album"); }}
					class="text-sm truncate cursor-pointer hover:underline text-left"
				>
					{album.title}
				</button>
				<div class="text-xs text-muted-foreground truncate">
					{#if artistList.length > 0}
						{#each artistList as artist, i}
							<button
								onclick={(e) => { e.stopPropagation(); setSelection(getArtistUidFromName(artist), "artist"); }}
								class="text-xs cursor-pointer hover:underline"
							>
								{artist}{i < artistList.length - 1 ? "," : ""}
							</button>
						{/each}
					{:else if album.album_artist}
						<button
							onclick={(e) => { e.stopPropagation(); setSelection(getArtistUidFromName(album.album_artist!), "artist"); }}
							class="text-xs cursor-pointer hover:underline"
						>
							{album.album_artist}
						</button>
					{:else}
						—
					{/if}
					{#if album.release_date}{" · "}{album.release_date}{/if}
					{" · "}{trackCount} {trackCount === 1 ? "track" : "tracks"}
				</div>
			</div>
		</div>

		{#if !selected}
			<div class="flex gap-2 shrink-0">
				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "destructive", size: "icon" })}
						onclick={(e: MouseEvent) => { e.stopPropagation(); removeAlbum(album.uid); }}
					>
						<Trash />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Delete album record</p></Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "outline", size: "icon" })}
						onclick={handleEnrich}
						disabled={enriching}
					>
						{#if enriching}
							<Loader2 class="animate-spin" />
						{:else}
							<CloudDownload />
						{/if}
					</Tooltip.Trigger>
					<Tooltip.Content><p>Fetch metadata via API</p></Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "outline", size: "icon" })}
						onclick={(e: MouseEvent) => { e.stopPropagation(); openEditModal({ type: "album", uid: album.uid }); }}
					>
						<Pencil />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Edit album</p></Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/if}
	</div>
</div>