<script lang="ts">
  
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";

  import MusicAlbums from "$lib/components/views/MusicAlbums.svelte";
  import MusicArtists from "$lib/components/views/MusicArtists.svelte";
  import MusicTracks from "$lib/components/views/MusicTracks.svelte";

  let activeTab = $state("album");

</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">
	
	<div class="flex justify-between items-center gap-2">
		<span>Music</span>

		<ToggleGroup.Root type="single" value={activeTab} onValueChange={(v) => { if (v) activeTab = v }} class="flex gap-2">
			<ToggleGroup.Item value="artist" aria-label="Toggle artist" class="rounded-md">
				Artists
			</ToggleGroup.Item>
			<ToggleGroup.Item value="album" aria-label="Toggle album">
				Albums
			</ToggleGroup.Item>
			<ToggleGroup.Item value="track" aria-label="Toggle track">
				Tracks
			</ToggleGroup.Item>
		</ToggleGroup.Root>
	</div>

	<ScrollArea class="min-h-0 min-w-0">
		
		{#if activeTab === "artist"}
      <MusicArtists />
    {:else if activeTab === "album"}
      <MusicAlbums />
    {:else if activeTab === "track"}
      <MusicTracks />
    {/if}

	</ScrollArea>

</div>

<style>

.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}

</style>