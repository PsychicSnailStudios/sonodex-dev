<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';

  import { CircleSmall, CircleDashed, Minus, X } from 'lucide-svelte';

  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  const appWindow = getCurrentWindow();
  let isMaximized = $state(false);

  async function startDrag() {
    await appWindow.startDragging();
  }

  async function minimize() {
    await appWindow.minimize();
  }

  async function maximize() {
    const isMaximized = await appWindow.isMaximized();
    if (isMaximized) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  }

  async function close() {
    await appWindow.close();
  }

  onMount(async () => {
    isMaximized = await appWindow.isMaximized();

    await appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    });
  });

</script>

<div
  class="flex h-9 w-full select-none items-center justify-between bg-background px-4"
  role="presentation"
  tabindex="-1"
  onmousedown={startDrag}>

  <DropdownMenu.Root>
    <DropdownMenu.Trigger>...</DropdownMenu.Trigger>
    <DropdownMenu.Content>
      <DropdownMenu.Group>
        <DropdownMenu.Item>Settings</DropdownMenu.Item>
      </DropdownMenu.Group>
    </DropdownMenu.Content>
  </DropdownMenu.Root>

  <div class="flex items-center gap-1" role="presentation" onmousedown={(e) => e.stopPropagation()}>
    
    <button
      onclick={minimize}
      class="flex h-7 w-7 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
      aria-label="Minimize"
    >
      <Minus size={16} />
    </button>

    <button
      onclick={maximize}
      class="flex h-7 w-7 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
      aria-label="Maximize"
    >
      {#if isMaximized}
      <CircleDashed size={12} />
      {:else}
      <CircleSmall size={16} />
      {/if}
    </button>

    <button
      onclick={close}
      class="flex h-7 w-7 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-destructive hover:text-destructive-foreground"
      aria-label="Close"
    >
      <X size={16} />
    </button>

  </div>

</div>