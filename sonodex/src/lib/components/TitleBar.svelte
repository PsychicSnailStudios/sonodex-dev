<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = getCurrentWindow();

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
</script>

<div
  class="flex h-9 w-full select-none items-center justify-between bg-background px-4"
  onmousedown={startDrag}
>
  <span class="text-sm font-medium text-foreground pointer-events-none">sonodex</span>

  <div class="flex items-center gap-1" onmousedown={(e) => e.stopPropagation()}>
    <button
      onclick={minimize}
      class="flex h-7 w-7 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
      aria-label="Minimize"
    >
      <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
        <rect width="10" height="1" />
      </svg>
    </button>

    <button
      onclick={maximize}
      class="flex h-7 w-7 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
      aria-label="Maximize"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
        <rect x="0.5" y="0.5" width="9" height="9" />
      </svg>
    </button>

    <button
      onclick={close}
      class="flex h-7 w-7 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-destructive hover:text-destructive-foreground"
      aria-label="Close"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5">
        <line x1="0" y1="0" x2="10" y2="10" />
        <line x1="10" y1="0" x2="0" y2="10" />
      </svg>
    </button>
  </div>
</div>