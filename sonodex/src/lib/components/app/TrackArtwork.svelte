<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Skeleton } from "$lib/components/ui/skeleton/index.js";

	let { id, width = 40, height = 40, type = "track" }: { id: number; width?: number; height?: number; type?: "track" | "album" | "artist" | "playlist" } = $props();

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

<div bind:this={el} style="width: {width}px; height: {height}px;" class="rounded-sm overflow-hidden relative bg-muted flex-shrink-0">
	{#if artworkUrl}
		<img
			src={artworkUrl}
			alt=""
			class="absolute inset-0 w-full h-full object-cover transition-opacity duration-600"
			class:opacity-0={!loaded}
			onload={() => loaded = true}
		/>
		{#if !loaded}
			<Skeleton class="w-full h-full" />
		{/if}
	{:else}
		<div class="absolute inset-0 flex items-center justify-center">
			<svg class="w-4 h-4 text-muted-foreground/40" viewBox="0 0 24 24" fill="currentColor">
				<path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6z"/>
			</svg>
		</div>
	{/if}
</div>