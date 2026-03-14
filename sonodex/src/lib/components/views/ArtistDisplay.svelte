<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	import { selection } from "$lib/session.svelte";
	import { library } from "$lib/library.svelte";
	import type { Artist, Track, Album } from "$lib/types";

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app/TrackTable.svelte";
	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	let artist: Artist | null = $state(null);

	let artistTracks: Track[] = $derived.by(() => {
		if (!artist) return [];
		return library.tracks.filter(t => {
			const artists: string[] = JSON.parse(t.artists ?? "[]");
			return artists.some(a => a.toLowerCase() === artist!.name.toLowerCase());
		});
	});

	let artistAlbums: Album[] = $derived.by(() => {
		if (!artist) return [];
		return library.albums.filter(a =>
			a.album_artist?.toLowerCase() === artist!.name.toLowerCase()
		);
	});

	function formatDuration(ms: number): string {
		const totalSeconds = Math.floor(ms / 1000);
		const minutes = Math.floor(totalSeconds / 60);
		const seconds = totalSeconds % 60;
		return `${minutes}:${seconds.toString().padStart(2, "0")}`;
	}

	$effect(() => {
		const id = selection.id;
		artist = null;
		invoke("get_artist", { id }).then((a) => {
			artist = a as Artist;
		});
	});
</script>

<div class="flex flex-col gap-4 border-2 h-full w-full overflow-hidden rounded-md">

	{#if artist}
		<div class="relative">
			{#if artist.banner_art_path}
				<img src={artist.banner_art_path} alt="" class="w-full h-32 object-cover rounded-t-md" />
			{:else}
				<div class="w-full h-32 bg-muted rounded-t-md"></div>
			{/if}

			<div class="absolute bottom-0 translate-y-1/2 left-4">
				<ArtworkDisplay id={artist.id} size={80} type="artist" />
			</div>
		</div>

		<div class="flex flex-col gap-2 px-4 pt-10">
			<div class="flex items-center gap-3">
				<h2 class="text-2xl font-bold">{artist.name}</h2>
				<div class="flex gap-2">
					<Button variant="default">Play All</Button>
					<Button variant="outline">Profile</Button>
				</div>
			</div>
		</div>

		<Tabs.Root value="home" class="flex flex-col min-h-0 flex-1 px-4">
			<Tabs.List>
				<Tabs.Trigger value="home">Home</Tabs.Trigger>
				<Tabs.Trigger value="discography">Discography</Tabs.Trigger>
				<Tabs.Trigger value="about">About</Tabs.Trigger>
			</Tabs.List>

			<Tabs.Content value="home" class="flex-1 overflow-y-auto">
				<h3 class="text-sm font-semibold mb-2 mt-2">TOP SONGS</h3>
				<!-- <TrackTable tracks={artistTracks.slice(0, 10)} /> -->
			</Tabs.Content>

			<Tabs.Content value="discography" class="flex-1 overflow-y-auto">
				<div class="grid gap-2 mt-2" style="grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));">
					{#each artistAlbums as album}
						<div class="flex flex-col gap-1 p-2 rounded-md border hover:border-primary cursor-default">
							<ArtworkDisplay id={album.id} size={120} type="album" />
							<span class="text-sm font-medium truncate">{album.title}</span>
							{#if album.release_date}
								<span class="text-xs text-muted-foreground">{album.release_date}</span>
							{/if}
						</div>
					{/each}
				</div>
			</Tabs.Content>

			<Tabs.Content value="about" class="flex-1 overflow-y-auto">
				{#if artist.about}
					<p class="text-sm leading-relaxed mt-2">{artist.about}</p>
				{:else}
					<p class="text-muted-foreground text-sm mt-2">No biography available.</p>
				{/if}
			</Tabs.Content>
		</Tabs.Root>
	{:else}
		<span class="text-muted-foreground text-sm p-4">Loading...</span>
	{/if}

</div>