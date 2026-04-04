<script lang="ts">

	// COMPONENTS
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	// CUSTOM COMPONENTS
	import TrackTable from "$lib/components/app-ui/track-table/TrackTable.svelte";
	import TrackTableSettings from "$lib/components/app-ui/track-table/TrackTableSettings.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";

	// SCRIPTS
	import { library } from "$lib/ts/library.svelte";
	import { createColumnState } from "$lib/ts/app/columnConfig.svelte"
	import { createPersistedViewState } from "$lib/ts/session.svelte";

	// VARIABLES
	let search = $state("");

	const cols = createColumnState("library");
	const view = createPersistedViewState("track-library", {
		sortField: "year",
		sortDir: "desc",
		colPreset: "library",
	});
	
	const filteredTracks = $derived(
		search.trim() === ""
			? library.tracks
			: library.tracks.filter((t) => {
					const q = search.toLowerCase();
					const title = t.title?.toLowerCase() ?? "";
					const artists = t.artists?.toLowerCase() ?? "";
					const album_artist = t.album_artist?.toLowerCase() ?? "";
					const albums = t.albums?.toLowerCase() ?? "";
					return title.includes(q) || artists.includes(q) || album_artist.includes(q) || albums.includes(q);
			  })
	);
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Tracks</h1>

		<SearchBar bind:search searchCount={filteredTracks.length} />

	</div>

	<div class="flex justify-between items-center gap-2">
		<span>{library.tracks.length} {library.tracks.length === 1 ? "track" : "tracks"}</span>
		<TrackTableSettings cols={cols} sort={view.sort} compact={view.compact} onCompactChange={(v) => view.compact = v} />
	</div>

	<ScrollArea class="h-full min-h-0 min-w-0 pr-4">
		<TrackTable tracks={filteredTracks} columns={cols} sort={view.sort} compact={view.compact} />
	</ScrollArea>

</div>