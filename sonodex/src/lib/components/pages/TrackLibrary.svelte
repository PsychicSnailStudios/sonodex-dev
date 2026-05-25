<script lang="ts">
	import { ScrollArea } from "$shadcn/scroll-area/index.js";

	import TrackTable from "$lib/components/custom/track-table/TrackTable.svelte";
	import TrackTableSettings from "$lib/components/custom/track-table/TrackTableButtons.svelte";
	import SearchBar from "$lib/components/custom/search/SearchBar.svelte";

	import { createPersistedViewState } from "$ts/store/session.svelte";
	import { getTracks, searchTracks, onLibraryChange } from "$ts/store/library.svelte";
	import { createColumnState } from "$ts/util/columnConfig.svelte";
	import type { Track } from "$ts/util/types";

	const cols = createColumnState("library");
	const view = createPersistedViewState("track-library", {
		sortField: "year",
		sortDir: "desc",
		colPreset: "library",
	});

	let search = $state("");
	let allTracks = $state<Track[]>([]);
	let filteredTracks = $state<Track[]>([]);

	async function load() {
		allTracks = await getTracks();
	}

	async function runSearch() {
		filteredTracks = await searchTracks(search);
	}

	$effect(() => { load(); });
	$effect(() => { search; runSearch(); });
	$effect(() => {
		const unsub = onLibraryChange("tracks:changed", load);
		return unsub;
	});
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Tracks</h1>
		<SearchBar bind:search searchCount={filteredTracks.length} />
	</div>

	<div class="flex justify-between items-center gap-2">
		<span>{allTracks.length} {allTracks.length === 1 ? "track" : "tracks"}</span>
		<TrackTableSettings cols={cols} sort={view.sort} compact={view.compact} onCompactChange={(v) => view.compact = v} />
	</div>

	<ScrollArea class="h-full min-h-0 min-w-0 pr-4">
		<TrackTable tracks={filteredTracks} columns={cols} sort={view.sort} compact={view.compact} />
	</ScrollArea>

</div>