<script lang="ts">
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";

	type Artist = {
		name: string;
	};

	let { artists }: { artists: Artist[] } = $props();
</script>

<div class="flex flex-col h-full w-full overflow-hidden">
	<ScrollArea class="min-h-0 min-w-0">

		<div class="app-music-grid grid gap-2 p-3">

			{#each artists as artist (artist.name)}
				<div class="flex flex-col items-center gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
					<div class="w-full aspect-square rounded-full bg-muted flex items-center justify-center overflow-hidden">
						<svg class="w-10 h-10 text-muted-foreground/40" viewBox="0 0 24 24" fill="currentColor">
							<path d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z"/>
						</svg>
					</div>
					<div class="min-w-0 w-full text-center">
						<p class="text-sm font-medium truncate">{artist.name}</p>
					</div>
				</div>
			{/each}

		</div>

	</ScrollArea>
</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>