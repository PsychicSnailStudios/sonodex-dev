<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
	import { Input } from "$lib/components/ui/input/index.js";

	import MusicAlbums from "$lib/components/views/MusicAlbums.svelte";
	import MusicArtists from "$lib/components/views/MusicArtists.svelte";
	import MusicTracks from "$lib/components/views/MusicTracks.svelte";

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
			<MusicArtists artists={filteredArtists} />
		{:else if activeTab === "album"}
			<MusicAlbums albums={filteredAlbums} />
		{:else if activeTab === "track"}
			<MusicTracks tracks={filteredTracks} />
		{/if}
	</div>

</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>