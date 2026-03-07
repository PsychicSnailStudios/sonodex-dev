<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import VirtualList from "svelte-virtual-list";

  let paths: { id: number; path: string }[] = [];
  let tracks: any[] = [];
  let newPath = "";
  let status = "";
  let loading = false;
  let scanProgress = 0;
  let scanTotal = 0;

  async function loadPaths() {
    paths = await invoke("get_paths");
  }

  async function loadTracks() {
    tracks = await invoke("get_tracks");
  }

  async function addPath() {
    if (!newPath.trim()) return;
    loading = true;
    status = "Scanning...";
    scanProgress = 0;
    scanTotal = 0;
    try {
      await invoke("add_path", { path: newPath.trim() });
      newPath = "";
      await loadPaths();
    } catch (e) {
      status = `Error: ${e}`;
      loading = false;
    }
  }

  async function removePath(path: string) {
    await invoke("remove_path", { path });
    await loadPaths();
    await loadTracks();
    status = `Removed ${path}`;
  }

  async function rescan() {
    loading = true;
    status = "Rescanning...";
    scanProgress = 0;
    scanTotal = 0;
    await invoke("rescan");
  }

  onMount(async () => {
    await loadPaths();
    await loadTracks();

    await listen("scan:progress", async (event: any) => {
      scanProgress = event.payload.scanned;
      scanTotal = event.payload.total;
      status = `Scanning... ${scanProgress} / ${scanTotal}`;
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

    await listen("scan:error", (event: any) => {
        status = `Scan error: ${event.payload}`;
        loading = false;
    });
  });
</script>

<div class="p-6 space-y-6 max-w-4xl mx-auto">
  <h1 class="text-2xl font-bold">Sonodex Debug</h1>

  <div class="space-y-2">
    <h2 class="text-lg font-semibold">Add Library Path</h2>
    <div class="flex gap-2">
      <input
        bind:value={newPath}
        placeholder="C:\Music or \\NAS\Music"
        class="flex-1 border rounded px-3 py-2 text-sm bg-background"
      />
      <Button onclick={addPath} disabled={loading}>Add & Scan</Button>
    </div>
  </div>

  <div class="space-y-2">
    <h2 class="text-lg font-semibold">Watched Paths ({paths.length})</h2>
    {#each paths as p}
      <div class="flex items-center justify-between border rounded px-3 py-2 text-sm">
        <span>{p.path}</span>
        <Button variant="destructive" onclick={() => removePath(p.path)}>Remove</Button>
      </div>
    {:else}
      <p class="text-sm text-muted-foreground">No paths added yet.</p>
    {/each}
  </div>

  <div class="flex items-center gap-4">
    <Button onclick={rescan} disabled={loading}>Rescan All</Button>
    {#if status}
      <span class="text-sm text-muted-foreground">{status}</span>
    {/if}
  </div>

  {#if loading && scanTotal > 0}
    <div class="space-y-1">
      <div class="w-full bg-muted rounded-full h-2">
        <div
          class="bg-primary h-2 rounded-full transition-all"
          style="width: {Math.round((scanProgress / scanTotal) * 100)}%"
        ></div>
      </div>
      <p class="text-xs text-muted-foreground">{scanProgress} / {scanTotal} files</p>
    </div>
  {/if}

  <div class="space-y-2">
  <h2 class="text-lg font-semibold">Tracks ({tracks.length})</h2>
  <div class="border rounded" style="height: 400px;">
    <div class="grid grid-cols-4 bg-muted sticky top-0 text-sm font-medium">
      <span class="px-3 py-2">Title</span>
      <span class="px-3 py-2">Artist</span>
      <span class="px-3 py-2">Album</span>
      <span class="px-3 py-2">Duration</span>
    </div>
    <VirtualList items={tracks} let:item style="height: 360px;">
      <div class="grid grid-cols-4 text-sm border-t hover:bg-muted/50">
        <span class="px-3 py-2 truncate">{item.title ?? "Unknown"}</span>
        <span class="px-3 py-2 truncate">{item.artist ?? "Unknown"}</span>
        <span class="px-3 py-2 truncate">{item.album ?? "Unknown"}</span>
        <span class="px-3 py-2">{item.duration_ms ? Math.round(item.duration_ms / 1000) + "s" : "—"}</span>
      </div>
    </VirtualList>
  </div>
</div>
</div>