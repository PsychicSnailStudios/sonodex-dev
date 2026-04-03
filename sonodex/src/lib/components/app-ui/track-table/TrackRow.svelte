<script lang="ts">

	// COMPONENTS
	import { Pause, Play } from "lucide-svelte";
	import Button from "$lib/components/ui/button/button.svelte";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import TrackTableEditButton from "$lib/components/app-ui/track-table/TrackTableEditButton.svelte";
   import TrackRating from "$lib/components/app-ui/TrackRating.svelte";

	// SCRIPTS
	import { setSelection } from "$lib/ts/session.svelte";
	import { trackSelection, selectTrack } from "$lib/ts/app/trackSelection.svelte";
	import { startDrag, endDrag } from "$lib/ts/app/dragState.svelte";
	import { player, playTrackByUid, togglePlay } from "$lib/ts/audio/audioManager.svelte";
	import { getAlbumUidFromName, getArtistUidFromName } from "$lib/ts/library.svelte";
	import { formatDuration, parseAlbum, parseArtists, parseTrackNumber } from "$lib/ts/util/helpers";
	import type { Track } from "$lib/ts/util/types";

	// PROPS
	let {
		track,
		orderedUids,
		index,
		compact = false,
		gridTemplate,
		showNumber,
		showArtwork,
		showTitle,
		showArtist,
		showAlbum,
		showYear,
		showRating,
		showDuration,
		showLabel,
		showOptions,
		playlistUid = null,
	} = $props<{
		track: Track;
		orderedUids: string[];
		index: number;
		compact?: boolean;
		gridTemplate: string;
		showNumber: boolean;
		showArtwork: boolean;
		showTitle: boolean;
		showArtist: boolean;
		showAlbum: boolean;
		showYear: boolean;
		showRating: boolean;
		showDuration: boolean;
		showLabel: boolean;
		showOptions: boolean;
		playlistUid?: string | null;
	}>();

	// VARIABLES
	const isGhosted = $derived(/^[a-z]+-[0-9a-f-]{36}$/.test(track.path));
	let isSelected = $derived(trackSelection.isSelected(track.uid));
	
	// FUNCTIONS
	function handleRowClick(e: MouseEvent) {
		if ((e.target as HTMLElement).closest("button")) return;
		selectTrack(track.uid, orderedUids, e);
	}

	function handleRowDblClick(e: MouseEvent) {
		if ((e.target as HTMLElement).closest("button")) return;
		playTrackByUid(track.uid);
	}

	function handleDragStart(e: DragEvent) {
		if (!e.dataTransfer) return;

		let uids: string[];
		if (trackSelection.isSelected(track.uid) && trackSelection.count > 1) {
			uids = orderedUids.filter((uid) => trackSelection.isSelected(uid));
		} else {
			uids = [track.uid];
		}

		startDrag({ type: "tracks", uids, sourcePlaylistUid: playlistUid });
		e.dataTransfer.effectAllowed = "move";
		e.dataTransfer.setData("text/plain", uids.join(","));
	}

	function handleDragEnd() {
		endDrag();
	}

	function getTrackNumber(): string {
		let num = parseTrackNumber(track.albums) ?? "#"

		if (playlistUid) {
			num = orderedUids.indexOf(track.uid) + 1
		}

		return num.toString();
	}
</script>

<div
	role="row"
	class="grid items-center content-center px-3 border-b cursor-pointer select-none transition-colors {isSelected ? 'bg-primary/15 hover:bg-primary/20' : 'hover:bg-muted/50'} {isGhosted ? 'opacity-50 pointer-events-none cursor-not-allowed' : ''}"
	style="grid-template-columns: {gridTemplate}; height: {compact ? '28px' : '56px'};"
	onclick={handleRowClick}
	ondblclick={handleRowDblClick}
	onkeydown={(e) => { if (e.key === 'Enter') playTrackByUid(track.uid); }}
	draggable="true"
	ondragstart={handleDragStart}
	ondragend={handleDragEnd}
	tabindex="0"
>
	{#if showNumber && showArtwork}
		<span class=" cursor-auto text-sm pointer-events-none">{getTrackNumber()}</span>
	{/if}

	{#if showNumber && !showArtwork}
		<div class="group relative w-full h-full flex items-center justify-start">
			<span class="text-sm pointer-events-none group-hover:opacity-0">{getTrackNumber()}</span>
			<div class="cursor-pointer absolute top-0 -left-3 inset-0 w-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
				{#if player.isPlaying && player.track?.uid === track.uid}
					<Button variant="ghost" size="icon" onclick={() => togglePlay()}>
						<Pause />
					</Button>
				{:else if player.track?.uid === track.uid}
					<Button variant="ghost" size="icon" onclick={() => togglePlay()}>
						<Play />
					</Button>
				{:else}
					<Button variant="ghost" size="icon" onclick={() => playTrackByUid(track.uid)}>
						<Play />
					</Button>
				{/if}
			</div>
		</div>
	{/if}

	{#if !compact && showArtwork}
		<div class="group relative w-full">
			<ArtworkDisplay uid={track.uid} size={30} />
			<div class="cursor-pointer absolute top-0 inset-0 -left-1 w-full h-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
				{#if player.isPlaying && player.track?.uid === track.uid}
					<Button variant="ghost" size="icon" onclick={() => togglePlay()}>
						<Pause />
					</Button>
				{:else if !player.isPlaying && player.track?.uid === track.uid}
					<Button variant="ghost" size="icon" onclick={() => togglePlay()}>
						<Play />
					</Button>
				{:else}
					<Button variant="ghost" size="icon" onclick={() => playTrackByUid(track.uid)}>
						<Play />
					</Button>
				{/if}
			</div>
		</div>
	{/if}

	{#if showTitle}
		{#if compact}
			<div class="min-w-0 flex items-center">
				<span role="button" tabindex="0" onclick={() => setSelection(track.uid, "track")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(track.uid, "track"); }} class="text-sm truncate cursor-pointer hover:underline">
					{track.title ?? "Unknown Title"}
				</span>
			</div>
		{:else}
			<div class="flex flex-col min-w-0">
				<div class="min-w-0 flex items-center">
					<span role="button" tabindex="0" onclick={() => setSelection(track.uid, "track")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(track.uid, "track"); }} class="text-sm truncate cursor-pointer hover:underline">
						{track.title ?? "Unknown Title"}
					</span>
				</div>
				<div class="min-w-0 flex items-center">
					<span role="button" tabindex="0" onclick={() => setSelection(getArtistUidFromName(track.album_artist ?? ""), "artist")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(getArtistUidFromName(track.album_artist ?? ""), "artist"); }} class="text-xs text-muted-foreground truncate cursor-pointer hover:underline">
						{parseArtists(track.artists)}
					</span>
				</div>
			</div>
		{/if}
	{/if}

	{#if showArtist}
		<span role="button" tabindex="0" onclick={() => setSelection(getAlbumUidFromName(track.album_artist.uid), "artist")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(getAlbumUidFromName(track.album_artist.uid), "artist"); }} class="text-sm truncate cursor-pointer hover:underline">
			{parseArtists(track.artists)}
		</span>
	{/if}

	{#if showAlbum}
		<div class="min-w-0 flex items-center pr-4">
			<span role="button" tabindex="0" onclick={() => setSelection(getAlbumUidFromName(parseAlbum(track.albums)), "album")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(getAlbumUidFromName(parseAlbum(track.albums)), "album"); }} class="text-sm truncate cursor-pointer hover:underline">
				{parseAlbum(track.albums)}
			</span>
		</div>
	{/if}

	{#if showYear}
		<span class="text-sm pointer-events-none">{track.year ?? "—"}</span>
	{/if}

	{#if showRating}
		<TrackRating uid={track.uid} rating={track.rating} />
	{/if}

	{#if showDuration}
		<span class="text-sm font-mono pointer-events-none">{formatDuration(track.duration_ms)}</span>
	{/if}

	{#if showLabel}
		<span class="text-sm truncate pointer-events-none">{track.label ?? "—"}</span>
	{/if}

	{#if showOptions}
			<TrackTableEditButton {track} />
	{/if}
</div>