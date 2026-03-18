<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	import Button from "../ui/button/button.svelte";

	import { selection } from "$lib/session.svelte";
	import { library } from "$lib/library.svelte";
	import type { Album, Track } from "$lib/types";
	import { QueueTracksById, QueueTracksByObject } from "$lib/audioManager.svelte";
	import { openEditModal } from "$lib/editModal.svelte";

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";
	
	import { formatDuration, totalDuration } from '$lib/helpers';

	let album: Album | null = $state(null);

	let tracks: Track[] = $derived.by(() => {
		if (!album) return [];
		const albumTrackIds: number[] = JSON.parse(album.tracks ?? "[]").map((t: { id: number }) => t.id);
		return library.tracks.filter(t => albumTrackIds.includes(t.id));
	});


	$effect(() => {
		const id = selection.id;
		album = null;
		invoke("get_album", { id }).then((a) => {
			album = a as Album;
		});
	});
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	{#if album}
		<div class="flex gap-4 items-end">
			<ArtworkDisplay id={album.id} size={160} type="album" />

			<div class="flex flex-col gap-1">
				{#if album.format}
					<span class="text-xs text-muted-foreground">{album.format}</span>
				{/if}
				<h2 class="text-2xl font-bold">{album.title}</h2>
				<div class="flex gap-3 text-sm text-muted-foreground">
					{#if album.album_artist}
						<span>{album.album_artist}</span>
					{/if}
					{#if album.release_date}
						<span>{album.release_date}</span>
					{/if}
					<span>{tracks.length} songs</span>
					{#if tracks.length > 0}
						<span>{totalDuration(tracks)}</span>
					{/if}
				</div>
			</div>
		</div>

		<div class="flex gap-2">
			<Button variant="default" onclick={() => QueueTracksByObject(tracks, true)}>Play All</Button>
			<Button variant="outline">Shuffle</Button>
			<Button variant="ghost" onclick={() => openEditModal({ type: "album", id: album!.id })}>...</Button>
		</div>

		<TrackTable type="album" tracks={tracks} />
	{:else}
		<span class="text-muted-foreground text-sm">Loading...</span>
	{/if}

</div>