<script lang="ts">
	
	// APP
	import { invoke } from "@tauri-apps/api/core";

	// COMPONENTS
	import { CirclePlus, Pencil } from "lucide-svelte";

	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	// CUSTOM COMPONENTS
	import TrackTableSettings from "$lib/components/app-ui/track-table/TrackTableSettings.svelte"
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app-ui/track-table/TrackTable.svelte";
	import NavButtons from "$lib/components/app-ui/NavButtons.svelte";

	// SCRIPTS
	import { selection } from "$lib/ts/session.svelte";
	import { getPlaylistTracks, library } from "$lib/ts/library.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { getArtworkColor, parseArtists, totalDuration } from '$lib/ts/util/helpers';
	import { addTrackToPlaylist, addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
	import { queueTracksByObject } from "$lib/ts/audio/audioManager.svelte";
	import { dragState, endDrag } from "$lib/ts/drag-n-drop/dragState.svelte";
	import { createPersistedViewState } from "$lib/ts/session.svelte";

	import type { Track } from "$lib/ts/util/types";

	// VARIABLES
	let search = $state("");
	let playlist = $derived(library.playlists.find(p => p.uid === selection.uid) ?? null);
	let color = $state("rgb(30, 30, 30)")

	const view = createPersistedViewState("playlist", {
		sortField: null,
		sortDir: "asc",
		colPreset: "playlist",
	});

	let tracks: Track[] = $derived.by(() => {
		if (!playlist) return [];
		return getPlaylistTracks(playlist.uid, view.sort);
	});

	const filteredTracks = $derived(
		search.trim() === ""
			? library.tracks
			: library.tracks.filter((t) => {
					const q = search.toLowerCase();
					const title = t.title?.toLowerCase() ?? "";
					const artists = t.artists?.toLowerCase() ?? "";
					const album_artist = t.album_artist?.toLowerCase() ?? "";
					const albums = t.albums ? JSON.stringify(t.albums).toLowerCase() : "";
					return title.includes(q) || artists.includes(q) || album_artist.includes(q) || albums.includes(q);
			  })
	);

	// APP FUNCTIONS
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

	// FUNCTIONS
	function handleDisplayDragOver(e: DragEvent) {
		if (dragState.active) e.preventDefault();
	}

	async function handleDisplayDrop(e: DragEvent) {
		e.preventDefault();
		if (!dragState.payload || !playlist) return;
		await addTracksToPlaylist(playlist.uid, dragState.payload.uids);
		endDrag();
	}
</script>

<div
	class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md"
	style="background: linear-gradient(180deg, {color} 0%, transparent 80%)"
	ondragover={handleDisplayDragOver}
	ondrop={handleDisplayDrop}
	aria-label="Playlist"
	role="region"
>
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
					<div class="flex gap-3 text-sm text-muted-foreground flex-wrap">
						<span>{tracks.length} songs</span>
						{#if tracks.length > 0}
							<span>|</span>
							<span>{totalDuration(tracks)}</span>
						{/if}
					</div>
					<div class="flex gap-2 flex-wrap">
						<Button variant="default" onclick={() => queueTracksByObject(tracks, true)}>Play All</Button>
						<Button variant="outline" onclick={() => queueTracksByObject(tracks, true, true)}>Shuffle</Button>
						<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "playlist", uid: playlist!.uid })}><Pencil /></Button>
						<TrackTableSettings cols={view.cols} sort={view.sort} compact={view.compact} onCompactChange={(v) => view.compact = v} />
					</div>
				</div>
			</div>

			<TrackTable tracks={tracks} columns={view.cols} sort={view.sort} compact={view.compact} playlistUid={playlist.uid} />
			
			<div class="flex flex-col gap-1 p-3 mt-8 m-4 rounded-md bg-muted">
				<div class="flex items-center justify-between p-2">
					<h3>Find More</h3>
					<Input
						placeholder="Search..."
						bind:value={search}
						class="w-48"
					/>
				</div>
				
				<ScrollArea class="min-h-0 min-w-0 h-[300px] p-2">
					<div class="flex flex-col gap-0.5">
						{#each filteredTracks as track}
							<div class="grid gap-2 p-2" style="grid-template-columns: auto auto 1fr auto;">
								<ArtworkDisplay uid={track.uid} size={30} type="track" />
								<div class="min-w-0 grid">
									<span class="text-sm font-medium truncate">{track.title}</span>
									<span class="text-xs text-muted-foreground truncate">{parseArtists(track.artists)}</span>
								</div>
								<span></span>
								<button onclick={() => { addTrackToPlaylist(playlist!, track) }}><CirclePlus size={20} /></button>
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