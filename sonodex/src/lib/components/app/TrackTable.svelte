<script lang="ts">
	import type { Track } from "$lib/types"
	import type { ColumnState } from "$lib/columnConfig.svelte"
	import { setSelection } from "$lib/session.svelte"
	import { Clock2, Star } from "lucide-svelte"
	import { Button } from "$lib/components/ui/button/index.js"
	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte"
	import { playTrackByUid } from "$lib/audioManager.svelte"
	import { getAlbumUidFromName, getArtistUidFromName } from "$lib/library.svelte"
	import { formatDuration, formatRating, parseAlbum, parseArtists } from "$lib/helpers"
    import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
    import TrackTableEditButton from "$lib/components/app/TrackTableEditButton.svelte";

	let { tracks, columns } = $props<{ tracks: Track[]; columns: ColumnState }>()

	const v = $derived(columns.visible)

	const gridTemplate = $derived(() => {
		const parts: string[] = []
		if (v.number)	parts.push("40px")
		if (v.artwork)	parts.push("40px")
		if (v.title)	parts.push("1fr")
		if (v.album)	parts.push("1fr")
		if (v.year)		parts.push("60px")
		if (v.rating)	parts.push("60px")
		if (v.duration)	parts.push("80px")
		if (v.label)	parts.push("100px")
		parts.push("40px")
		return parts.join(" ")
	})
</script>

<div class="flex flex-col p-0">
	<div
		class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b"
		style="grid-template-columns: {gridTemplate()};"
	>
		{#if v.number}		<span>#</span>								{/if}
		{#if v.artwork}	<span></span>								{/if}
		{#if v.title}		<span>Title</span>						{/if}
		{#if v.album}		<span>Album</span>						{/if}
		{#if v.year}		<span>Year</span>							{/if}
		{#if v.rating}		<span><Star size={14} /></span>		{/if}
		{#if v.duration}	<span><Clock2 size={14} /></span>	{/if}
		{#if v.label}		<span>Label</span>						{/if}
		<span></span>
	</div>

	<div class="">
		{#each tracks as track (track.uid)}
			<div
				class="grid items-center px-3 border-b hover:bg-muted/50"
				style="grid-template-columns: {gridTemplate()}; height: 56px;"
			>
				{#if v.number}
					<span class="text-sm">{track.number ?? "#"}</span>
				{/if}
				{#if v.artwork}
					<button onclick={() => playTrackByUid(track.uid)}>
						<ArtworkDisplay uid={track.uid} size={30} />
					</button>
				{/if}
				{#if v.title}
					<div class="flex flex-col min-w-0">
						<button onclick={() => setSelection(track.uid, "track")} class="text-sm truncate text-left">
							{track.title ?? "Unknown Title"}
						</button>
						<button onclick={() => setSelection(getArtistUidFromName(track.album_artist!), "artist")} class="text-xs text-muted-foreground truncate text-left">
							{parseArtists(track.artists)}
						</button>
					</div>
				{/if}
				{#if v.album}
					<button onclick={() => setSelection(getAlbumUidFromName(parseAlbum(track.albums)), "album")} class="text-sm truncate pr-4 text-left">
						{parseAlbum(track.albums)}
					</button>
				{/if}
				{#if v.year}
					<span class="text-sm">{track.year ?? "—"}</span>
				{/if}
				{#if v.rating}
					<span class="text-sm font-mono">{formatRating(track.rating)}</span>
				{/if}
				{#if v.duration}
					<span class="text-sm font-mono">{formatDuration(track.duration_ms)}</span>
				{/if}
				{#if v.label}
					<span class="text-sm truncate">{track.label ?? "—"}</span>
				{/if}
				<TrackTableEditButton track={track} />
			</div>
		{/each}
	</div>
</div>