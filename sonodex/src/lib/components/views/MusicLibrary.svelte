<!-- NOTE: This view has been deprecated -->

<script lang="ts">
	import { library } from "$lib/ts/library.svelte";
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";

	import { createColumnState } from "$lib/ts/app/columnConfig.svelte"

	import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import Toggle from "$lib/components/ui/toggle/toggle.svelte";

	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
	import TrackTable from "$lib/components/app-ui/track-table/TrackTable.svelte";
	import ArtworkDisplay from "../app-ui/ArtworkDisplay.svelte";
	import TrackTableSettings from "$lib/components/app-ui/track-table/TrackTableSettings.svelte";
	import { SortState } from "$lib/ts/app/sortConfig.svelte"
	import { ArrowDownAZ, ArrowUpAZ, ArrowUpDown, ChevronUp, ChevronDown, LayoutGrid, List } from "lucide-svelte";
	import { parseArtists } from "$lib/ts/util/helpers";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	
	let activeTab = $state("album");
	let search = $state("");
	
	const sort = new SortState("year", "desc");
	const cols = createColumnState("library");
	let compact = $state(false);

	let artistSortDir = $state<"asc" | "desc">("asc");


	let albumSortField = $state<"name" | "artist" | "year">("name");
	let albumSortDir = $state<"asc" | "desc">("asc");

	function toggleAlbumSort(field: "name" | "artist" | "year") {
		if (albumSortField === field) {
			albumSortDir = albumSortDir === "asc" ? "desc" : "asc";
		} else {
			albumSortField = field;
			albumSortDir = "asc";
		}
	}

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

	const filteredArtists = $derived(() => {
		const list = search.trim() === ""
			? library.artists
			: library.artists.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()));

		return [...list].sort((a, b) => {
			const cmp = a.name.localeCompare(b.name);
			return artistSortDir === "asc" ? cmp : -cmp;
		});
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
			if (albumSortField === "name") {
				cmp = a.title.localeCompare(b.title);
			} else if (albumSortField === "artist") {
				cmp = (a.album_artist ?? "").localeCompare(b.album_artist ?? "");
			} else if (albumSortField === "year") {
				cmp = (a.release_date ?? "").localeCompare(b.release_date ?? "");
			}
			return albumSortDir === "asc" ? cmp : -cmp;
		});
	});
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<h1 class="h1">Music Library</h1>

		<ToggleGroup.Root type="single" value={activeTab} onValueChange={(v) => { if (v) activeTab = v }} class="flex gap-2">
			<ToggleGroup.Item value="artist" aria-label="Toggle artist" class="rounded-md">
				Artists
			</ToggleGroup.Item>
			<ToggleGroup.Item value="album" aria-label="Toggle album">
				Albums
			</ToggleGroup.Item>
			<ToggleGroup.Item value="track" aria-label="Toggle track">
				Tracks
			</ToggleGroup.Item>
		</ToggleGroup.Root>

		<Input
			placeholder="Search..."
			bind:value={search}
			class="w-48"
		/>

	</div>

	<div class="min-h-0 h-full w-full flex-1 overflow-hidden">

		{#if activeTab === "artist"}
			<div class="flex flex-col h-full w-full overflow-hidden gap-3">
				<div class="flex justify-between">
					<span>{library.artists.length} {library.artists.length === 1 ? "artist" : "artists"}</span>
					<div class="flex items-center gap-1">
						<Button variant="ghost" size="icon" onclick={() => artistSortDir = artistSortDir === "asc" ? "desc" : "asc"}>
							{#if artistSortDir === "asc"}
								<ArrowDownAZ />
							{:else}
								<ArrowUpAZ />
							{/if}
						</Button>
						<Toggle onPressedChange={() => compact = !compact}>
							{#if compact}
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
				{#if compact}
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
		{:else if activeTab === "album"}
			<div class="flex flex-col h-full w-full overflow-hidden gap-3">
				<div class="flex justify-between">
					<span>{library.albums.length} {library.albums.length === 1 ? "album" : "albums"}</span>
					<div class="flex items-center gap-1">
						<DropdownMenu.Root>
							<DropdownMenu.Trigger>
								<Button variant="outline" size="sm" class="gap-2">
									<ArrowUpDown size={14} />
									{{ name: "Name", artist: "Artist", year: "Year" }[albumSortField]}
								</Button>
							</DropdownMenu.Trigger>
							<DropdownMenu.Content>
								<DropdownMenu.Item
									class={albumSortField === "name" ? "bg-accent" : ""}
									onSelect={() => toggleAlbumSort("name")}
								>Name</DropdownMenu.Item>
								<DropdownMenu.Item
									class={albumSortField === "artist" ? "bg-accent" : ""}
									onSelect={() => toggleAlbumSort("artist")}
								>Artist</DropdownMenu.Item>
								<DropdownMenu.Item
									class={albumSortField === "year" ? "bg-accent" : ""}
									onSelect={() => toggleAlbumSort("year")}
								>Year</DropdownMenu.Item>
							</DropdownMenu.Content>
						</DropdownMenu.Root>

						<Button variant="outline" size="sm" onclick={() => albumSortDir = albumSortDir === "asc" ? "desc" : "asc"} class="px-2">
							{#if albumSortDir === "asc"}
								<ChevronUp size={14} />
							{:else}
								<ChevronDown size={14} />
							{/if}
						</Button>

						<Toggle onPressedChange={() => compact = !compact}>
							{#if compact}
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
				{#if compact}
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
		{:else if activeTab === "track"}
			<div class="flex flex-col h-full overflow-hidden gap-3">
				<div class="flex justify-between items-center gap-2">
					<span>{library.tracks.length} {library.tracks.length === 1 ? "track" : "tracks"}</span>
					<TrackTableSettings columns={cols} sort={sort} bind:compact />
				</div>
				<ScrollArea class="h-full min-h-0 min-w-0 pr-4">
					<TrackTable tracks={filteredTracks} columns={cols} sort={sort} compact={compact} />
				</ScrollArea>
			</div>
		{/if}

	</div>

</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
}
</style>