<script lang="ts">

	// APP
	import { invoke } from "@tauri-apps/api/core";
   import { onMount } from "svelte";

	// COMPONENTS
	import { Pencil } from "lucide-svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
   import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import AudioCard from "$lib/components/app-ui/AudioCard.svelte";
   import NavButtons from "$lib/components/app-ui/NavButtons.svelte";
   import TopTracks from "$lib/components/app-ui/TopTracks.svelte";
	import TagList from "$lib/components/app-ui/TagList.svelte";

	// SCRIPTS
	import { selection } from "$lib/ts/app-states/state_session.svelte";
	import { library } from "$lib/ts/library.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
   import { parseTags } from "$lib/ts/util/helpers";
	import { currentArtistTab } from "$lib/ts/app-states/state_session.svelte";
	import type { Artist, Track, Album } from "$lib/ts/util/types";

	// VARIABLES
	let artist = $state<Artist | null>(null);
	let genres = $derived(artist?.genres ? parseTags(artist.genres) : null);
	let tags = $derived(artist?.tags ? parseTags(artist.tags) : null);
	
	let artistAlbums: Album[] = $derived.by(() => {
		if (!artist) return [];

		const akaNames: string[] = artist.aka ? (JSON.parse(artist.aka) as string[]).map(n => n.toLowerCase()) : [];
		const allNames = new Set([artist.name.toLowerCase(), ...akaNames]);

		const albumUidsWithArtist = new Set<string>();
		for (const t of library.tracks) {
			if (!t.artists) continue;
			let artistsArr: string[];
			try { artistsArr = JSON.parse(t.artists); } catch { continue; }
			if (!artistsArr.some(a => allNames.has(a.toLowerCase()))) continue;
			if (!t.albums) continue;
			let albumsArr: { uid: string }[];
			try { albumsArr = JSON.parse(t.albums); } catch { continue; }
			for (const a of albumsArr) albumUidsWithArtist.add(a.uid);
		}

		return library.albums.filter(album => {
			if (album.album_artist && allNames.has(album.album_artist.toLowerCase())) return true;
			return albumUidsWithArtist.has(album.uid);
		});
	});

	// APP FUNCTIONS
	$effect(() => {
		const uid = selection.uid;
		if (!uid) return;
		artist = null;
		invoke<string>("resolve_uid", { uid }).then((resolvedUid) => {
			invoke("get_artist", { uid: resolvedUid }).then((a) => {
				artist = a as Artist;
			});
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
					<Button variant="ghost" onclick={() => openEditModal({ type: "artist", uid: artist!.uid })}><Pencil/></Button>
				</div>
			</div>
		</div>

		<Tabs.Root bind:value={currentArtistTab.id} class="flex flex-col min-h-0 flex-1 px-4">
			<Tabs.List>
				<Tabs.Trigger value="home">Home</Tabs.Trigger>
				<Tabs.Trigger value="discography">Discography</Tabs.Trigger>
				<Tabs.Trigger value="about">About</Tabs.Trigger>
			</Tabs.List>

			<Tabs.Content value="home" class="flex-1 overflow-y-auto">
				<h3 class="text-sm font-semibold mb-2 mt-2">TAGS & GENRES</h3>
				<TagList tags={genres} canEdit={false} />
				<TagList tags={tags} canEdit={false} />

				<h3 class="text-sm font-semibold mb-2 mt-2 pt-4">TOP SONGS</h3>
				<TopTracks uid={artist.uid} />
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
				<h3 class="text-sm font-semibold mb-2 mt-2">AKA</h3>
				{#if artist.aka}
					{@const akaList = JSON.parse(artist.aka) as string[]}
					<div class="flex flex-row gap-1 flex-wrap">
						{#each akaList as aka, i}
							<p class="text-sm leading-relaxed">{aka}{i < akaList.length - 1 ? "," : ""}</p>
						{/each}
					</div>
				{/if}
				
				<h3 class="text-sm font-semibold mb-2 mt-2 pt-4">BIO</h3>
				{#if artist.about}
					<p class="text-sm leading-relaxed mt-2">{artist.about}</p>
				{:else}
					<p class="text-muted-foreground text-sm mt-2">No biography available.</p>
				{/if}
			</Tabs.Content>
		</Tabs.Root>
	{:else}
		<NavButtons />
		<span class="text-muted-foreground text-sm p-4">Loading...</span>
	{/if}

</div>