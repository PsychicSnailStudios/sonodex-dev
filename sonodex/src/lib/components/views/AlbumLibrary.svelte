<script lang="ts">
	// APP
	// COMPONENTS
	import { ArrowDownAZ, ArrowUpAZ, ArrowUpDown, ChevronUp, ChevronDown, LayoutGrid, List } from "lucide-svelte";

	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import Toggle from "$lib/components/ui/toggle/toggle.svelte";

	// CUSTOM COMPONENTS
	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";

	// SCRIPTS
	import { library } from "$lib/ts/library.svelte";
	import { setSelection } from "$lib/ts/session.svelte";
	import { parseArtists } from "$lib/ts/util/helpers";
	import { createPersistedViewState } from "$lib/ts/session.svelte";
	import type { SortField } from "$lib/ts/app/sortConfig.svelte";
	
	// VARIABLES
	let search = $state("");

	const view = createPersistedViewState("album-library", {
		sortField: "number",
		sortDir: "asc",
		colPreset: "album",
	});

	const filteredAlbums = $derived(() => {
		const list = search.trim() === ""
			? library.albums
			: library.albums.filter((a) => {
				const q = search.toLowerCase();
				return a.title.toLowerCase().includes(q) || (a.artists?.toLowerCase() ?? "").includes(q);
			});

		return [...list].sort((a, b) => {
			let cmp = 0;
			if (view.sort.field === "name" as SortField) {
				cmp = a.title.localeCompare(b.title);
			} else if (view.sort.field === "artist" as SortField) {
				cmp = (a.album_artist ?? "").localeCompare(b.album_artist ?? "");
			} else if (view.sort.field === "year" as SortField) {
				cmp = (a.release_date ?? "").localeCompare(b.release_date ?? "");
			}
			return view.sort.direction === "asc" ? cmp : -cmp;
		});
	});

	// FUNCTIONS
	function toggleAlbumSort(field: "name" | "artist" | "year") {
		if (view.sort.field === field) {
			view.sort.direction = view.sort.direction === "asc" ? "desc" : "asc";
		} else {
			view.sort.field = field as SortField;
			view.sort.direction = "asc";
		}
	}
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Albums</h1>

		<SearchBar bind:search searchCount={filteredAlbums.length} />

	</div>

	<div class="flex flex-col h-full w-full overflow-hidden gap-3">
		<div class="flex justify-between">
			<span>{library.albums.length} {library.albums.length === 1 ? "album" : "albums"}</span>
			<div class="flex items-center gap-1">
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						<Button variant="ghost" size="sm" class="gap-2">
							<!-- <ArrowUpDown size={14} /> -->
							{view.sort.field}
						</Button>
					</DropdownMenu.Trigger>
					<DropdownMenu.Content>
						<DropdownMenu.Item
							class={view.sort.field === "name" as SortField ? "bg-accent" : ""}
							onSelect={() => toggleAlbumSort("name")}
						>Name</DropdownMenu.Item>
						<DropdownMenu.Item
							class={view.sort.field === "artist" ? "bg-accent" : ""}
							onSelect={() => toggleAlbumSort("artist")}
						>Artist</DropdownMenu.Item>
						<DropdownMenu.Item
							class={view.sort.field === "year" ? "bg-accent" : ""}
							onSelect={() => toggleAlbumSort("year")}
						>Year</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>

				<Button variant="ghost" size="sm" onclick={() => view.sort.direction = view.sort.direction === "asc" ? "desc" : "asc"} class="px-2">
					{#if view.sort.direction === "asc"}
						<ChevronUp size={14} />
					{:else}
						<ChevronDown size={14} />
					{/if}
				</Button>

				<Toggle onPressedChange={() => view.compact = !view.compact}>
					{#if view.compact}
						<LayoutGrid />
						<!-- Table -->
					{:else}
						<List />
						<!-- List -->
					{/if}
				</Toggle>
			</div>
		</div>

		<ScrollArea class="min-h-0 min-w-0 pr-4">
		{#if view.compact}
			{#each filteredAlbums() as album}
			<div
				class="grid items-center px-3 border-b hover:bg-muted/50"
				style="grid-template-columns: 40px 1fr 1fr 40px; height: 56px;"
			>
				<ArtworkDisplay uid={album.uid} type="album" size={40} />
				<button onclick={() => setSelection(album.uid, "album")} class="pl-2 text-sm truncate text-left">
					<p class="text-sm font-medium">{album.title}</p>
				</button>
				<p class="text-sm">{parseArtists(album.artists)}</p>
				<p class="text-sm">{album.release_date}</p>
			</div>
			{/each}
		{:else}
			<div class="app-music-grid grid gap-2">
				{#each filteredAlbums() as album}
					<AudioCard title={album.title} subTitle={album.album_artist} artworkUid={album.uid} type="album" />
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