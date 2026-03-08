<script lang="ts">
  import './layout.css';
  import favicon from '$lib/assets/favicon.svg';
  import TitleBar from '$lib/components/TitleBar.svelte';
  let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex h-screen flex-col overflow-hidden">
  <TitleBar />
  <main class="flex-1 overflow-auto">
    {@render children()}
  </main>
</div>