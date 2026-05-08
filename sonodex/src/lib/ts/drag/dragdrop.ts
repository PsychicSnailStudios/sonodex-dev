export { startDrag, endDrag } from "$ts/store/state_drag.svelte";

export function isDraggingFolderType(e: DragEvent): boolean {
	return e.dataTransfer?.types.includes("application/x-sonodex-folder") ?? false;
}
