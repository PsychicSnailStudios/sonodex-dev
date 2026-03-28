<script lang="ts">
	import type { Track } from "$lib/ts/util/types";
	import { setSelection } from "$lib/session.svelte";
	import { trackSelection, selectTrack } from "$lib/ts/app/trackSelection.svelte";
	import { startDrag, endDrag } from "$lib/ts/app/dragState.svelte";
	import { playTrackByUid } from "$lib/ts/audio/audioManager.svelte";
	import { getAlbumUidFromName, getArtistUidFromName } from "$lib/library.svelte";
	import { formatDuration, formatRating, parseAlbum, parseArtists, parseTrackNumber } from "$lib/ts/util/helpers";
	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTableEditButton from "$lib/components/app/TrackTableEditButton.svelte";
    import { get } from "svelte/store";

	let {
		track,
		orderedUids,
		index,
		compact = false,
		gridTemplate,
		showNumber,
		showArtwork,
		showTitle,
		showAlbum,
		showYear,
		showRating,
		showDuration,
		showLabel,
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
		showAlbum: boolean;
		showYear: boolean;
		showRating: boolean;
		showDuration: boolean;
		showLabel: boolean;
		playlistUid?: string | null;
	}>();

	let isSelected = $derived(trackSelection.isSelected(track.uid));

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
	class="grid items-center px-3 border-b cursor-pointer select-none transition-colors {isSelected ? 'bg-primary/15 hover:bg-primary/20' : 'hover:bg-muted/50'}"
	style="grid-template-columns: {gridTemplate}; height: {compact ? '25px' : '56px'};"
	onclick={handleRowClick}
	ondblclick={handleRowDblClick}
	onkeydown={(e) => { if (e.key === 'Enter') playTrackByUid(track.uid); }}
	draggable="true"
	ondragstart={handleDragStart}
	ondragend={handleDragEnd}
	tabindex="0"
>
	{#if showNumber}
		<span class="text-sm pointer-events-none">{getTrackNumber()}</span>
	{/if}

	{#if !compact && showArtwork}
		<button onclick={() => playTrackByUid(track.uid)} class="flex items-center">
			<ArtworkDisplay uid={track.uid} size={30} />
		</button>
	{/if}

	{#if showTitle}
		{#if compact}
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
		<span class="text-sm font-mono pointer-events-none">{formatRating(track.rating)}</span>
	{/if}

	{#if showDuration}
		<span class="text-sm font-mono pointer-events-none">{formatDuration(track.duration_ms)}</span>
	{/if}

	{#if showLabel}
		<span class="text-sm truncate pointer-events-none">{track.label ?? "—"}</span>
	{/if}

	<TrackTableEditButton {track} />
</div>