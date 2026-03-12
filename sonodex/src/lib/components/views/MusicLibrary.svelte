<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";

	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import VirtualList from "svelte-virtual-list";
	import { Button } from "$lib/components/ui/button/index.js";

	import TrackArtwork from "$lib/components/TrackArtwork.svelte";

  	let activeTab = $state("album");
	let search = $state("");
	let tracks: Track[] = $state([]);

	type Track = {
		id: number;
		path: string;
		last_modified: number;
		title: string | null;
		artists: string | null;
		album_artist: string | null;
		albums: string | null;
		genres: string | null;
		year: string | null;
		rating: number | null;
		tags: string | null;
		duration_ms: number | null;
		bpm: number | null;
		key: string | null;
	};

  type Artist = {
    name: string;
  };

  const artists = $derived((() => {
    const seen = new Set<string>();
    for (const track of tracks) {
      const names: string[] = track.artists ? (() => { try { return JSON.parse(track.artists); } catch { return []; } })() : [];
      for (const name of names) {
        if (name) seen.add(name);
      }
      if (track.album_artist) seen.add(track.album_artist);
    }
    return Array.from(seen).sort().map((name) => ({ name }));
  })());

  const filteredArtists = $derived(
    search.trim() === ""
      ? artists
      : artists.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()))
  );

  type Album = {
    name: string;
    artist: string | null;
    year: string | null;
    artworkTrackId: number;
  };

  const albums = $derived((() => {
    const map = new Map<string, Album>();
    for (const track of tracks) {
      const parsed = track.albums ? (() => { try { return JSON.parse(track.albums); } catch { return []; } })() : [];
      for (const a of parsed) {
        const name = typeof a === "string" ? a : a.name;
        if (name && !map.has(name)) {
          map.set(name, {
            name,
            artist: track.album_artist ?? null,
            year: track.year ?? null,
            artworkTrackId: track.id,
          });
        }
      }
    }
    return Array.from(map.values());
  })());

  const filteredAlbums = $derived(
    search.trim() === ""
      ? albums
      : albums.filter((a) => {
          const q = search.toLowerCase();
          return a.name.toLowerCase().includes(q) || (a.artist?.toLowerCase() ?? "").includes(q);
        })
  );

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

	onMount(async () => {
		tracks = await invoke("get_tracks");
	});
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div class="flex justify-between items-center gap-2">
		<span>Music</span>

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
				<ScrollArea class="min-h-0 min-w-0">

					<div class="app-music-grid grid gap-2 p-3">

						{#each artists as artist (artist.name)}
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
				<ScrollArea class="min-h-0 min-w-0">
					
					<div class="app-music-grid grid gap-2 p-3">

					{#each albums as album (album.name)}
						<AudioCard {album} />
					{/each}
					
					</div>

				</ScrollArea>
			</div>
		{:else if activeTab === "track"}
			<div class="flex flex-col h-full">
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
					<VirtualList items={tracks} itemHeight={56} let:item={track}>
						<div class="grid items-center px-3 border-b hover:bg-muted/50" style="grid-template-columns: 40px 1fr 1fr 60px 120px 60px 40px; height: 56px;">
					<TrackArtwork id={track.id} />
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

.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>