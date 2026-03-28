<script lang="ts">
	import { invoke, convertFileSrc } from "@tauri-apps/api/core";
	import { Skeleton } from "$lib/components/ui/skeleton/index.js";
	import { Music4, User, DiscAlbum, ListMusic } from "lucide-svelte";
	import type { AudioCatagories } from "$lib/types";
	import { library } from "$lib/ts/library.svelte";
	import { untrack } from "svelte";

	let { uid, size = null, type = "track" }: { uid: string; size?: number | null; type?: AudioCatagories } = $props();

	let artworkUrl: string | null = $state(null);
	let loaded = $state(false);
	let el: HTMLDivElement;
	let fetchedKey = $state<string | null>(null);

	const commandMap = {
		track: "get_track_artwork",
		album: "get_album_artwork",
		artist: "get_artist_profile_art",
		playlist: "get_playlist_artwork",
	};

	const pathMap: Record<AudioCatagories, () => string | null> = {
		track: () => library.tracks.find(t => t.uid === uid)?.artwork_path ?? null,
		album: () => library.albums.find(a => a.uid === uid)?.artwork_path ?? null,
		artist: () => library.artists.find(a => a.uid === uid)?.profile_art_path ?? null,
		playlist: () => library.playlists.find(p => p.uid === uid)?.artwork_path ?? null,
	};

	$effect(() => {
		const currentUid = uid;
		const currentType = type;
		const currentKey = `${currentType}:${currentUid}`;

		if (untrack(() => fetchedKey) === currentKey && untrack(() => artworkUrl) !== null) {
			return;
		}

		let observer: IntersectionObserver | null = null;

		loaded = false;

		const localPath = untrack(() => pathMap[currentType]?.() ?? null);

		if (localPath) {
			artworkUrl = convertFileSrc(localPath);
			fetchedKey = currentKey;
			return;
		}

		artworkUrl = null;

		if (el) {
			observer = new IntersectionObserver(async ([entry]) => {
				if (entry.isIntersecting) {
					observer?.disconnect();
					try {
						const bytes: number[] | null = await invoke(commandMap[currentType], { uid: currentUid });
						if (bytes) {
							const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
							artworkUrl = URL.createObjectURL(blob);
							fetchedKey = currentKey;
						} else {
							artworkUrl = null;
						}
					} catch {
						artworkUrl = null;
					}
				}
			}, { rootMargin: "200px" });

			observer.observe(el);
		}

		return () => {
			observer?.disconnect();
		};
	});
</script>

<div bind:this={el}
	  style={size ? `width: ${size}px; height: ${size}px;` : ""}
	  class="rounded-sm overflow-hidden relative bg-muted flex-shrink-0 w-full aspect-square">

	{#if artworkUrl}
		{#if !loaded}
			<Skeleton class="w-full h-full" />
		{/if}
		<img
			src={artworkUrl}
			alt=""
			class="absolute inset-0 w-full h-full object-cover transition-opacity duration-600 aspect-square"
			class:opacity-0={!loaded}
			onload={() => loaded = true}
		/>
	{:else}
		<div class="absolute inset-0 flex items-center justify-center">
			{#if type === "track"}
				<Music4 class="text-muted-foreground" />
			{:else if type === "album"}
				<DiscAlbum class="text-muted-foreground" />
			{:else if type === "artist"}
				<User class="text-muted-foreground" />
			{:else if type === "playlist"}
				<ListMusic class="text-muted-foreground" />
			{/if}
		</div>
	{/if}
</div>
