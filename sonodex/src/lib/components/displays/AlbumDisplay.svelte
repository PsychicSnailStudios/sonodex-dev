<script lang="ts">

	// APP
	import { invoke } from "@tauri-apps/api/core";

	// COMPONENTS
	import { Pencil } from "lucide-svelte";

	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import Button from "$lib/components/ui/button/button.svelte";

	// CUSTOM COMPONENTS
	import TrackTableSettings from "$lib/components/app-ui/track-table/TrackTableSettings.svelte";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app-ui/track-table/TrackTable.svelte";
	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
	import NavButtons from "$lib/components/app-ui/NavButtons.svelte";

	// SCRIPTS
	import { selection, setSelection } from "$lib/ts/session.svelte";
	import { getArtistUidFromName, library, getAlbumTracks } from "$lib/ts/library.svelte";
	import { queueTracksByObject } from "$lib/ts/audio/audioManager.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { getArtworkColor, totalDuration } from "$lib/ts/util/helpers"
	import { createPersistedViewState } from "$lib/ts/session.svelte";
	
	import type { Album, Track } from "$lib/ts/util/types";

	// VARIABLES
	const view = createPersistedViewState("album", {
		sortField: "number",
		sortDir: "asc",
		colPreset: "album",
	});

	let album: Album | null = $state(null);
	let color = $state("rgb(30, 30, 30)");

	let tracks: Track[] = $derived.by(() => {
		if (!album) return [];
		return getAlbumTracks(album.uid, view.sort);
	});

	let artistAlbums: Album[] = $derived.by(() => {
		if (!album?.album_artist) return [];
		return library.albums.filter(a =>
			a.album_artist?.toLowerCase() === album?.album_artist?.toLowerCase()
		);
	});

	// APP FUNCTIONS
	$effect(() => {
		const uid = selection.uid;
		album = null;
		invoke("get_album", { uid }).then((a) => {
			album = a as Album;
		});

		invoke("get_album_artwork", { uid }).then((bytes) => {
			if (bytes) getArtworkColor(bytes as number[], 0.3).then((c) => color = c)
		});
	});
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md" style="background: linear-gradient(180deg, {color} 0%, transparent 80%)">

	{#if album}
	<ScrollArea class="min-h-0 min-w-0 h-full">
	<div class="flex flex-col gap-4 pb-4 pr-4">
		<NavButtons />

		<div class="flex gap-4 items-end">
			<ArtworkDisplay uid={album.uid} size={160} type="album" />

			<div class="flex flex-col gap-1">
				<span class="text-xs text-muted-foreground">{album.format ? album.format : "Album"}</span>
				<h2 class="text-2xl font-bold">{album.title}</h2>
				<div class="flex gap-3 text-sm text-muted-foreground flex-wrap">
					{#if album.album_artist}
						<span role="button" tabindex="0" onclick={() => setSelection(getArtistUidFromName(album.album_artist), "artist")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(getArtistUidFromName(album.album_artist)); }} class="text-sm truncate cursor-pointer hover:underline">
							{album.album_artist}
						</span>
					{/if}
					{#if album.release_date}
						<span>|</span>
						<span>{album.release_date}</span>
					{/if}
					<span>|</span>
					<span>{tracks.length} {tracks.length === 1 ? "song" : "songs"}</span>
					{#if tracks.length > 0}
						<span>|</span>
						<span>{totalDuration(tracks)}</span>
					{/if}
				</div>
			</div>
		</div>

		<div class="flex gap-2  flex-wrap">
			<Button variant="default" onclick={() => queueTracksByObject(tracks, true)}>{ tracks.length === 1 ? "Play" : "Play All"}</Button>
			{#if tracks.length > 1}
				<Button variant="outline" onclick={() => queueTracksByObject(tracks, true, true)}>Shuffle</Button>
			{/if}
			<Button variant="ghost" onclick={() => openEditModal({ type: "album", uid: album!.uid })}><Pencil /></Button>
			<TrackTableSettings cols={view.cols} sort={view.sort} compact={view.compact} onCompactChange={(v) => view.compact = v} />
		</div>

		<TrackTable tracks={tracks} columns={view.cols} sort={view.sort} compact={view.compact} />

		<div class="flex flex-col gap-2 w-full pt-4">
			<h4>More by {album.album_artist}</h4>
			<ScrollArea orientation="horizontal" class="min-h-0 min-w-0">
				<div class="grid gap-2 pb-4" style="grid-auto-columns: 150px; grid-auto-flow: column;">
					{#each artistAlbums as album}
						<AudioCard title={album.title} subTitle={album.album_artist} artworkUid={album.uid} type="album" />
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