<script lang="ts">
	import { selection } from "$lib/session.svelte";
	import { library } from "$lib/library.svelte";
	import type { Playlist, Track } from "$lib/types";
	import { openEditModal } from "$lib/editModal.svelte";

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";
	import { Button } from "$lib/components/ui/button/index.js";

	import { totalDuration } from '$lib/helpers';

	let playlist = $derived(library.playlists.find(p => p.uid === selection.uid) ?? null);

	let tracks: Track[] = $derived.by(() => {
		if (!playlist) return [];
		const playlistTrackUids: string[] = JSON.parse(playlist.tracks ?? "[]").map((t: { uid: string }) => t.uid);
		return library.tracks.filter(t => playlistTrackUids.includes(t.uid));
	});
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	{#if playlist}
		<div class="flex gap-4 items-end">
			<ArtworkDisplay uid={playlist.uid} size={160} type="playlist" />

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
					<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "playlist", uid: playlist!.uid })}>⋯</Button>
				</div>
			</div>
		</div>

		<TrackTable type="playlist" tracks={tracks} />
	{:else}
		<span class="text-muted-foreground text-sm">Loading...</span>
	{/if}

</div>
