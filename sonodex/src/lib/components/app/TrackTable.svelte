<script lang="ts">
	import type { Track } from "$lib/ts/util/types"
	import type { ColumnState } from "$lib/ts/app/columnConfig.svelte"
	import type { SortState } from "$lib/ts/app/sortConfig.svelte"
	import { setSelection } from "$lib/session.svelte"
	import { Clock2, Star, ChevronUp, ChevronDown, ChevronsUpDown } from "lucide-svelte"
	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte"
	import { playTrackByUid } from "$lib/ts/audio/audioManager.svelte"
	import { getAlbumUidFromName, getArtistUidFromName } from "$lib/library.svelte"
	import { formatDuration, formatRating, parseAlbum, parseArtists, parseTrackNumber } from "$lib/ts/util/helpers"
	import TrackTableEditButton from "$lib/components/app/TrackTableEditButton.svelte"

	let { tracks, columns, sort, compact = false  } = $props<{ tracks: Track[]; columns: ColumnState; sort: SortState, compact?: boolean }>()

	const v = $derived(columns.visible)

	const sortedTracks = $derived.by(() => {
		if (!sort.field || !sort.direction) return tracks

		const dir = sort.direction === "asc" ? 1 : -1

		return [...tracks].sort((a, b) => {
			switch (sort.field) {
				case "title":
					return dir * (a.title ?? "").localeCompare(b.title ?? "")
				case "album":
					return dir * (parseAlbum(a.albums) ?? "").localeCompare(parseAlbum(b.albums) ?? "")
				case "year":
					return dir * ((a.year ?? "").localeCompare(b.year ?? ""))
				case "rating":
					return dir * ((a.rating ?? -1) - (b.rating ?? -1))
				case "duration":
					return dir * ((a.duration_ms ?? 0) - (b.duration_ms ?? 0))
				case "label":
					return dir * (a.label ?? "").localeCompare(b.label ?? "")
				case "artist":
					return dir * (a.album_artist ?? "").localeCompare(b.album_artist ?? "")
				case "number":
					return dir * ((parseTrackNumber(a.albums) ?? 0) - (parseTrackNumber(b.albums) ?? 0))
				default:
					return 0
			}
		})
	})

	const gridTemplate = $derived(() => {
		const parts: string[] = []
		if (v.number)		parts.push("40px")
		if (v.artwork)		parts.push("40px")
		if (v.title)		parts.push("1fr")
		if (v.album)		parts.push("1fr")
		if (v.year)			parts.push("60px")
		if (v.rating)		parts.push("60px")
		if (v.duration)		parts.push("80px")
		if (v.label)		parts.push("100px")
		parts.push("40px")
		return parts.join(" ")
	})
</script>

<div class="flex flex-col p-0">
	<div
		class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b"
		style="grid-template-columns: {gridTemplate()};"
	>
		{#if v.number}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("number")}
			>
				#
				{#if sort.active("number")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.artwork}<span></span>{/if}
		{#if v.title}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("title")}
			>
				Title
				{#if sort.active("title")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if compact && v.title}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("artist")}
			>
				Artist
				{#if sort.active("artist")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.album}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("album")}
			>
				Album
				{#if sort.active("album")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.year}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("year")}
			>
				Year
				{#if sort.active("year")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.rating}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("rating")}
			>
				<Star size={14} />
				{#if sort.active("rating")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.duration}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("duration")}
			>
				<Clock2 size={14} />
				{#if sort.active("duration")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.label}
			<button
				class="flex items-center gap-1 hover:text-foreground transition-colors"
				onclick={() => sort.cycle("label")}
			>
				Label
				{#if sort.active("label")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		<span></span>
	</div>

	<div>
		{#each sortedTracks as track (track.uid)}
		{#if compact}
			<div
				class="grid items-center px-3 border-b hover:bg-muted/50"
				style="grid-template-columns: {gridTemplate()}; height: 25px;"
			>
				{#if v.number}
					<span class="text-sm">{parseTrackNumber(track.albums) ?? "#"}</span>
				{/if}
				{#if v.title}
					<button onclick={() => setSelection(track.uid, "track")} class="text-sm truncate text-left">
						{track.title ?? "Unknown Title"}
					</button>
				{/if}
				{#if v.title}
					<button onclick={() => setSelection(getArtistUidFromName(track.album_artist!), "artist")} class="text-xs text-muted-foreground truncate text-left">
						{parseArtists(track.artists)}
					</button>
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
		{:else}
			<div
				class="grid items-center px-3 border-b hover:bg-muted/50"
				style="grid-template-columns: {gridTemplate()}; height: 56px;"
			>
				{#if v.number}
					<span class="text-sm">{parseTrackNumber(track.albums) ?? "#"}</span>
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
		{/if}
		{/each}
	</div>
</div>