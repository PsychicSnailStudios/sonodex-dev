<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	let { album } = $props<{ album: { name: string; artist: string | null; year: string | null; artworkTrackId: number } }>();

	let artworkUrl: string | null = $state(null);
	let loaded = $state(false);
	let cardEl: HTMLDivElement;

	const artist = $derived(album.artist ?? "Unknown Artist");
	const title = $derived(album.name);

	$effect(() => {
		const observer = new IntersectionObserver(async ([entry]) => {
			if (entry.isIntersecting) {
				observer.disconnect();
				try {
					const bytes: number[] | null = await invoke("get_track_artwork", { id: album.artworkTrackId });
					if (bytes) {
						const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
						artworkUrl = URL.createObjectURL(blob);
					}
				} catch {}
			}
		}, { rootMargin: "100px" });

		if (cardEl) observer.observe(cardEl);
		return () => observer.disconnect();
	});
</script>

<div bind:this={cardEl} class="flex flex-col gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
	
	<div class="aspect-square w-full rounded-sm overflow-hidden relative bg-black">
		
		{#if artworkUrl}
			<img
				src={artworkUrl}
				alt=""
				class="absolute inset-0 w-full h-full object-cover scale-110 blur-lg opacity-60"
			/>
			<img
				src={artworkUrl}
				alt={title}
				class="absolute inset-0 w-full h-full object-cover transition-opacity duration-600"
				class:opacity-0={!loaded}
				onload={() => loaded = true}
			/>
		{:else}
			<div class="absolute inset-0 flex items-center justify-center bg-black">
				<svg class="w-10 h-10 text-white/20" viewBox="0 0 24 24" fill="currentColor">
					<path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6z"/>
				</svg>
			</div>
		{/if}

	</div>

	<div class="min-w-0">
		<p class="text-sm font-medium truncate">{title}</p>
		<p class="text-xs text-muted-foreground truncate">{artist}</p>
	</div>

</div>