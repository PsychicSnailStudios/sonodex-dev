<script lang="ts">

	// COMPONENTS
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	// CUSTOM COMPONENTS
	import TrackTable from "$lib/components/app-ui/track-table/TrackTable.svelte";
	import TrackTableSettings from "$lib/components/app-ui/track-table/TrackTableButtons.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";

	// SCRIPTS
	import Fuse from "fuse.js";
	import { createPersistedViewState } from "$lib/ts/app-states/state_session.svelte";
	import { library } from "$lib/ts/library.svelte";
	import { createColumnState } from "$lib/ts/app/columnConfig.svelte"
	import { parseArtists, parseAlbum } from "$lib/ts/util/helpers";

	// VARIABLES
	const cols = createColumnState("library");
	const view = createPersistedViewState("track-library", {
		sortField: "year",
		sortDir: "desc",
		colPreset: "library",
	});
	
	let search = $state("");
	
	let fuseInstance: Fuse<(typeof library.tracks)[0]> | null = $state(null);
	let lastTracksRef: typeof library.tracks | null = null;

	function getFuse() {
		if (fuseInstance && lastTracksRef === library.tracks) return fuseInstance;
		lastTracksRef = library.tracks;
		fuseInstance = new Fuse(library.tracks, {
			keys: [
				{ name: "title",        weight: 0.5,  getFn: (t) => t.title ?? ""                    },
				{ name: "artists",      weight: 0.25, getFn: (t) => parseArtists(t.artists ?? "[]")  },
				{ name: "album_artist", weight: 0.15, getFn: (t) => t.album_artist ?? ""             },
				{ name: "albums",       weight: 0.1,  getFn: (t) => parseAlbum(t.albums ?? "[]")     },
				{ name: "tags",         weight: 0.05, getFn: (t) => t.tags ?? ""                     },
				{ name: "genres",       weight: 0.05, getFn: (t) => t.genres ?? ""                   },
			],
			threshold: 0.35,
			ignoreLocation: true,
			includeScore: false,
			useExtendedSearch: false,
			minMatchCharLength: 2,
		});
		return fuseInstance;
	}

	const filteredTracks = $derived(
		search.trim().length < 2
			? library.tracks
			: getFuse().search(search).map((r) => r.item)
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