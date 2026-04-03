<script lang="ts">

	// APP
	import { invoke } from "@tauri-apps/api/core";

	// COMPONENTS
	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
   import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
   import NavButtons from "$lib/components/app-ui/NavButtons.svelte";

	// SCRIPTS
	import { selection } from "$lib/ts/session.svelte";
	import { library } from "$lib/ts/library.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import type { Artist, Track, Album } from "$lib/ts/util/types";

	// VARIABLES
	let artist: Artist | null = $state(null);
	
	let artistAlbums: Album[] = $derived.by(() => {
		if (!artist) return [];
		return library.albums.filter(a =>
			a.album_artist?.toLowerCase() === artist!.name.toLowerCase()
		);
	});

	// APP FUNCTIONS
	$effect(() => {
		const uid = selection.uid;
		artist = null;
		invoke("get_artist", { uid }).then((a) => {
			artist = a as Artist;
		});
	});
</script>

<div class="flex flex-col gap-4 border-2 h-full w-full overflow-hidden rounded-md">

	{#if artist}
		<div class="relative">
			<NavButtons class="absolute top-4 left-4"/>

			{#if artist.banner_art_path}
				<img src={artist.banner_art_path} alt="" class="w-full h-32 object-cover rounded-t-md" />
			{:else}
				<div class="w-full h-32 bg-muted rounded-t-md"></div>
			{/if}

			<div class="absolute bottom-0 translate-y-1/2 left-4">
				<ArtworkDisplay uid={artist.uid} size={80} type="artist" />
			</div>
		</div>

		<div class="flex flex-col gap-2 px-4 pt-10">
			<div class="flex items-center gap-3">
				<h2 class="text-2xl font-bold">{artist.name}</h2>
				<div class="flex gap-2">
					<Button variant="default">Play All</Button>
					<Button variant="outline">Shuffle</Button>
					<Button variant="ghost" onclick={() => openEditModal({ type: "artist", uid: artist!.uid })}>...</Button>
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
			</Tabs.Content>

			<Tabs.Content value="discography" class="flex-1 overflow-y-auto">
				<div class="flex flex-col gap-2 h-full w-full">
					<ScrollArea class="min-h-0 min-w-0">
						<div class="grid gap-2 mt-2 pr-4" style="grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));">
							{#each artistAlbums as album}
								<AudioCard title={album.title} subTitle={album.album_artist} artworkUid={album.uid} type="album" />
							{/each}
						</div>
					</ScrollArea>
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
