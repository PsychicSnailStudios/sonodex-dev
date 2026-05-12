<script lang="ts">
	// COMPONENTS
	import { Clock2, Star, ChevronUp, ChevronDown, ChevronsUpDown } from "lucide-svelte"
	import * as ContextMenu from "$shadcn/context-menu/index.js";

	// CUSTOM COMPONENTS
	import TrackRow from "$lib/components/custom/track-table/TrackRow.svelte"
	import TrackContext from "$lib/components/context-menus/TrackContext.svelte";

	// SCRIPTS
	import { dragState, endDrag } from "$ts/store/drag.svelte"
	import { generateViewId, trackSelection, setTrackSelectionContext, clearTrackSelection, copySelectedToClipboard } from "$ts/store/trackSelection.svelte"
	import { removeTracksFromPlaylist, reorderPlaylistTracks, addTracksToPlaylist, parseTracks } from "$ts/audio/playlistManager.svelte"
	import { parseDiscNumber, buildDiscBreaks, type DiscBreakEntry } from "$ts/util/discHelpers";

	// TYPES
	import type { ColumnState } from "$ts/util/columnConfig.svelte"
	import type { SortState } from "$ts/util/sortConfig.svelte"
	import type { Track } from "$ts/util/types"
	import { library } from "$ts/store/library.svelte";

	// PROPS
	let { tracks, columns, sort, compact = false, playlistUid = null, albumUid = null, emulateType = null } = $props<{
		tracks: Track[];
		columns: ColumnState;
		sort: SortState;
		compact?: boolean;
		playlistUid?: string | null;
		albumUid?: string | null;
		emulateType?: string | null;
	}>();

	// VARIABLES
	const viewId = generateViewId()
	const v = $derived(columns.visible)

	const sortedTracks = $derived.by(() => {
		const dir = sort.direction === "asc" ? 1 : -1
		const hasSort = !!(sort.field && sort.direction)

		function sortGroup(group: Track[]): Track[] {
			if (!hasSort) return group
			return [...group].sort((a, b) => {
				switch (sort.field) {
					case "title":    return dir * (a.title ?? "").localeCompare(b.title ?? "")
					case "album":    return dir * ((a.albums?.[0]?.name ?? "").localeCompare(b.albums?.[0]?.name ?? ""))
					case "year":     return dir * ((a.year ?? "").localeCompare(b.year ?? ""))
					case "rating":   return dir * ((a.rating ?? -1) - (b.rating ?? -1))
					case "duration": return dir * ((a.duration_ms ?? 0) - (b.duration_ms ?? 0))
					case "label":    return dir * (a.label ?? "").localeCompare(b.label ?? "")
					case "artist":   return dir * (a.album_artist?.name ?? "").localeCompare(b.album_artist?.name ?? "")
					case "number":   return dir * sortByNumber(a, b)
					default:         return 0
				}
			})
		}

		if (!albumUid) return sortGroup(tracks)

		const noDisc: Track[] = []
		const discMap = new Map<number, Track[]>()

		for (const track of tracks) {
			const disc = parseDiscNumber(track.albums, albumUid)
			if (disc == null) {
				noDisc.push(track)
			} else {
				if (!discMap.has(disc)) discMap.set(disc, [])
				discMap.get(disc)!.push(track)
			}
		}

		const sortedDiscKeys = [...discMap.keys()].sort((a, b) => a - b)

		return [
			...sortGroup(noDisc),
			...sortedDiscKeys.flatMap(d => sortGroup(discMap.get(d)!))
		]
	})

	const discBreaks = $derived.by(() => {
		if (!albumUid) return new Map<string, DiscBreakEntry>()
		return buildDiscBreaks(sortedTracks, albumUid, emulateType)
	})

	const gridTemplate = $derived.by(() => {
		const parts: string[] = []
		if (v.number)            	parts.push("40px")
		if (v.artwork && !compact) parts.push("40px")
		if (v.title)             	parts.push("1fr")
		if (v.artist)				parts.push("1fr")
		if (v.album)             	parts.push("1fr")
		if (v.year)              	parts.push("60px")
		if (v.rating)            	parts.push("60px")
		if (v.duration)          	parts.push("50px")
		if (v.label)             	parts.push("100px")
		if (v.options)           	parts.push("30px")
		return parts.join(" ")
	})

	const orderedUids = $derived(sortedTracks.map((t) => t.uid))

	// VIRTUALIZATION
	const ROW_HEIGHT = $derived(compact ? 28 : 56)
	const OVERSCAN = 10
	let scrollTop = $state(0)
	let containerHeight = $state(600)
	let scrollContainer = $state<HTMLElement | null>(null)

	const visibleRange = $derived.by(() => {
		const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN)
		const end = Math.min(sortedTracks.length, Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + OVERSCAN)
		return { start, end }
	})

	const totalHeight = $derived(sortedTracks.length * ROW_HEIGHT)
	const offsetY = $derived(visibleRange.start * ROW_HEIGHT)

	// disable virtualization when disc breaks are present (album view) — list is small enough
	const useVirtualization = $derived(!albumUid && sortedTracks.length > 200)

	let dragOverIndex = $state<number | null>(null)
	let dragOverPosition = $state<"above" | "below">("below")

	// APP FUNCTIONS
	$effect(() => {
		setTrackSelectionContext(playlistUid ? "playlist" : "library", playlistUid ?? null)
	})

	$effect(() => {
		if (!scrollContainer) return
		const ro = new ResizeObserver((entries) => {
			containerHeight = entries[0].contentRect.height
		})
		ro.observe(scrollContainer)
		return () => ro.disconnect()
	})

	// FUNCTIONS
	function sortByNumber(a: Track, b: Track) {
		if (playlistUid) return 0
		return (a.albums?.[0]?.track_number ?? 0) - (b.albums?.[0]?.track_number ?? 0)
	}

	function handleTableClick(e: MouseEvent) {
		if ((e.target as HTMLElement) === e.currentTarget) clearTrackSelection(viewId)
	}

	async function handleKeyDown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			clearTrackSelection(viewId)
			return
		}
		if ((e.ctrlKey || e.metaKey) && e.key === "c") {
			if (trackSelection.count === 0) return;
			e.preventDefault();
			copySelectedToClipboard(orderedUids);
			return;
		}
		if (e.key === "Delete" && playlistUid && trackSelection.count > 0) {
			e.preventDefault()
			removeTracksFromPlaylist(playlistUid, [...trackSelection.selected])
			clearTrackSelection(viewId)
			return
		}
		if ((e.ctrlKey || e.metaKey) && e.key === "v") {
			if (!playlistUid) return;
			e.preventDefault();
			const uids = trackSelection.clipboardUids;
			if (uids.length === 0) return;
			await addTracksToPlaylist(playlistUid, uids);
			return;
		}
	}

	function handleRowDragOver(e: DragEvent, index: number) {
		e.preventDefault()
		if (e.dataTransfer) e.dataTransfer.dropEffect = "move"
		if (!playlistUid) return
		const target = e.currentTarget as HTMLElement
		const rect = target.getBoundingClientRect()
		dragOverIndex = index
		dragOverPosition = e.clientY < rect.top + rect.height / 2 ? "above" : "below"
	}

	function handleRowDragLeave() {
		dragOverIndex = null
	}

	async function handleRowDrop(e: DragEvent, dropIndex: number) {
		e.preventDefault();
		if (!playlistUid) return;
		const raw = e.dataTransfer?.getData("text/plain");
		if (!raw) return;
		const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
		const sourcePlaylistUid = dragState.payload?.sourcePlaylistUid ?? null;

		if (sourcePlaylistUid === playlistUid) {
			const movingSet = new Set(uids);
			const without = orderedUids.filter((uid) => !movingSet.has(uid));
			const anchor = orderedUids[dropIndex];
			const anchorIndexInWithout = without.indexOf(anchor);
			const insertAt = dragOverPosition === "above" ? anchorIndexInWithout : anchorIndexInWithout + 1;
			without.splice(insertAt, 0, ...uids);
			await reorderPlaylistTracks(playlistUid, without);
		} else {
			await addTracksToPlaylist(playlistUid, uids);

			const playlist = library.playlists.find((p) => p.uid === playlistUid);
			if (!playlist) { dragOverIndex = null; endDrag(); return; }
			const current = parseTracks(playlist.tracks);
			const currentUids = current.sort((a, b) => a.order - b.order).map((t) => t.uid);

			const movingSet = new Set(uids);
			const without = currentUids.filter((uid) => !movingSet.has(uid));
			const anchor = orderedUids[dropIndex];
			const anchorIndexInWithout = without.indexOf(anchor);
			const insertAt = dragOverPosition === "above" ? anchorIndexInWithout : anchorIndexInWithout + 1;
			without.splice(insertAt, 0, ...uids);
			await reorderPlaylistTracks(playlistUid, without);
		}

		dragOverIndex = null;
		endDrag();
	}

	async function handleTableDrop(e: DragEvent) {
		e.preventDefault()
		dragOverIndex = null
		if (!playlistUid || !dragState.payload) return
		const { uids, sourcePlaylistUid } = dragState.payload
		if (sourcePlaylistUid !== playlistUid) {
			await addTracksToPlaylist(playlistUid, uids)
		}
		endDrag()
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

<div
	role="grid"
	aria-label="Track list"
	tabindex="-1"
	class="flex flex-col p-0 outline-none h-full"
	onclick={handleTableClick}
	onkeydown={handleKeyDown}
	ondragover={(e) => e.preventDefault()}
	ondrop={handleTableDrop}
>
	<div
		class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b shrink-0"
		style="grid-template-columns: {gridTemplate};"
	>
		{#if v.number}
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("number")}>
				#
				{#if sort.active("number")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.artwork && !compact}<span></span>{/if}
		{#if v.title}
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("title")}>
				Title
				{#if sort.active("title")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.artist}
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("artist")}>
				Artist
				{#if sort.active("artist")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.album}
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("album")}>
				Album
				{#if sort.active("album")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.year}
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("year")}>
				Year
				{#if sort.active("year")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.rating}
			<button class="flex items-center justify-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("rating")}>
				<Star size={14} />
				{#if sort.active("rating")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.duration}
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("duration")}>
				<Clock2 size={14} />
				{#if sort.active("duration")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.label}
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("label")}>
				Label
				{#if sort.active("label")}
					{#if sort.direction === "asc"}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
				{:else}
					<ChevronsUpDown size={12} class="opacity-40" />
				{/if}
			</button>
		{/if}
		{#if v.options}
			<span></span>
		{/if}
	</div>

	{#if useVirtualization}
		<div
			bind:this={scrollContainer}
			class="overflow-y-auto flex-1 min-h-0"
			onscroll={(e) => { scrollTop = (e.currentTarget as HTMLElement).scrollTop }}
		>
			<div style="height: {totalHeight}px; position: relative;">
				<div style="position: absolute; top: {offsetY}px; left: 0; right: 0;">
					{#each sortedTracks.slice(visibleRange.start, visibleRange.end) as track, localI (track.uid)}
						{@const i = visibleRange.start + localI}
						<div
							class="relative"
							role="row"
							tabindex={i}
							ondragover={(e) => handleRowDragOver(e, i)}
							ondragleave={handleRowDragLeave}
							ondrop={(e) => handleRowDrop(e, i)}
						>
							{#if dragOverIndex === i && dragOverPosition === "above"}
								<div class="absolute top-0 left-0 right-0 h-0.5 bg-primary z-10 pointer-events-none"></div>
							{/if}

							<ContextMenu.Root>
								<ContextMenu.Trigger>
									<TrackRow
										{track}
										{orderedUids}
										index={i}
										{compact}
										{gridTemplate}
										{viewId}
										showNumber={v.number}
										showArtwork={v.artwork}
										showTitle={v.title}
										showArtist={v.artist}
										showAlbum={v.album}
										showYear={v.year}
										showRating={v.rating}
										showDuration={v.duration}
										showLabel={v.label}
										showOptions={v.options}
										{playlistUid}
									/>
								</ContextMenu.Trigger>
								<TrackContext track={track} />
							</ContextMenu.Root>

							{#if dragOverIndex === i && dragOverPosition === "below"}
								<div class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary z-10 pointer-events-none"></div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		</div>
	{:else}
		<div>
			{#each sortedTracks as track, i (track.uid)}
				{#if discBreaks.has(track.uid)}
					{@const entry = discBreaks.get(track.uid)!}
					<div class="flex items-center gap-2 px-3 py-3 text-xs font-medium text-muted-foreground">
						<svelte:component this={entry.Icon} class="size-3.5 shrink-0" />
						<span>{entry.label}</span>
					</div>
				{/if}
				<div
					class="relative"
					role="row"
					tabindex={i}
					ondragover={(e) => handleRowDragOver(e, i)}
					ondragleave={handleRowDragLeave}
					ondrop={(e) => handleRowDrop(e, i)}
				>
					{#if dragOverIndex === i && dragOverPosition === "above"}
						<div class="absolute top-0 left-0 right-0 h-0.5 bg-primary z-10 pointer-events-none"></div>
					{/if}

					<ContextMenu.Root>
						<ContextMenu.Trigger>
							<TrackRow
								{track}
								{orderedUids}
								index={i}
								{compact}
								{gridTemplate}
								{viewId}
								showNumber={v.number}
								showArtwork={v.artwork}
								showTitle={v.title}
								showArtist={v.artist}
								showAlbum={v.album}
								showYear={v.year}
								showRating={v.rating}
								showDuration={v.duration}
								showLabel={v.label}
								showOptions={v.options}
								{playlistUid}
							/>
						</ContextMenu.Trigger>
						<TrackContext track={track} />
					</ContextMenu.Root>

					{#if dragOverIndex === i && dragOverPosition === "below"}
						<div class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary z-10 pointer-events-none"></div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>