<script lang="ts">
	import { Pencil, Trash, FolderInput, Paperclip, CloudDownload, FolderOpen } from "lucide-svelte";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import ArtistsList from "$lib/components/app-ui/ArtistsList.svelte";
	import { getAlbumUidFromName, library } from "$lib/ts/library.svelte";
	import { parseAlbumEntries } from "$lib/ts/util/helpers";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import {
		removeTrackFromLibrary,
		enrichTrack,
		replaceTrackPath,
		openTrackInExplorer,
	} from "$lib/ts/app/libraryManager";
	import type { Track } from "$lib/ts/types";
    import ScrollingText from "../ScrollingText.svelte";

	let { track }: { track: Track } = $props();

	const albumList = $derived(parseAlbumEntries(track.albums));
</script>

<div class="flex gap-2 p-2 border-2 rounded-md justify-between items-center">
	<div class="flex gap-2 min-w-0 flex-1">
		<ArtworkDisplay uid={track.uid} size={40} type="track" />
		<div class="min-w-0 grid">
			{#if track.path != ""}
				<button
					onclick={() => setSelection(track.uid, "track")}
					class="text-sm text-foreground truncate cursor-pointer hover:underline text-left"
				>
					<ScrollingText text={track.title} hoverOnly />
				</button>
			{:else}
				<button
					onclick={() => setSelection(track.uid, "track")}
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
							onclick={() => setSelection(getAlbumUidFromName(album.name), "album")}
							class="text-xs cursor-pointer hover:underline"
						>
							{album.name}{i < albumList.length - 1 ? "," : ""}
						</button>
					{/each}
				{/if}
			</div>
		</div>
	</div>

	<div class="flex gap-2 shrink-0">
		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "destructive", size: "icon" })}
				onclick={() => removeTrackFromLibrary(track.uid)}
			>
				<Trash />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Remove from library</p></Tooltip.Content>
		</Tooltip.Root>

		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "outline", size: "icon" })}
				onclick={() => enrichTrack(track.uid)}
			>
				<CloudDownload />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Fetch metadata via API</p></Tooltip.Content>
		</Tooltip.Root>

		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "outline", size: "icon" })}
				onclick={() => replaceTrackPath(track.uid)}
			>
				<Paperclip />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Replace file path</p></Tooltip.Content>
		</Tooltip.Root>

		{#if track.path != ""}
			<Tooltip.Root>
				<Tooltip.Trigger
					class={buttonVariants({ variant: "outline", size: "icon" })}
					onclick={() => openTrackInExplorer(track.path)}
				>
					<FolderOpen />
				</Tooltip.Trigger>
				<Tooltip.Content><p>Show in file explorer</p></Tooltip.Content>
			</Tooltip.Root>
		{/if}

		<Tooltip.Root>
			<Tooltip.Trigger
				class={buttonVariants({ variant: "outline", size: "icon" })}
				onclick={() => openEditModal({ type: "track", uid: track.uid })}
			>
				<Pencil />
			</Tooltip.Trigger>
			<Tooltip.Content><p>Edit metadata</p></Tooltip.Content>
		</Tooltip.Root>
	</div>
</div>
