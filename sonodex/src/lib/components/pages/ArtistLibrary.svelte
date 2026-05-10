<script lang="ts">

	// COMPONENTS
	import { ArrowDownAZ, ArrowUpAZ, ArrowUpDown, ChevronUp, ChevronDown, LayoutGrid, List } from "lucide-svelte";

	import { Button } from "$shadcn/button/index.js";
	import { ScrollArea } from "$shadcn/scroll-area/index.js";
	import { Input } from "$shadcn/input/index.js";
	import Toggle from "$shadcn/toggle/toggle.svelte";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";
	import MediaGrid from "$lib/layouts/MediaGrid.svelte";

	// SCRIPTS
	import Fuse from "fuse.js";
	import { library } from "$ts/store/library.svelte";
	import { setSelection } from "$ts/store/session.svelte";
	import { createPersistedViewState } from "$ts/store/session.svelte";
	
	// VARIABLES
	let search = $state("");

	const view = createPersistedViewState("artist-library", {
		sortField: "number",
		sortDir: "asc",
		colPreset: "album",
	});

	let fuseInstance: Fuse<(typeof library.artists)[0]> | null = $state(null);
	let lastArtistsRef: typeof library.artists | null = null;

	function getFuse() {
		if (fuseInstance && lastArtistsRef === library.artists) return fuseInstance;
		lastArtistsRef = library.artists;
		fuseInstance = new Fuse(library.artists, {
			keys: [
				{ name: "name",   weight: 0.5,  getFn: (t) => t.name ?? ""   },
				{ name: "akas",   weight: 0.35, getFn: (t) => t.aka ?? ""    },
				{ name: "tags",   weight: 0.05, getFn: (t) => t.tags ?? ""   },
				{ name: "genres", weight: 0.05, getFn: (t) => t.genres ?? "" },
			],
			threshold: 0.35,
			ignoreLocation: true,
			includeScore: false,
			useExtendedSearch: false,
			minMatchCharLength: 2,
		});
		return fuseInstance;
	}

	const filteredArtists = $derived.by(() => {
		const list = search.trim().length < 2
			? library.artists
			: getFuse().search(search).map((r) => r.item);

		return [...list].sort((a, b) => {
			const cmp = a.name.localeCompare(b.name);
			return view.sort.direction === "asc" ? cmp : -cmp;
		});
	});
</script>

<div class="flex flex-col gap-2 pt-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2 pr-4 pl-4">
		<h1 class="h1">Artists</h1>

		<SearchBar bind:search searchCount={filteredArtists.length} />

	</div>

	<div class="flex flex-col h-full w-full overflow-hidden gap-3 p-0.5">
		<div class="flex justify-between pr-4 pl-4">
			<span>{library.artists.length} {library.artists.length === 1 ? "artist" : "artists"}</span>
			<div class="flex items-center gap-1">
				<Button variant="ghost" size="icon" onclick={() => view.sort.direction = view.sort.direction === "asc" ? "desc" : "asc"}>
					{#if view.sort.direction === "asc"}
						<ArrowDownAZ />
					{:else}
						<ArrowUpAZ />
					{/if}
				</Button>
				<Button variant="ghost" size="icon" onclick={() => view.compact = !view.compact}>
					{#if view.compact}
					<!-- Table -->
					<List />
					{:else}
					<LayoutGrid />
						<!-- List -->
					{/if}
				</Button>
			</div>
		</div>

		<ScrollArea class="min-h-0 min-w-0 pr-2">
		{#if view.compact}
			<div class="pr-4 pl-4 pb-4">
				{#each filteredArtists as artist}
					<div
						class="grid items-center px-3 border-b hover:bg-muted/50"
						style="grid-template-columns: 40px 1fr; height: 56px;"
					>
						<ArtworkDisplay uid={artist.uid} type="artist" size={40} />
						<button onclick={() => setSelection(artist.uid, "artist")} class="pl-2 text-sm truncate text-left">
							<p class="text-sm font-medium">{artist.name}</p>
						</button>
					</div>
				{/each}
			</div>
		{:else}
			<MediaGrid>
				{#each filteredArtists as artist}
					<button onclick={() => setSelection(artist.uid, "artist")} class="flex flex-col items-center gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
						<div class="w-full aspect-square rounded-full bg-muted flex items-center justify-center overflow-hidden">
							<ArtworkDisplay uid={artist.uid} type="artist" />
						</div>
						<div class="min-w-0 w-full text-center">
							<p class="text-sm font-medium">{artist.name}</p>
						</div>
					</button>
				{/each}
			</MediaGrid>
		{/if}
		</ScrollArea>
	</div>
		
</div>