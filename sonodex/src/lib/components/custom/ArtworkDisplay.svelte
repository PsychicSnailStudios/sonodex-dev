<script lang="ts">
	import { untrack } from "svelte";
	import { invoke, convertFileSrc } from "@tauri-apps/api/core";

	import { Music4, User, DiscAlbum } from "lucide-svelte";

	import type { AudioCatagories, Track } from "$ts/util/types";
	import { artworkCache, artworkInflight } from "$ts/library/artworkLoader";

	let { uid, size = null, type = "track", previewPath = null }: {
		uid: string;
		size?: number | null;
		type?: AudioCatagories;
		previewPath?: string | null
	} = $props();

	let artworkUrl: string | null = $state(null);
	let loaded = $state(false);
	let el: HTMLDivElement;
	let fetchedKey = $state<string | null>(null);

	// artwork_path and artwork_thumb are fetched on demand alongside artwork blob.
	// We no longer read from library maps; we rely solely on the cache + invoke.
	const commandMap: Partial<Record<AudioCatagories, string>> = {
		track: "get_track_artwork",
		album: "get_album_artwork",
		artist: "get_artist_profile_art",
		playlist: "get_playlist_artwork",
	};

	const pathCommandMap: Partial<Record<AudioCatagories, string>> = {
		track: "get_track",
		album: "get_album",
		artist: "get_artist",
		playlist: "get_playlist",
	};

	// thumb is fetched once per uid/type and stored in a local map to avoid
	// re-invoking on every render. It's only used for the blur-up placeholder.
	const thumbCache = new Map<string, string | null>();
	let thumb = $state<string | null>(null);

	async function fetchThumb(cacheKey: string) {
		if (thumbCache.has(cacheKey)) {
			thumb = thumbCache.get(cacheKey) ?? null;
			return;
		}
		const cmd = pathCommandMap[type];
		if (!cmd) return;
		try {
			const entity = await invoke<any>(cmd, { uid });
			const t = entity?.artwork_thumb ?? null;
			thumbCache.set(cacheKey, t);
			thumb = t;
		} catch {
			thumbCache.set(cacheKey, null);
		}
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
		thumb = null;
		fetchThumb(currentKey);

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
					const command = commandMap[currentType];
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
	class="rounded-sm overflow-hidden relative bg-muted flex-shrink-0 w-full aspect-square">

	{#if thumb && !loaded}
		<img
			src={thumb}
			alt=""
			class="absolute inset-0 w-full h-full object-cover"
			style="filter: blur(4px); transform: scale(1.1);"
		/>
	{/if}

	{#if artworkUrl}
		<img
			src={artworkUrl}
			alt=""
			class="absolute inset-0 w-full h-full object-cover transition-opacity duration-500 aspect-square"
			class:opacity-0={!loaded}
			onload={() => loaded = true}
		/>
	{:else if !thumb}
		<div class="absolute inset-0 flex items-center justify-center">
			{#if type === "track"}
				<Music4 class="text-muted-foreground" />
			{:else if type === "album"}
				<DiscAlbum class="text-muted-foreground" />
			{:else if type === "artist"}
				<User class="text-muted-foreground" />
			{:else if type === "playlist"}
				<slot />
			{/if}
		</div>
	{/if}
</div>