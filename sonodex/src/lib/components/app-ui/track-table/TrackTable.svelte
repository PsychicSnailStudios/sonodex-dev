<script lang="ts">
	import type { Track } from "$lib/ts/util/types"
	import { readText } from "@tauri-apps/plugin-clipboard-manager";
	import type { ColumnState } from "$lib/ts/app/columnConfig.svelte"
	import type { SortState } from "$lib/ts/app/sortConfig.svelte"
	import { Clock2, Star, ChevronUp, ChevronDown, ChevronsUpDown } from "lucide-svelte"
	import { parseAlbum, parseTrackNumber } from "$lib/ts/util/helpers"
	import { trackSelection, setTrackSelectionContext, clearTrackSelection, copySelectedToClipboard, copySelectedNameToClipboard } from "$lib/ts/app/trackSelection.svelte"
	import { dragState, endDrag } from "$lib/ts/app/dragState.svelte"
	import { removeTracksFromPlaylist, reorderPlaylistTracks, addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte"
	import TrackRow from "$lib/components/app-ui/track-table/TrackRow.svelte"
   import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";
   import { addTrackToQueue } from "$lib/ts/audio/audioManager.svelte";

	let {
		tracks,
		columns,
		sort,
		compact = false,
		playlistUid = null,
	} = $props<{
		tracks: Track[];
		columns: ColumnState;
		sort: SortState;
		compact?: boolean;
		playlistUid?: string | null;
	}>()

	const v = $derived(columns.visible)

	const sortedTracks = $derived.by(() => {
		if (!sort.field || !sort.direction) return tracks
		const dir = sort.direction === "asc" ? 1 : -1
		return [...tracks].sort((a, b) => {
			switch (sort.field) {
				case "title":    return dir * (a.title ?? "").localeCompare(b.title ?? "")
				case "album":    return dir * (parseAlbum(a.albums) ?? "").localeCompare(parseAlbum(b.albums) ?? "")
				case "year":     return dir * ((a.year ?? "").localeCompare(b.year ?? ""))
				case "rating":   return dir * ((a.rating ?? -1) - (b.rating ?? -1))
				case "duration": return dir * ((a.duration_ms ?? 0) - (b.duration_ms ?? 0))
				case "label":    return dir * (a.label ?? "").localeCompare(b.label ?? "")
				case "artist":   return dir * (a.album_artist ?? "").localeCompare(b.album_artist ?? "")
				case "number":   return dir * (sortByNumber(a, b))
				default:         return 0
			}
		})
	})

	function sortByNumber(a, b) {
		if (playlistUid) return 0
		
		return (parseTrackNumber(a.albums) ?? 0) - (parseTrackNumber(b.albums) ?? 0)
	}

	const orderedUids = $derived(sortedTracks.map((t) => t.uid))

	const gridTemplate = $derived.by(() => {
		const parts: string[] = []
		if (v.number)            parts.push("40px")
		if (v.artwork)           parts.push("40px")
		if (v.title)             parts.push("1fr")
		if (v.artist)				 parts.push("1fr")
		if (v.album)             parts.push("1fr")
		if (v.year)              parts.push("60px")
		if (v.rating)            parts.push("60px")
		if (v.duration)          parts.push("80px")
		if (v.label)             parts.push("100px")
		parts.push("40px")
		return parts.join(" ")
	})

	let dragOverIndex = $state<number | null>(null)
	let dragOverPosition = $state<"above" | "below">("below")

	$effect(() => {
		setTrackSelectionContext(playlistUid ? "playlist" : "library", playlistUid ?? null)
	})

	function handleTableClick(e: MouseEvent) {
		if ((e.target as HTMLElement) === e.currentTarget) clearTrackSelection()
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			clearTrackSelection()
			return
		}
		if ((e.ctrlKey || e.metaKey) && e.key === "c") {
			e.preventDefault()
			copySelectedToClipboard(orderedUids)
			return
		}
		if (e.key === "Delete" && playlistUid && trackSelection.count > 0) {
			e.preventDefault()
			removeTracksFromPlaylist(playlistUid, [...trackSelection.selected])
			clearTrackSelection()
			return
		}
		if ((e.ctrlKey || e.metaKey) && e.key === "v") {
			if (!playlistUid) return;
			e.preventDefault();
			readText().then(async (text) => {
				const parts = text.split("\n---\n");
				if (parts.length < 2) return;
				const uids = parts[1].split("\n").map((u) => u.trim()).filter(Boolean);
				if (uids.length === 0) return;
				await addTracksToPlaylist(playlistUid, uids);
			});
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
	class="flex flex-col p-0 outline-none"
	onclick={handleTableClick}
	onkeydown={handleKeyDown}
	ondragover={(e) => e.preventDefault()}
	ondrop={handleTableDrop}
>
	<div
		class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b"
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
		{#if v.artwork}<span></span>{/if}
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
			<button class="flex items-center gap-1 hover:text-foreground transition-colors" onclick={() => sort.cycle("rating")}>
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

	<div>
		{#each sortedTracks as track, i (track.uid)}
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
					<ContextMenu.Content>
						<ContextMenu.Group>
							<ContextMenu.Item onSelect={() => copySelectedNameToClipboard(track)}>Copy Track & Artist Name</ContextMenu.Item>
							<ContextMenu.Item onSelect={() => addTrackToQueue(track)}>Add to Queue</ContextMenu.Item>
						</ContextMenu.Group>
						<ContextMenu.Separator />
						<ContextMenu.Group>
							<ContextMenu.Item>
								<TrackPlaylistEditButton track={track} isButton={false} />
							</ContextMenu.Item>
						</ContextMenu.Group>
					</ContextMenu.Content>
				</ContextMenu.Root>

				{#if dragOverIndex === i && dragOverPosition === "below"}
					<div class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary z-10 pointer-events-none"></div>
				{/if}
			</div>
		{/each}
	</div>
</div>