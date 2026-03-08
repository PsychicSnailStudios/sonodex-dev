<script lang="ts">
  import AudioCard from "$lib/components/app/AudioCard.svelte";

  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";

	type Album = {
		name: string;
		artist: string | null;
		year: string | null;
		artworkTrackId: number;
	};

	let { albums }: { albums: Album[] } = $props();
</script>

<div class="flex flex-col h-full w-full overflow-hidden">
  <ScrollArea class="min-h-0 min-w-0">
    
    <div class="app-music-grid grid gap-2 p-3">

      {#each albums as album (album.name)}
        <AudioCard {album} />
      {/each}
    
    </div>

  </ScrollArea>
</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
</style>