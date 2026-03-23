<script lang="ts">
	import { selection } from "$lib/session.svelte";
	import { library } from "$lib/library.svelte";
	import type { Playlist, Track } from "$lib/types";
	import { openEditModal } from "$lib/editModal.svelte";

	import { CirclePlus, Pencil } from "lucide-svelte";

	import { createColumnState } from "$lib/columnConfig.svelte"
	import ColumnToggle from "$lib/components/app/ColumnToggle.svelte"

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	import { parseArtists, totalDuration } from '$lib/helpers';
    import Circle from "@lucide/svelte/icons/circle";
    import { addTrackToPlaylist } from "$lib/playlistManager.svelt";

	const cols = createColumnState("playlist");
	const searchCols = createColumnState();
	let search = $state("");

	let playlist = $derived(library.playlists.find(p => p.uid === selection.uid) ?? null);

	let tracks: Track[] = $derived.by(() => {
		if (!playlist) return [];
		const playlistTrackUids: string[] = JSON.parse(playlist.tracks ?? "[]").map((t: { uid: string }) => t.uid);
		return library.tracks.filter(t => playlistTrackUids.includes(t.uid));
	});

	console.log(playlist);

	const filteredTracks = $derived(
		search.trim() === ""
			? library.tracks
			: library.tracks.filter((t) => {
					const q = search.toLowerCase();
					const title = t.title?.toLowerCase() ?? "";
					const artists = t.artists?.toLowerCase() ?? "";
					const album_artist = t.album_artist?.toLowerCase() ?? "";
					const albums = t.albums?.toLowerCase() ?? "";
					return title.includes(q) || artists.includes(q) || album_artist.includes(q) || albums.includes(q);
			  })
	);
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
					<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "playlist", uid: playlist!.uid })}><Pencil /></Button>
					<ColumnToggle columns={cols} />
				</div>
			</div>
		</div>

		<TrackTable tracks={tracks} columns={cols} />
		
		<div class="flex items-center justify-between p-2">
			<h3>Find More</h3>
			<Input
				placeholder="Search..."
				bind:value={search}
				class="w-48"
			/>
		</div>
		
		<ScrollArea class="min-h-0 min-w-0 h-[300px]">
			<div class="flex flex-col gap-0.5">
				{#each filteredTracks as track}
					<div class="grid gap-2 p-2" style="grid-template-columns: auto auto 1fr auto;">
						<ArtworkDisplay uid={track.uid} size={30} type="track" />
						<div class="min-w-0 grid">
							<span class="text-sm font-medium truncate">{track.title}</span>
							<span class="text-xs text-muted-foreground truncate">{parseArtists(track.artists)}</span>
						</div>
						<span></span>
						<button class="" onclick={() => { addTrackToPlaylist(playlist, track)}}><CirclePlus size={20} /></button>
					</div>
				{/each}
			</div>
		</ScrollArea>

	{:else}
		<span class="text-muted-foreground text-sm">Loading...</span>
	{/if}

</div>
