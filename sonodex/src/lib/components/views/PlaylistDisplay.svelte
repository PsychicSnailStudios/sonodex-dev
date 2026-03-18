<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	import { selection } from "$lib/session.svelte";
	import { library } from "$lib/library.svelte";
	import type { Playlist, Track } from "$lib/types";
	import { openEditModal } from "$lib/editModal.svelte";

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";
	import { Button } from "$lib/components/ui/button/index.js";

	import { formatDuration, totalDuration } from '$lib/helpers';

	let playlist: Playlist | null = $state(null);

	let tracks: Track[] = $derived.by(() => {
		if (!playlist) return [];
		const playlistTrackIds: number[] = JSON.parse(playlist.tracks ?? "[]").map((t: { id: number }) => t.id);
		return library.tracks.filter(t => playlistTrackIds.includes(t.id));
	});

	$effect(() => {
		const id = selection.id;
		playlist = null;
		invoke("get_playlist", { id }).then((p) => {
			playlist = p as Playlist;
		});
	});
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	{#if playlist}
		<div class="flex gap-4 items-end">
			<ArtworkDisplay id={playlist.id} size={160} type="playlist" />

			<div class="flex flex-col gap-2">
				<h2 class="text-2xl font-bold">{playlist.title}</h2>
				{#if playlist.description}
					<span class="text-sm text-muted-foreground">{playlist.description}</span>
				{/if}
				<div class="flex gap-3 text-sm text-muted-foreground">
					<span>{tracks.length} songs</span>
					{#if tracks.length > 0}
						<span>{totalDuration(tracks)}</span>
					{/if}
				</div>
				<div class="flex gap-2">
					<Button variant="default">Play All</Button>
					<Button variant="outline">Shuffle</Button>
					<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "playlist", id: playlist!.id })}>⋯</Button>
				</div>
			</div>
		</div>

		<TrackTable type="playlist" tracks={tracks} />
	{:else}
		<span class="text-muted-foreground text-sm">Loading...</span>
	{/if}

</div>