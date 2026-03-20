<script lang="ts">
	import { library } from "$lib/library.svelte";
	import { setSelection } from "$lib/session.svelte";

	import type { Track, Album, Artist } from '$lib/types';

	import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	import AudioCard from "$lib/components/app/AudioCard.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";
    import ArtworkDisplay from "../app/ArtworkDisplay.svelte";

	let activeTab = $state("album");
	let search = $state("");

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

	const filteredAlbums = $derived(
		search.trim() === ""
			? library.albums
			: library.albums.filter((a) => {
				const q = search.toLowerCase();
				return a.title.toLowerCase().includes(q) || (a.artists?.toLowerCase() ?? "").includes(q);
			})
	);

	const filteredArtists = $derived(
		search.trim() === ""
			? library.artists
			: library.artists.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()))
	);
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

	<div class="min-h-0 flex-1 overflow-hidden">

		{#if activeTab === "artist"}
			<div class="flex flex-col h-full w-full overflow-hidden">
				<span>{library.artists.length} artists</span>
				<ScrollArea class="min-h-0 min-w-0">

					<div class="app-music-grid grid gap-2 p-3">

						{#each filteredArtists as artist}

							<button onclick={() => setSelection(artist.id, "artist")} class="flex flex-col items-center gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
								<div class="w-full aspect-square rounded-full bg-muted flex items-center justify-center overflow-hidden">
									<ArtworkDisplay id={artist.id} type="artist" />
								</div>
								<div class="min-w-0 w-full text-center">
									<p class="text-sm font-medium">{artist.name}</p>
								</div>
							</button>

						{/each}

					</div>

				</ScrollArea>
			</div>
		{:else if activeTab === "album"}
			<div class="flex flex-col h-full w-full overflow-hidden">
				<span>{library.albums.length} albums</span>
				<ScrollArea class="min-h-0 min-w-0">
					<div class="app-music-grid grid gap-2 p-3">

						{#each filteredAlbums as album}
							<AudioCard title={album.title} subTitle={album.album_artist} artworkId={album.id} type="album" />
						{/each}
					
					</div>
				</ScrollArea>
			</div>
		{:else if activeTab === "track"}
			<span>{library.tracks.length} tracks</span>
			<TrackTable type="library" tracks={filteredTracks} />
		{/if}
	
	</div>

</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>