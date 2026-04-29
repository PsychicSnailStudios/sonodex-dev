<script lang="ts">

	// APP
	import { untrack } from "svelte";
	import { invoke, convertFileSrc } from "@tauri-apps/api/core";

	// COMPONENTS
	import { Music4, User, DiscAlbum, ListMusic } from "lucide-svelte";
	import { Skeleton } from "$lib/components/ui/skeleton/index.js";
	
	// SCRIPTS
	import { library } from "$lib/ts/library.svelte";
	import type { AudioCatagories, Track } from "$lib/ts/util/types";
	import { trackSelection } from "$lib/ts/app/trackSelection.svelte";
	import { artworkCache, artworkInflight } from "$lib/ts/app-states/artworkCache";
	
	// PROPS
	let { uid, size = null, type = "track", previewPath = null }: { uid: string; size?: number | null; type?: AudioCatagories; previewPath?: string | null } = $props();

	// VARIABLES
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
		unknown: () => null,
	};

	const isGhost = $derived.by(() => {
		if (type !== "track") return false;
		const track: Track | undefined = library.tracks.find(t => t.uid === uid);
		if (track) return isGhostTrack(track);
		return true;
	});

	function isGhostTrack(track: Track): boolean {
		if (track.remote_path && track.remote_path.length > 0) return false;
		if (!track.path || track.path === "" || track.path === track.uid) return true;
		return false;
	}

	async function fetchAndCache(cacheKey: string, command: string, fetchUid: string): Promise<string | null> {
		if (artworkCache.has(cacheKey)) return artworkCache.get(cacheKey)!;

		if (artworkInflight.has(cacheKey)) return artworkInflight.get(cacheKey)!;

		const promise = invoke<number[] | null>(command, { uid: fetchUid }).then((bytes) => {
			let url: string | null = null;
			if (bytes) {
				const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
				url = URL.createObjectURL(blob);
			}
			artworkCache.set(cacheKey, url);
			artworkInflight.delete(cacheKey);
			return url;
		}).catch(() => {
			artworkCache.set(cacheKey, null);
			artworkInflight.delete(cacheKey);
			return null;
		});

		artworkInflight.set(cacheKey, promise);
		return promise;
	}

	// APP FUNCTIONS
	$effect(() => {
		if (previewPath) {
			artworkUrl = convertFileSrc(previewPath);
			fetchedKey = null;
			loaded = false;
			return;
		}

		const currentUid = uid;
		const currentType = type;
		const currentKey = `${currentType}:${currentUid}`;

		if (untrack(() => fetchedKey) === currentKey && untrack(() => artworkUrl) !== null) {
			return;
		}

		loaded = false;

		const localPath = untrack(() => pathMap[currentType]?.() ?? null);

		if (localPath) {
			artworkUrl = convertFileSrc(localPath);
			fetchedKey = currentKey;
			return;
		}

		// Check cache immediately before setting up observer
		if (artworkCache.has(currentKey)) {
			artworkUrl = artworkCache.get(currentKey) ?? null;
			fetchedKey = currentKey;
			return;
		}

		artworkUrl = null;

		let observer: IntersectionObserver | null = null;

		if (el) {
			observer = new IntersectionObserver(async ([entry]) => {
				if (entry.isIntersecting) {
					observer?.disconnect();
					const command = commandMap[currentType as keyof typeof commandMap];
					if (!command) return;
					const url = await fetchAndCache(currentKey, command, currentUid);
					artworkUrl = url;
					fetchedKey = currentKey;
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
	  class="rounded-sm overflow-hidden relative bg-muted flex-shrink-0 w-full aspect-square"
	  class:opacity-50={isGhost}>

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
				<!-- <ListMusic class="text-muted-foreground" /> -->
				<slot />
			{/if}
		</div>
	{/if}
</div>
