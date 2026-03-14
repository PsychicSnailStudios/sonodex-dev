<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	import { Skeleton } from "$lib/components/ui/skeleton/index.js";

	import { Music4, User, DiscAlbum, ListMusic } from "lucide-svelte";



	let { id, size = 40, type = "track" }: { id: number; size?: number; type?: "track" | "album" | "artist" | "playlist" } = $props();

	let artworkUrl: string | null = $state(null);
	let loaded = $state(false);
	let el: HTMLDivElement;

	const commandMap = {
		track: "get_track_artwork",
		album: "get_album_artwork",
		artist: "get_artist_profile_art",
		playlist: "get_playlist_artwork",
	};

	$effect(() => {
		const observer = new IntersectionObserver(async ([entry]) => {
			if (entry.isIntersecting) {
				observer.disconnect();
				try {
					const bytes: number[] | null = await invoke(commandMap[type], { id });
					if (bytes) {
						const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
						artworkUrl = URL.createObjectURL(blob);
					}
				} catch {}
			}
		}, { rootMargin: "200px" });

		if (el) observer.observe(el);
		return () => observer.disconnect();
	});
</script>

<div bind:this={el} style="width: {size}px; height: {size}px;" class="rounded-sm overflow-hidden relative bg-muted flex-shrink-0">
	{#if artworkUrl}
		{#if !loaded}
			<Skeleton class="w-full h-full" />
		{/if}
		<img
			src={artworkUrl}
			alt=""
			class="absolute inset-0 w-full h-full object-cover transition-opacity duration-600"
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