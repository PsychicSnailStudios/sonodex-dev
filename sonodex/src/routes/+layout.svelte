<script lang="ts">
  import './layout.css';
  import favicon from '$lib/assets/favicon.svg';
  import TitleBar from '$lib/components/app/TitleBar.svelte';
  import EditModal from '$lib/components/dialogs/edit-metadata/EditModal.svelte';
  import { ModeWatcher } from "mode-watcher";
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
</div>