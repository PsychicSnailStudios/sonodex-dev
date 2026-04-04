<script lang="ts">

	// COMPONENTS
	import { ArrowDownAZ, ArrowUpAZ, ArrowUpDown, ChevronUp, ChevronDown, LayoutGrid, List } from "lucide-svelte";

	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import Toggle from "$lib/components/ui/toggle/toggle.svelte";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";

	// SCRIPTS
	import { library } from "$lib/ts/library.svelte";
	import { setSelection } from "$lib/ts/session.svelte";
	import { createPersistedViewState } from "$lib/ts/session.svelte";
	
	// VARIABLES
	let search = $state("");

	const view = createPersistedViewState("artist-library", {
		sortField: "number",
		sortDir: "asc",
		colPreset: "album",
	});

	const filteredArtists = $derived(() => {
		const list = search.trim() === ""
			? library.artists
			: library.artists.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()));

		return [...list].sort((a, b) => {
			const cmp = a.name.localeCompare(b.name);
			return view.sort.direction === "asc" ? cmp : -cmp;
		});
	});
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Artists</h1>

		<SearchBar bind:search searchCount={filteredArtists.length} />

	</div>

	<div class="flex flex-col h-full w-full overflow-hidden gap-3">
		<div class="flex justify-between">
			<span>{library.artists.length} {library.artists.length === 1 ? "artist" : "artists"}</span>
			<div class="flex items-center gap-1">
				<Button variant="ghost" size="icon" onclick={() => view.sort.direction = view.sort.direction === "asc" ? "desc" : "asc"}>
					{#if view.sort.direction === "asc"}
						<ArrowDownAZ />
					{:else}
						<ArrowUpAZ />
					{/if}
				</Button>
				<Toggle onPressedChange={() => view.compact = !view.compact}>
					{#if view.compact}
						<LayoutGrid />
						Table
					{:else}
						<List />
						List
					{/if}
				</Toggle>
			</div>
		</div>

		<ScrollArea class="min-h-0 min-w-0 pr-4">
		{#if view.compact}
			{#each filteredArtists() as artist}
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
		{:else}
			<div class="app-music-grid grid gap-2">
				{#each filteredArtists() as artist}
					<button onclick={() => setSelection(artist.uid, "artist")} class="flex flex-col items-center gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
						<div class="w-full aspect-square rounded-full bg-muted flex items-center justify-center overflow-hidden">
							<ArtworkDisplay uid={artist.uid} type="artist" />
						</div>
						<div class="min-w-0 w-full text-center">
							<p class="text-sm font-medium">{artist.name}</p>
						</div>
					</button>
				{/each}
			</div>
		{/if}
		</ScrollArea>
	</div>
		
</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
}
</style>