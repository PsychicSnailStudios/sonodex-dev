<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import VirtualList from "svelte-virtual-list";
  import { open } from "@tauri-apps/plugin-dialog";
  import TrackEditModal from "$lib/components/TrackEditModal.svelte";

  let editingTrack: any = null;

  let activeTab: "library" | "settings" | "duplicates" = "library";

  let paths: { id: number; path: string }[] = [];
  let tracks: any[] = [];
  let newPath = "";
  let status = "";
  let loading = false;
  let scanProgress = 0;
  let scanTotal = 0;
  let duplicateGroups: { tracks: any[] }[] = [];
  let duplicatesCount = 0;

  let darkMode = document.documentElement.classList.contains("dark");

  function toggleDarkMode(enabled: boolean) {
    darkMode = enabled;
    document.documentElement.classList.toggle("dark", enabled);
    saveSetting("dark_mode", enabled ? "true" : "false");
  }

  let settings: Record<string, string> = {};

  const PRIORITY_OPTIONS = [
    { value: "tag", label: "File Tag" },
    { value: "filename", label: "Filename (override)" },
    { value: "filename_fallback", label: "Filename (fallback)" },
  ];

  const SETTING_LABELS: Record<string, string> = {
    filename_priority_title: "Title",
    filename_priority_artist: "Artist",
    filename_priority_album: "Album",
    filename_priority_year: "Year",
    filename_custom_pattern: "Custom Filename Pattern",
  };

  async function browsePath() {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected) {
      newPath = selected as string;
    }
  }

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

  async function loadSettings() {
    const raw: { key: string; value: string }[] = await invoke("get_settings");
    settings = Object.fromEntries(raw.map((s) => [s.key, s.value]));
  }

  async function saveSetting(key: string, value: string) {
    settings[key] = value;
    await invoke("save_setting", { key, value });
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

    darkMode = (settings["dark_mode"] ?? "false") === "true";
    document.documentElement.classList.toggle("dark", darkMode);

    await listen("library:updated", async () => {
      console.log("library:updated received");
      await loadTracks();
      await loadDuplicates();
      status = `Library updated. ${tracks.length} tracks.`;
    });

    await listen("scan:progress", async (event: any) => {
      scanProgress = event.payload.scanned;
      scanTotal = event.payload.total;
      status = `Scanning... ${scanProgress} / ${scanTotal}`;
    });

    await listen("duplicates:found", async () => {
      await loadDuplicates();
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

<div class="flex flex-col h-screen">
  <div class="border-b px-6 pt-4 flex gap-4">
    <h1 class="text-xl font-bold mr-4">Sonodex Debug</h1>
    <button
      class="pb-3 px-1 text-sm border-b-2 transition-colors {activeTab === 'library' ? 'border-primary font-medium' : 'border-transparent text-muted-foreground'}"
      onclick={() => activeTab = "library"}
    >Library</button>
    <button
      class="pb-3 px-1 text-sm border-b-2 transition-colors {activeTab === 'settings' ? 'border-primary font-medium' : 'border-transparent text-muted-foreground'}"
      onclick={() => activeTab = "settings"}
    >Settings</button>
    <button
      class="pb-3 px-1 text-sm border-b-2 transition-colors {activeTab === 'duplicates' ? 'border-primary font-medium' : 'border-transparent text-muted-foreground'}"
      onclick={() => activeTab = "duplicates"}
    >Duplicates</button>
  </div>

  {#if activeTab === "library"}
    <div class="p-6 space-y-6 flex-1 overflow-auto">
      
      <div class="space-y-1">
        <h2 class="text-sm font-semibold">Tracks ({tracks.length})</h2>
        <div class="border rounded">
          <div class="grid text-xs font-medium bg-muted px-3 py-2" style="grid-template-columns: 2fr 1fr 1fr 1fr auto;">
            <span>Title</span>
            <span>Main Artist</span>
            <span>Album(s)</span>
            <span>Other Artists</span>
            <span></span>
          </div>
          <div style="height: 440px;" class="overflow-hidden">
            <VirtualList items={tracks} itemHeight={36} let:item>
              <div class="grid text-sm border-t hover:bg-muted/50 px-3 py-2" style="grid-template-columns: 2fr 1fr 1fr 1fr auto;">
                <span class="truncate pr-2">{item.title ?? "—"}</span>
                <span class="truncate pr-2">{item.album_artist ?? parseJson(item.artists)[0] ?? "—"}</span>
                <span class="truncate pr-2">{formatAlbums(item.albums)}</span>
                <span class="truncate text-muted-foreground text-xs">
                  {parseJson(item.artists).filter((a: string) => a !== item.album_artist).join(", ") || "—"}
                </span>
                <button class="text-muted-foreground hover:text-foreground text-xs px-1" onclick={() => editingTrack = item}>✎</button>
              </div>
            </VirtualList>
          </div>
        </div>
        {#if tracks.length > 0}
          <pre class="text-xs bg-muted p-2 rounded overflow-auto max-h-80">{JSON.stringify(tracks[0], null, 2)}</pre>
        {/if}
      </div>
    </div>

  {:else if activeTab === "settings"}
  <div class="p-6 space-y-6 flex-1">
      <div class="space-y-2">
        <h2 class="text-sm font-semibold">Add Library Path</h2>
        <div class="flex gap-2">
          <input
            bind:value={newPath}
            placeholder="C:\Music or \\NAS\Music"
            class="flex-1 border rounded px-3 py-2 text-sm bg-background"
          />
          <Button variant="outline" onclick={browsePath}>Browse</Button>
          <Button onclick={addPath} disabled={loading}>Add & Scan</Button>
        </div>
      </div>

      <div class="space-y-2">
        <h2 class="text-sm font-semibold">Watched Paths ({paths.length})</h2>
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
      </div>

      <div class="p-6 space-y-6 max-w-xl">
          <div class="flex items-center justify-between">
        <div>
          <h2 class="text-sm font-semibold">Appearance</h2>
          <p class="text-xs text-muted-foreground">Toggle dark mode.</p>
        </div>
        <input
          type="checkbox"
          checked={darkMode}
          onchange={(e) => toggleDarkMode((e.target as HTMLInputElement).checked)}
        />
      </div>
      <h2 class="text-sm font-semibold">Metadata Priority</h2>
      <p class="text-xs text-muted-foreground">Choose whether each field should prefer file tags or be parsed from the filename.</p>

      {#each ["filename_priority_title", "filename_priority_artist", "filename_priority_album", "filename_priority_year"] as key}
        <div class="flex items-center justify-between gap-4">
          <label class="text-sm w-24">{SETTING_LABELS[key]}</label>
          <select
            class="flex-1 border rounded px-3 py-2 text-sm bg-background"
            value={settings[key] ?? "tag"}
            onchange={(e) => saveSetting(key, (e.target as HTMLSelectElement).value)}
          >
            {#each PRIORITY_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>
      {/each}

      <div class="space-y-1">
        <label class="text-sm font-medium">Custom Filename Pattern</label>
        <p class="text-xs text-muted-foreground">Use tokens: {"{title}"} {"{artist}"} {"{album}"} {"{year}"}. Example: <code>{"{artist}"} - {"{year}"} - {"{title}"}</code></p>
        <input
          class="w-full border rounded px-3 py-2 text-sm bg-background"
          placeholder="{'{artist}'} - {'{album}'} - {'{year}'} - {'{title}'}"
          value={settings["filename_custom_pattern"] ?? ""}
          onchange={(e) => saveSetting("filename_custom_pattern", (e.target as HTMLInputElement).value)}
        />
      </div>

      <div class="space-y-2">
        <h2 class="text-sm font-semibold">Folder Path Fallback</h2>
        <p class="text-xs text-muted-foreground">If a field is missing from tags and filename, infer it from the folder structure (e.g. /Artist/Album/track).</p>

        {#each [
          { key: "folder_fallback_artist", label: "Artist" },
          { key: "folder_fallback_album", label: "Album" },
          { key: "folder_fallback_year", label: "Year" },
        ] as { key, label }}
          <div class="flex items-center justify-between gap-4">
            <label class="text-sm">{label}</label>
            <input
              type="checkbox"
              checked={settings[key] === "true"}
              onchange={(e) => saveSetting(key, (e.target as HTMLInputElement).checked ? "true" : "false")}
            />
          </div>
        {/each}
      </div>

      <Button onclick={rescan} disabled={loading}>Apply & Rescan</Button>
      {#if status}<span class="text-sm text-muted-foreground">{status}</span>{/if}
    </div>
    
    {:else if activeTab === "duplicates"}
      <div class="p-6 space-y-4 flex-1 overflow-auto">
        <div class="flex items-center justify-between">
          <h2 class="text-sm font-semibold">Duplicate Tracks ({duplicateGroups.length})</h2>
        </div>

        {#if duplicateGroups.length === 0}
          <p class="text-sm text-muted-foreground">No duplicates found.</p>
        {/if}

        {#each duplicateGroups as group}
          <div class="border rounded-lg overflow-hidden">
            <div class="bg-muted px-4 py-2 text-xs font-medium text-muted-foreground">
              {group.tracks[0].title ?? "Unknown"} — {group.tracks[0].album_artist ?? parseJson(group.tracks[0].artists)[0] ?? "Unknown"}
            </div>
            {#each group.tracks as track}
              <div class="grid gap-2 px-4 py-3 border-t first:border-t-0" style="grid-template-columns: 1fr auto auto;">
                <div class="space-y-0.5 min-w-0">
                  <p class="text-sm truncate">{track.path}</p>
                  <p class="text-xs text-muted-foreground">
                    {track.duration_ms ? (track.duration_ms / 1000).toFixed(1) + "s" : "—"}
                    {track.year ? "· " + track.year : ""}
                    {track.bpm ? "· " + track.bpm + " BPM" : ""}
                  </p>
                </div>
                <Button variant="outline" onclick={() => removeFromLibrary(track.id)}>Remove from library</Button>
                <Button variant="destructive" onclick={() => deleteFile(track.id, track.path)}>Delete file</Button>
              </div>
            {/each}
          </div>
        {/each}
      </div>

    {/if}

  <TrackEditModal
    track={editingTrack}
    onClose={() => editingTrack = null}
    onSaved={async () => { await loadTracks(); await loadDuplicates(); }}
  />
</div>