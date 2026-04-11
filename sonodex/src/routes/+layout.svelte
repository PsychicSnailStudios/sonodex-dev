<script lang="ts">
  import './layout.css';
  import favicon from '$lib/assets/favicon.svg';

  import TitleBar from '$lib/components/app/title-bar/TitleBar.svelte';
  import EditModal from '$lib/components/dialogs/edit-metadata/EditModal.svelte';

  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { ModeWatcher } from "mode-watcher";

  import WarningDialog from "$lib/components/dialogs/WarnDialog.svelte";
	import { dialogState, confirmDialog, cancelDialog } from "$lib/ts/app/dialogManager.svelte";

  let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex h-screen flex-col overflow-hidden">
  <ModeWatcher />
  <TitleBar />
  <main class="h-screen overflow-hidden">
    {@render children()}
  </main>
  <EditModal />
  <Toaster position="top-center" />
  <WarningDialog bind:open={dialogState.open} title={dialogState.title} description={dialogState.description} onconfirm={confirmDialog} oncancel={cancelDialog} />
</div>
