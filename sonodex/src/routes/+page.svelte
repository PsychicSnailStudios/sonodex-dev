<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";

  let paths: { id: number; path: string }[] = [];
  let tracks: any[] = [];
  let newPath = "";
  let status = "";
  let loading = false;

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
    try {
      await invoke("add_path", { path: newPath.trim() });
      newPath = "";
      await loadPaths();
      await loadTracks();
      status = `Done. ${tracks.length} tracks in library.`;
    } catch (e) {
      status = `Error: ${e}`;
    }
    loading = false;
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
    await invoke("rescan");
    await loadTracks();
    status = `Done. ${tracks.length} tracks in library.`;
    loading = false;
  }

  onMount(async () => {
    await loadPaths();
    await loadTracks();
    await listen("library:updated", async () => {
      await loadTracks();
      status = `Library updated. ${tracks.length} tracks.`;
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

  <div class="space-y-2">
    <h2 class="text-lg font-semibold">Tracks ({tracks.length})</h2>
    <div class="border rounded overflow-auto max-h-96">
      <table class="w-full text-sm">
        <thead class="bg-muted sticky top-0">
          <tr>
            <th class="text-left px-3 py-2">Title</th>
            <th class="text-left px-3 py-2">Artist</th>
            <th class="text-left px-3 py-2">Album</th>
            <th class="text-left px-3 py-2">Duration</th>
          </tr>
        </thead>
        <tbody>
          {#each tracks as track}
            <tr class="border-t hover:bg-muted/50">
              <td class="px-3 py-2">{track.title ?? "Unknown"}</td>
              <td class="px-3 py-2">{track.artist ?? "Unknown"}</td>
              <td class="px-3 py-2">{track.album ?? "Unknown"}</td>
              <td class="px-3 py-2">{track.duration_ms ? Math.round(track.duration_ms / 1000) + "s" : "—"}</td>
            </tr>
          {:else}
            <tr>
              <td colspan="4" class="px-3 py-2 text-muted-foreground">No tracks yet.</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>