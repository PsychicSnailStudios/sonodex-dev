<script lang="ts">
	import { Pencil, Trash, FolderInput, Paperclip, CloudDownload, FolderOpen, Loader2 } from "lucide-svelte";
	import * as Tooltip from "$shadcn/tooltip/index.js";
	import { buttonVariants } from "$shadcn/button/index.js";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import ArtistsList from "$lib/components/app-ui/ArtistsList.svelte";
	import { getAlbumUidFromName, library } from "$ts/store/library.svelte";
	import { parseAlbumEntries } from "$ts/util/helpers";
	import { setSelection } from "$ts/store/state_session.svelte";
	import { openEditModal } from "$ts/store/editModal.svelte";
	import {
		removeTrackFromLibrary,
		enrichTrack,
		replaceTrackPath,
		openTrackInExplorer,
	} from "$ts/library/libraryManager";
	import type { Track } from "$ts/util/types";
	import ScrollingText from "$lib/components/app-ui/ScrollingText.svelte";
	import { Checkbox } from "$shadcn/checkbox/index.js";

	let {
		track,
		selected = false,
		onToggle,
		onShiftClick,
	}: {
		track: Track;
		selected?: boolean;
		onToggle?: () => void;
		onShiftClick?: () => void;
	} = $props();

	const albumList = $derived(parseAlbumEntries(track.albums));

	let enriching = $state(false);

	async function handleEnrich(e: MouseEvent) {
		e.stopPropagation();
		enriching = true;
		try {
			await enrichTrack(track.uid);
		} finally {
			enriching = false;
		}
	}

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
			<ArtworkDisplay uid={track.uid} size={40} type="track" />
			<div class="min-w-0 grid">
				{#if track.path != ""}
					<button
						onclick={(e) => { e.stopPropagation(); setSelection(track.uid, "track"); }}
						class="text-sm truncate cursor-pointer hover:underline text-left"
					>
						<ScrollingText text={track.title} hoverOnly />
					</button>
				{:else}
					<button
						onclick={(e) => { e.stopPropagation(); setSelection(track.uid, "track"); }}
						class="text-sm text-muted-foreground truncate cursor-pointer hover:underline text-left"
					>
						<ScrollingText text={track.title} hoverOnly />
					</button>
				{/if}
				<div class="text-xs text-muted-foreground truncate">
					<ArtistsList artists={track.artists} />
					{#if albumList.length > 0}
						{" · "}
						{#each albumList as album, i}
							<button
								onclick={(e) => { e.stopPropagation(); setSelection(getAlbumUidFromName(album.name), "album"); }}
								class="text-xs cursor-pointer hover:underline"
							>
								{album.name}{i < albumList.length - 1 ? "," : ""}
							</button>
						{/each}
					{/if}
				</div>
			</div>
		</div>

		{#if !selected}
			<div class="flex gap-2 shrink-0">
				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "destructive", size: "icon" })}
						onclick={(e: MouseEvent) => { e.stopPropagation(); removeTrackFromLibrary(track.uid); }}
					>
						<Trash />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Remove from library</p></Tooltip.Content>
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
						onclick={(e: MouseEvent) => { e.stopPropagation(); replaceTrackPath(track.uid); }}
					>
						<Paperclip />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Replace file path</p></Tooltip.Content>
				</Tooltip.Root>

				{#if track.path != ""}
					<Tooltip.Root>
						<Tooltip.Trigger
							class={buttonVariants({ variant: "outline", size: "icon" })}
							onclick={(e: MouseEvent) => { e.stopPropagation(); openTrackInExplorer(track.path); }}
						>
							<FolderOpen />
						</Tooltip.Trigger>
						<Tooltip.Content><p>Show in file explorer</p></Tooltip.Content>
					</Tooltip.Root>
				{/if}

				<Tooltip.Root>
					<Tooltip.Trigger
						class={buttonVariants({ variant: "outline", size: "icon" })}
						onclick={(e: MouseEvent) => { e.stopPropagation(); openEditModal({ type: "track", uid: track.uid }); }}
					>
						<Pencil />
					</Tooltip.Trigger>
					<Tooltip.Content><p>Edit metadata</p></Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/if}
	</div>
</div>