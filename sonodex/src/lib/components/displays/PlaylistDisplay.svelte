<script lang="ts">
	
	// APP
	import { invoke } from "@tauri-apps/api/core";

	// COMPONENTS
	import { CirclePlus, Pencil, X } from "lucide-svelte";

	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	// CUSTOM COMPONENTS
	import TrackTableSettings from "$lib/components/app-ui/track-table/TrackTableSettings.svelte"
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app-ui/track-table/TrackTable.svelte";
   import DefultPlaylistArt from "$lib/components/app-ui/playlist/DefultPlaylistArt.svelte";
	import NavButtons from "$lib/components/app-ui/NavButtons.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";
	import DownloadButton from "$lib/components/app-ui/DownloadButton.svelte";

	// SCRIPTS
	import Fuse from "fuse.js";
	import { selection } from "$lib/ts/app-states/state_session.svelte";
	import { getPlaylistTracks, library } from "$lib/ts/library.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { getArtworkColor, getArtworkColorFromPath, parseAlbum, parseArtists, totalDuration } from '$lib/ts/util/helpers';
	import { addTrackToPlaylist, addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
	import { queueTracksByObject } from "$lib/ts/audio/audioManager.svelte";
	import { dragState, endDrag } from "$lib/ts/app-states/state_drag.svelte";
	import { createPersistedViewState } from "$lib/ts/app-states/state_session.svelte";

	import type { Track } from "$lib/ts/util/types";

	// VARIABLES
	let search = $state("");
	let playlist = $derived(library.playlists.find(p => p.uid === selection.uid) ?? null);
	let color = $state("rgb(30, 30, 30)")
	let showSearch = $state(false);

	const view = createPersistedViewState("playlist", {
		sortField: null,
		sortDir: "asc",
		colPreset: "playlist",
	});

	let tracks: Track[] = $derived.by(() => {
		if (!playlist) return [];
		return getPlaylistTracks(playlist.uid, view.sort);
	});

	let fuseInstance: Fuse<(typeof library.tracks)[0]> | null = $state(null);
	let lastTracksRef: typeof library.tracks | null = null;

	function getFuse() {
		if (fuseInstance && lastTracksRef === library.tracks) return fuseInstance;
		lastTracksRef = library.tracks;
		fuseInstance = new Fuse(library.tracks, {
			keys: [
				{ name: "title",        weight: 0.5,  getFn: (t) => t.title ?? ""                    },
				{ name: "artists",      weight: 0.25, getFn: (t) => parseArtists(t.artists ?? "[]")  },
				{ name: "album_artist", weight: 0.15, getFn: (t) => t.album_artist ?? ""             },
				{ name: "albums",       weight: 0.1,  getFn: (t) => parseAlbum(t.albums ?? "[]")     },
				{ name: "tags",         weight: 0.05, getFn: (t) => t.tags ?? ""                     },
				{ name: "genres",       weight: 0.05, getFn: (t) => t.genres ?? ""                   },
			],
			threshold: 0.35,
			ignoreLocation: true,
			includeScore: false,
			useExtendedSearch: false,
			minMatchCharLength: 2,
		});
		return fuseInstance;
	}

	const filteredTracks = $derived(
		search.trim().length < 2
			? library.tracks
			: getFuse().search(search).map((r) => r.item)
	);

	// APP FUNCTIONS
	$effect(() => {
		const uid = selection.uid;
		const firstTrack = tracks[0];
		if (!uid) return;

		color = "var(--muted)";

		if (playlist?.artwork_path) {
			getArtworkColorFromPath(playlist.artwork_path, 0.3).then((c) => color = c);
			return;
		}

		invoke("get_playlist_artwork", { uid }).then((bytes) => {
			if (bytes) {
				getArtworkColor(bytes as number[], 0.3).then((c) => color = c);
			} else {
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

	let allGhosts = $derived(
		tracks.length > 0 && tracks.every(t => {
			const hasLocal = t.path && t.path !== "" && t.path !== t.uid;
			const hasRemote = t.remote_path && t.remote_path.length > 0;
			return !hasLocal && !hasRemote;
		})
	);
</script>

<div
	class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md"
	style="background: linear-gradient(180deg, {color} 0%, transparent 80%)"
	ondragover={handleDisplayDragOver}
	ondrop={handleDisplayDrop}
	aria-label="Playlist"
	role="region"
>
	<ScrollArea class="min-h-0 min-w-0">	
		<div class="flex flex-col gap-4 pb-4 pr-4 pl-1">
			<NavButtons />
			
			{#if playlist}
			<div class="flex gap-4 items-end p-0">
				<div class="drop-shadow-md">
					<ArtworkDisplay uid={playlist.uid} size={160} type="playlist">
						<DefultPlaylistArt tracks={tracks} />
					</ArtworkDisplay>
				</div>

				<div class="flex flex-col gap-2">
					<span class="text-xs text-muted-foreground">Playlist</span>
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
						<span>|</span>
						<span>{playlist.owner}</span>
					</div>
				</div>
			</div>

			<div class="flex gap-2 justify-between items-center flex-wrap p-2 rounded-md"
				  style="background: {color};">
				<div>
					<Button variant="default" disabled={allGhosts} onclick={() => queueTracksByObject(tracks, true)}>{ tracks.length === 1 ? "Play" : "Play All"}</Button>
					{#if tracks.length > 1}
					<Button variant="outline" disabled={allGhosts} onclick={() => queueTracksByObject(tracks, true, true)}>Shuffle</Button>
					{/if}
				</div>
				
				<div class="flex gap-1 items-center">
					<DownloadButton uid={playlist.uid} variant="ghost" />
					<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "playlist", uid: playlist!.uid })}><Pencil /></Button>
					<TrackTableSettings cols={view.cols} sort={view.sort} compact={view.compact} onCompactChange={(v) => view.compact = v} />
				</div>
			</div>

			<TrackTable tracks={tracks} columns={view.cols} sort={view.sort} compact={view.compact} playlistUid={playlist.uid} />
			
			<div class="flex flex-col justify-end items-end m-4 mt-8 gap-1">
			{#if showSearch}
				<Button variant="ghost" size="icon" onclick={() => showSearch = false}><X /></Button>

				<div class="flex flex-col gap-1 p-3 rounded-md bg-muted">
					<div class="flex items-center justify-between p-2">
						<h3>Find More</h3>
						<SearchBar bind:search searchCount={filteredTracks.length} />
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
			{:else}
				<Button variant="ghost" onclick={() => showSearch = true}>Find More</Button>
			{/if}
			</div>
			{:else}
				<span class="text-muted-foreground text-sm">Loading...</span>
			{/if}

		</div>
	</ScrollArea>


</div>
