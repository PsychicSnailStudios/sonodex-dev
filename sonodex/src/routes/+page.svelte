<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";
	import { loadLibrary } from "$lib/library.svelte";
  import { selection } from "$lib/session.svelte";

  import * as Resizable from "$lib/components/ui/resizable/index.js";
  import { Button } from "$lib/components/ui/button/index.js";

  import HomeVeiw from "$lib/components/views/Home.svelte";
  import MusicView from "$lib/components/views/MusicLibrary.svelte";
  import PlaylistsView from "$lib/components/views/Playlists.svelte";
  import SettingsVeiw from "$lib/components/views/Settings.svelte";
  import PlayControls from "$lib/components/PlayControls.svelte";
  import NowPlaying from "$lib/components/NowPlaying.svelte";

  import AlbumDisplay from "$lib/components/views/AlbumDisplay.svelte";
  import ArtistDisplay from "$lib/components/views/ArtistDisplay.svelte";
  import PlaylistDisplay from "$lib/components/views/PlaylistDisplay.svelte";
  import TrackDisplay from "$lib/components/views/TrackDisplay.svelte";

  import { House, Music, ListMusic, Settings } from "lucide-svelte"



  const VIEW_TABS = [
    { value: "home", label: "Home", icon: House },
    { value: "music", label: "Music Library", icon: Music },
    { value: "playlists", label: "Playlists", icon: ListMusic },
    { value: "settings", label: "Settings", icon: Settings },
  ];

  let activeView = $state("home");

  onMount(() => {
		loadLibrary();
	});
  
</script>

<div class="app-wrapper grid gap-2 p-2 overflow-hidden">

  <div class="app-sidebar grid gap-1">

    <div>
      <!-- scan progress -->
    </div>

    <div class="app-nav bg-muted flex flex-col p-2 gap-1 rounded-md">

      {#each VIEW_TABS as tab}
        <Button variant="{activeView === tab.value ? 'default' : 'outline'}" onclick={() => activeView = tab.value}>
          <svelte:component this={tab.icon} />
          {tab.label}
        </Button>
      {/each}

    </div>

    <NowPlaying />

  </div>

  <div class="app-body h-full w-full overflow-hidden grid gap-2">

    <div class="app-views h-full w-full overflow-hidden flex gap-2">

      <Resizable.PaneGroup direction="horizontal" class="h-full w-full">
        <Resizable.Pane >
          {#if activeView === "home"}
            <HomeVeiw />
          {:else if activeView === "music"}
            <MusicView />
          {:else if activeView === "playlists"}
            <PlaylistsView />
          {:else if activeView === "settings"}
            <SettingsVeiw />
          {/if}
        </Resizable.Pane>

        {#if selection.type !== "none"}
        <Resizable.Handle class="px-1 opacity-0" />
        
        <Resizable.Pane defaultSize={50}>
          {#if selection.type === "album"}
            <AlbumDisplay />
          {/if}
          {#if selection.type === "track"}
            <TrackDisplay />
          {/if}
          {#if selection.type === "playlist"}
            <PlaylistDisplay />
          {/if}
          {#if selection.type === "artist"}
            <ArtistDisplay />
          {/if}
        </Resizable.Pane>
        {/if}
      </Resizable.PaneGroup>

    </div>

    <PlayControls />

  </div>

</div>

<style>
  .app-wrapper {
    grid-template-columns: minmax(18rem, 25rem) minmax(50%, 85%);
    height: 100%;
  }

  .app-sidebar {
    grid-template-rows: auto 1fr auto;
    flex-direction: column;
    min-width: 10rem;
    max-width: 30rem;
  }

  .app-body {
    grid-template-rows: 1fr auto;
    flex-direction: column;
  }
</style>