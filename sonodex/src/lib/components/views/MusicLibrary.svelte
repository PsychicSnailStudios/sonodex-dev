<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";

	import type { Track, Album, Artist } from '$lib/types';

	import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import VirtualList from "svelte-virtual-list";

	import TrackArtwork from "$lib/components/app/TrackArtwork.svelte";
	import AudioCard from "$lib/components/app/AudioCard.svelte";

  	let activeTab = $state("album");
	let search = $state("");
	let tracks: Track[] = $state([]);
	let albums: Album[] = $state([]);
	let artists: Artist[] = $state([]);

	const filteredTracks = $derived(
		search.trim() === ""
			? tracks
			: tracks.filter((t) => {
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
			? albums
			: albums.filter((a) => {
				const q = search.toLowerCase();
				return a.title.toLowerCase().includes(q) || (a.artists?.toLowerCase() ?? "").includes(q);
			})
	);

	const filteredArtists = $derived(
		search.trim() === ""
			? artists
			: artists.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()))
	);

	function parseArtists(artists: string | null): string {
		if (!artists) return "Unknown Artist";
		try {
			const parsed = JSON.parse(artists);
			return Array.isArray(parsed) ? parsed.join(", ") : "Unknown Artist";
		} catch {
			return "Unknown Artist";
		}
	}

	function parseAlbum(albums: string | null): string {
		if (!albums) return "—";
		try {
			const parsed = JSON.parse(albums);
			return Array.isArray(parsed) && parsed.length > 0 ? parsed[0].name : "—";
		} catch {
			return "—";
		}
	}

	function formatRating(rating: number | null): string {
		if (rating === null) return "—";
		return rating.toFixed(1);
	}

	function formatDuration(ms: number | null): string {
		if (ms === null) return "—";
		const totalSeconds = Math.floor(ms / 1000);
		const minutes = Math.floor(totalSeconds / 60);
		const seconds = totalSeconds % 60;
		return `${minutes}:${seconds.toString().padStart(2, "0")}`;
	}
	
	onMount(async () => {
		tracks = await invoke("get_tracks");
		albums = await invoke("get_albums");
		artists = await invoke("get_artists");
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

	<div class="min-h-0 flex-1 overflow-hidden">

		{#if activeTab === "artist"}
			<div class="flex flex-col h-full w-full overflow-hidden">
				<span>{artists.length} artists</span>
				<ScrollArea class="min-h-0 min-w-0">

					<div class="app-music-grid grid gap-2 p-3">

						{#each filteredArtists as artist}
							<div class="flex flex-col items-center gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
								<div class="w-full aspect-square rounded-full bg-muted flex items-center justify-center overflow-hidden">
									<svg class="w-10 h-10 text-muted-foreground/40" viewBox="0 0 24 24" fill="currentColor">
										<path d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z"/>
									</svg>
								</div>
								<div class="min-w-0 w-full text-center">
									<p class="text-sm font-medium truncate">{artist.name}</p>
								</div>
							</div>
						{/each}

					</div>

				</ScrollArea>
			</div>
		{:else if activeTab === "album"}
			<div class="flex flex-col h-full w-full overflow-hidden">
				<span>{albums.length} albums</span>
				<ScrollArea class="min-h-0 min-w-0">
					
					<div class="app-music-grid grid gap-2 p-3">

						{#each filteredAlbums as album}
							<AudioCard title={album.title} subTitle={album.album_artist} artworkId={album.id} />
						{/each}
					
					</div>

				</ScrollArea>
			</div>
		{:else if activeTab === "track"}
			<div class="flex flex-col h-full">
				<span>{tracks.length} tracks</span>

				<div class="grid text-xs font-medium text-muted-foreground px-3 py-2 border-b" style="grid-template-columns: 40px 1fr 1fr 60px 120px 60px 40px;">
					<span></span>
					<span>Title</span>
					<span>Album</span>
					<span>Year</span>
					<span>Rating</span>
					<span>Duration</span>
					<span></span>
				</div>

				<div class="flex-1 overflow-hidden">
					<VirtualList items={filteredTracks} itemHeight={56} let:item={track}>
						<div class="grid items-center px-3 border-b hover:bg-muted/50" style="grid-template-columns: 40px 1fr 1fr 60px 120px 60px 40px; height: 56px;">
							<TrackArtwork id={track.id} width={30} height={30} />
							<div class="flex flex-col min-w-0">
								<span class="text-sm truncate">{track.title ?? "Unknown Title"}</span>
								<span class="text-xs text-muted-foreground truncate">{parseArtists(track.artists)}</span>
							</div>

							<span class="text-sm truncate pr-4">{parseAlbum(track.albums)}</span>
							<span class="text-sm">{track.year ?? "—"}</span>
							<span class="text-sm font-mono">{formatRating(track.rating)}</span>
							<span class="text-sm font-mono">{formatDuration(track.duration_ms)}</span>
							
							<Button variant="ghost" size="icon">⋯</Button>
						</div>
					</VirtualList>
				</div>
			</div>
		{/if}
	
	</div>

</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>