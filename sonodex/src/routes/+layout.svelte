<script lang="ts">
  import { saveWindowState, restoreStateCurrent, StateFlags } from "@tauri-apps/plugin-window-state";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  
  import './layout.css';
  import favicon from '$lib/assets/favicon.svg';

  import TitleBar from '$lib/components/app/title-bar/TitleBar.svelte';
  import EditModal from '$lib/components/dialogs/edit-metadata/EditModal.svelte';

	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { ModeWatcher } from "mode-watcher";

  import WarningDialog from "$lib/components/dialogs/WarnDialog.svelte";
	import { dialogState, confirmDialog, cancelDialog } from "$lib/ts/app/dialogManager.svelte";
    import { onMount } from "svelte";

  let { children } = $props();

  onMount(async () => {
    await restoreStateCurrent(StateFlags.ALL);

    const appWindow = getCurrentWindow();
    await appWindow.onCloseRequested(async () => {
        await saveWindowState(StateFlags.ALL);
    });
  })
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex h-screen flex-col overflow-hidden main-app-bg">
  <ModeWatcher />
  <TitleBar />
  <Tooltip.Provider>
    <main class="h-screen overflow-hidden">
      {@render children()}
    </main>
  </Tooltip.Provider>
  <EditModal />
  <Toaster position="top-center" />
  <WarningDialog bind:open={dialogState.open} title={dialogState.title} description={dialogState.description} onconfirm={confirmDialog} oncancel={cancelDialog} />
</div>

<style>
  .main-app-bg {
    background-color: oklch(from var(--background) l c h / 95%);
    backdrop-filter: blur(100px);
  }
</style>
