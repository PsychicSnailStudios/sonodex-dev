<script lang="ts">
	import { selection } from "$lib/session.svelte";
	import { library } from "$lib/library.svelte";
	import type { Playlist, Track } from "$lib/types";
	import { openEditModal } from "$lib/editModal.svelte";

	import { CirclePlus, Pencil } from "lucide-svelte";

	import { createColumnState } from "$lib/columnConfig.svelte"
	import TrackTableSettings from "$lib/components/app/TrackTableSettings.svelte"

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	import { getArtworkColor, parseArtists, totalDuration } from '$lib/helpers';
   import Circle from "@lucide/svelte/icons/circle";
   import { addTrackToPlaylist } from "$lib/playlistManager.svelte";
   import { queueTracksByObject } from "$lib/audioManager.svelte";
   import { invoke } from "@tauri-apps/api/core";
	import { SortState } from "$lib/sortConfig.svelte"
   import NavButtons from "$lib/components/app/NavButtons.svelte";
	 
	const cols = createColumnState("playlist");
	const sort = new SortState("number", "asc");
	const searchCols = createColumnState();
	let search = $state("");
	let compact = $state(false);

	let playlist = $derived(library.playlists.find(p => p.uid === selection.uid) ?? null);

	let tracks: Track[] = $derived.by(() => {
		if (!playlist) return [];
		const playlistTrackUids: string[] = JSON.parse(playlist.tracks ?? "[]").map((t: { uid: string }) => t.uid);
		return library.tracks.filter(t => playlistTrackUids.includes(t.uid));
	});

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

	let color = $state("rgb(30, 30, 30)")

	$effect(() => {
		const uid = selection.uid;
		if (!uid) return;
		invoke("get_playlist_artwork", { uid }).then((bytes) => {
			if (bytes) {
				getArtworkColor(bytes as number[], 0.3).then((c) => color = c);
			} else {
				const firstTrack = tracks[0];
				if (!firstTrack) return;
				invoke("get_track_artwork", { uid: firstTrack.uid }).then((trackBytes) => {
					if (trackBytes) getArtworkColor(trackBytes as number[], 0.3).then((c) => color = c);
				});
			}
		});
	});
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md" style="background: linear-gradient(180deg, {color} 0%, transparent 80%)">
	{#if playlist}
	<ScrollArea class="min-h-0 min-w-0">	
		<div class="flex flex-col gap-4 pb-4 pr-4">
			<NavButtons />
			
			<div class="flex gap-4 items-end p-0">
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
						<Button variant="default" onclick={() => queueTracksByObject(tracks, true)}>Play All</Button>
						<Button variant="outline" onclick={() => queueTracksByObject(tracks, true, true)}>Shuffle</Button>
						<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "playlist", uid: playlist!.uid })}><Pencil /></Button>
						<TrackTableSettings columns={cols} sort={sort} bind:compact />
					</div>
				</div>
			</div>

			<TrackTable tracks={tracks} columns={cols} sort={sort} compact={compact} />
			
			<div class="flex flex-col gap-1 p-3 mt-8 m-4 rounded-md bg-accent">
				<div class="flex items-center justify-between p-2">
					<h3>Find More</h3>
					<Input
						placeholder="Search..."
						bind:value={search}
						class="w-48"
					/>
				</div>
				
				<ScrollArea class="min-h-0 min-w-0 h-[300px] p-2 ">
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
			</div>

		</div>
	</ScrollArea>

	{:else}
		<span class="text-muted-foreground text-sm">Loading...</span>
	{/if}

</div>
