<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button/index.js";

  export let track: any = null;
  export let onClose: () => void = () => {};
  export let onSaved: () => void = () => {};

  let title = "";
  let artists: string[] = [];
  let albumArtist = "";
  let albums: { name: string; track_number: number | string | null }[] = [];
  let year = "";
  let genres: string[] = [];
  let bpm: number | null = null;
  let rating: number | null = null;
  let tags: string[] = [];
  let enriching = false;
  let enrichStatus = "";

  async function enrichTrack() {
    enriching = true;
    enrichStatus = "";
    try {
      await invoke("enrich_track", { id: track.id });
      enrichStatus = "Done — reopen track to see updated values.";
      onSaved();
    } catch (e) {
      enrichStatus = `Not found: ${e}`;
    } finally {
      enriching = false;
    }
  }

  function parseJson(val: string | null): any[] {
    if (!val) return [];
    try { return JSON.parse(val); } catch { return []; }
  }

  $: if (track) {
    title = track.title ?? "";
    artists = parseJson(track.artists);
    albumArtist = track.album_artist ?? "";
    albums = parseJson(track.albums).map((a: any) =>
      typeof a === "string"
        ? { name: a, track_number: "" }
        : { name: a.name, track_number: a.track_number ?? "" }
    );
    year = track.year ?? "";
    genres = parseJson(track.genres);
    bpm = track.bpm ?? null;
    rating = track.rating ?? null;
    tags = parseJson(track.tags);
  }

  function addAlbum() {
    albums = [...albums, { name: "", track_number: null }];
  }

  function removeAlbum(i: number) {
    albums = albums.filter((_, idx) => idx !== i);
  }

  function addArtist() {
    artists = [...artists, ""];
  }

  function removeArtist(i: number) {
    artists = artists.filter((_, idx) => idx !== i);
  }

  function addGenre() {
    genres = [...genres, ""];
  }

  function removeGenre(i: number) {
    genres = genres.filter((_, idx) => idx !== i);
  }

  function addTag() {
    tags = [...tags, ""];
  }

  function removeTag(i: number) {
    tags = tags.filter((_, idx) => idx !== i);
  }

  function buildUpdate() {
    return {
      title: title || null,
      artists: JSON.stringify(artists.filter(a => a.trim())),
      album_artist: albumArtist || null,
      albums: JSON.stringify(
        albums
          .filter(a => a.name.trim())
          .map(a => ({
            name: a.name,
            track_number: a.track_number !== "" && a.track_number !== null
              ? Number(a.track_number)
              : null,
          }))
      ),
      year: year || null,
      genres: JSON.stringify(genres.filter(g => g.trim())),
      bpm: bpm ?? null,
      rating: rating ?? null,
      tags: JSON.stringify(tags.filter(t => t.trim())),
    };
  }

  async function saveToLibrary() {
    await invoke("update_track_metadata", { id: track.id, update: buildUpdate() });
    onSaved();
    onClose();
  }

  async function saveToFile() {
    await invoke("write_track_tags", { id: track.id, path: track.path, update: buildUpdate() });
    onSaved();
    onClose();
  }
</script>

{#if track}
  <div class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center" onclick={onClose}>
    <div class="bg-background border rounded-lg w-full max-w-xl max-h-[90vh] overflow-y-auto p-6 space-y-4" onclick={(e) => e.stopPropagation()}>
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold">Edit Track</h2>
        <button class="text-muted-foreground hover:text-foreground text-lg leading-none" onclick={onClose}>✕</button>
      </div>

      <div class="space-y-1">
        <label class="text-xs text-muted-foreground">Title</label>
        <input class="w-full border rounded px-3 py-2 text-sm bg-background" bind:value={title} />
      </div>

      <div class="space-y-1">
        <div class="flex items-center justify-between">
          <label class="text-xs text-muted-foreground">Artists</label>
          <button class="text-xs text-muted-foreground hover:text-foreground" onclick={addArtist}>+ Add</button>
        </div>
        {#each artists as artist, i}
          <div class="flex gap-2">
            <input class="flex-1 border rounded px-3 py-2 text-sm bg-background" bind:value={artists[i]} />
            <button class="text-xs text-muted-foreground hover:text-destructive" onclick={() => removeArtist(i)}>✕</button>
          </div>
        {/each}
      </div>

      <div class="space-y-1">
        <label class="text-xs text-muted-foreground">Album Artist</label>
        <input class="w-full border rounded px-3 py-2 text-sm bg-background" bind:value={albumArtist} />
      </div>

      <div class="space-y-1">
        <div class="flex items-center justify-between">
          <label class="text-xs text-muted-foreground">Albums</label>
          <button class="text-xs text-muted-foreground hover:text-foreground" onclick={addAlbum}>+ Add</button>
        </div>
        {#each albums as album, i}
          <div class="flex gap-2 items-center">
            <input class="flex-1 border rounded px-3 py-2 text-sm bg-background" placeholder="Album name" bind:value={albums[i].name} />
            <input class="w-20 border rounded px-3 py-2 text-sm bg-background" placeholder="#" type="number" bind:value={albums[i].track_number} />
            <button class="text-xs text-muted-foreground hover:text-destructive" onclick={() => removeAlbum(i)}>✕</button>
          </div>
        {/each}
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground">Year</label>
          <input class="w-full border rounded px-3 py-2 text-sm bg-background" bind:value={year} />
        </div>
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground">Rating</label>
          <input class="w-full border rounded px-3 py-2 text-sm bg-background" type="number" min="0" max="10" step="0.1" bind:value={rating} />
        </div>
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground">BPM</label>
          <input class="w-full border rounded px-3 py-2 text-sm bg-background" type="number" bind:value={bpm} />
        </div>
      </div>

      <div class="space-y-1">
        <div class="flex items-center justify-between">
          <label class="text-xs text-muted-foreground">Genres</label>
          <button class="text-xs text-muted-foreground hover:text-foreground" onclick={addGenre}>+ Add</button>
        </div>
        {#each genres as genre, i}
          <div class="flex gap-2">
            <input class="flex-1 border rounded px-3 py-2 text-sm bg-background" bind:value={genres[i]} />
            <button class="text-xs text-muted-foreground hover:text-destructive" onclick={() => removeGenre(i)}>✕</button>
          </div>
        {/each}
      </div>

      <div class="space-y-1">
        <div class="flex items-center justify-between">
          <label class="text-xs text-muted-foreground">Tags</label>
          <button class="text-xs text-muted-foreground hover:text-foreground" onclick={addTag}>+ Add</button>
        </div>
        {#each tags as tag, i}
          <div class="flex gap-2">
            <input class="flex-1 border rounded px-3 py-2 text-sm bg-background" bind:value={tags[i]} />
            <button class="text-xs text-muted-foreground hover:text-destructive" onclick={() => removeTag(i)}>✕</button>
          </div>
        {/each}
      </div>

      <div class="border-t pt-4 space-y-2">
        <div class="flex items-center gap-3">
          <Button variant="outline" onclick={enrichTrack} disabled={enriching}>
            {enriching ? "Fetching..." : "Enrich from Online APIs"}
          </Button>
          {#if enrichStatus}
            <span class="text-xs text-muted-foreground">{enrichStatus}</span>
          {/if}
        </div>
        <p class="text-xs text-muted-foreground">Fetches metadata from MusicBrainz / TheAudioDB using your priority settings. Saves directly to library.</p>
      </div>

      <div class="flex gap-2 pt-2">
        <Button variant="outline" onclick={saveToLibrary}>Save to Library</Button>
        <Button onclick={saveToFile}>Save to File & Library</Button>
        <Button variant="ghost" onclick={onClose}>Cancel</Button>
      </div>
    </div>
  </div>
{/if}