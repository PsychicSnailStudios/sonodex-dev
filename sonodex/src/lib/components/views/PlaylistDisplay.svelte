<script lang="ts">
	import { CirclePlus, Pencil, X } from "lucide-svelte";

	import { Button } from "$shadcn/button/index.js";
	import { ScrollArea } from "$shadcn/scroll-area/index.js";

	import TrackTableSettings from "$lib/components/custom/track-table/TrackTableSettings.svelte";
	import ArtworkDisplay from "$lib/components/custom/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/custom/track-table/TrackTable.svelte";
	import DefultPlaylistArt from "$lib/components/pages/playlist/DefultPlaylistArt.svelte";
	import NavButtons from "$lib/components/custom/NavButtons.svelte";
	import SearchBar from "$lib/components/custom/search/SearchBar.svelte";
	import DownloadButton from "$lib/components/custom/DownloadButton.svelte";

	import { selection } from "$ts/store/session.svelte";
	import { getTrackArrayFromUID, library } from "$ts/store/library.svelte";
	import { openEditModal } from "$ts/ui/editModal.svelte";
	import { getArtworkColor, getArtworkColorFromPath, totalDuration } from "$ts/util/helpers";
	import { queueTracksByObject } from "$ts/audio/audioManager.svelte";
	import { dragState, endDrag } from "$ts/store/drag.svelte";
	import { createPersistedViewState } from "$ts/store/session.svelte";
	import { searchTracks } from "$ts/store/fuseStore.svelte";
	import { artworkCache } from "$ts/library/artworkLoader";

	import type { Track } from "$ts/util/types";
   import { addTracksToPlaylist } from "$ts/audio/playlistManager.svelte";
    import { parseArtistsToString } from "$ts/util/parsers";

	let search = $state("");
	let playlist = $derived(library.playlists.find(p => p.uid === selection.uid) ?? null);
	let color = $state("rgb(30, 30, 30)");
	let showSearch = $state(false);

	const view = createPersistedViewState("playlist", {
		sortField: null,
		sortDir: "asc",
		colPreset: "playlist",
	});

	let tracks: Track[] = $derived.by(() => {
		if (!playlist) return [];
		return getTrackArrayFromUID(playlist.uid, view.sort);
	});

	let filteredTracks = $derived(searchTracks(search));

	$effect(() => {
		const uid = selection.uid;
		const firstTrack = tracks[0];
		if (!uid) return;

		color = "var(--muted)";

		if (playlist?.artwork_path) {
			getArtworkColorFromPath(playlist.artwork_path, 0.3).then((c) => color = c);
			return;
		}

		const cached = artworkCache.get(`playlist:${uid}`);
		if (cached) {
			fetch(cached).then(r => r.arrayBuffer()).then(buf => {
				getArtworkColor(Array.from(new Uint8Array(buf)), 0.3).then(c => color = c);
			}).catch(() => {
				if (!firstTrack) return;
				const trackCached = artworkCache.get(`track:${firstTrack.uid}`);
				if (trackCached) {
					fetch(trackCached).then(r => r.arrayBuffer()).then(buf => {
						getArtworkColor(Array.from(new Uint8Array(buf)), 0.3).then(c => color = c);
					}).catch(() => {});
				}
			});
		}
	});

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
			const hasRemote = (t as any).remote_path && (t as any).remote_path.length > 0;
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
										<span class="text-xs text-muted-foreground truncate">{parseArtistsToString(track.artists)}</span>
									</div>
									<span></span>
									<button onclick={() => { addTracksToPlaylist(playlist, track) }}><CirclePlus size={20} /></button>
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