<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	import { Skeleton } from "$lib/components/ui/skeleton/index.js";

	let { id }: { id: number } = $props();

	let artworkUrl: string | null = $state(null);
	let loaded = $state(false);
	let el: HTMLDivElement;

	$effect(() => {
		const observer = new IntersectionObserver(async ([entry]) => {
			if (entry.isIntersecting) {
				observer.disconnect();
				try {
					const bytes: number[] | null = await invoke("get_track_artwork", { id });
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

<div bind:this={el} class="w-10 h-10 rounded-sm overflow-hidden relative bg-muted flex-shrink-0">
	{#if artworkUrl}
		<img
			src={artworkUrl}
			alt=""
			class="absolute inset-0 w-full h-full object-cover transition-opacity duration-600"
			class:opacity-0={!loaded}
			onload={() => loaded = true}
		/>
		{#if !loaded}
			<Skeleton class="w-10 h-10 rounded-full" />
		{/if}
	{:else}
		<div class="absolute inset-0 flex items-center justify-center">
			<svg class="w-4 h-4 text-muted-foreground/40" viewBox="0 0 24 24" fill="currentColor">
				<path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6z"/>
			</svg>
		</div>
	{/if}
</div>