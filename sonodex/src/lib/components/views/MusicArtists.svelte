<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
  import VirtualList from "svelte-virtual-list";

  import AudioCard from "$lib/components/app/AudioCard.svelte";

  let editingTrack: any = null;

  let activeTab = $state("album");

  let paths: { id: number; path: string }[] = [];
  let tracks: any[] = [];
  let duplicateGroups: { tracks: any[] }[] = [];

  async function loadPaths() {
    paths = await invoke("get_paths");
  }

  async function loadTracks() {
    tracks = await invoke("get_tracks");
  }

  async function loadDuplicates() {
    duplicateGroups = await invoke("get_duplicates");
    duplicatesCount = duplicateGroups.length;
  }

  async function removeFromLibrary(id: number) {
    await invoke("remove_track_from_library", { id });
    await loadDuplicates();
  }

  async function deleteFile(id: number, path: string) {
    await invoke("delete_track_file", { id, path });
    await loadDuplicates();
  }

  function parseJson(val: string | null): string[] {
    if (!val) return [];
    try { return JSON.parse(val); } catch { return []; }
  }

  function formatAlbums(val: string | null): string {
    const albums = parseJson(val);
    return albums.map((a: any) => typeof a === "string" ? a : a.name).join(", ") || "—";
  }

  onMount(async () => {
    await loadPaths();
    await loadTracks();
    await loadSettings();
    await loadDuplicates();

    await listen("library:updated", async () => {
      console.log("library:updated received");
      await loadTracks();
      await loadDuplicates();
      status = `Library updated. ${tracks.length} tracks.`;
    });

    await listen("scan:done", async () => {
      await loadTracks();
      status = `Done. ${tracks.length} tracks in library.`;
      loading = false;
      scanProgress = 0;
      scanTotal = 0;
    });

    await listen("library:updated", async () => {
      await loadTracks();
      status = `Library updated. ${tracks.length} tracks.`;
    });
  });
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">
	
	
	<ScrollArea class="min-h-0 min-w-0">
		
		<div class="app-music-grid grid gap-2">

			{#each tracks as track (track.id)}
				<AudioCard {track} />
			{/each}
		
		</div>

	</ScrollArea>

</div>

<style>

.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}

</style>