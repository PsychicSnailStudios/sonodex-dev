<script lang="ts">
	// COMPONENTS
	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";

	// CUSTOM COMPONENTS
	import QueueTrackItem from "$lib/components/app/now-playing/QueueTrackItem.svelte";

	// SCRIPTS
	import {
		clearQueue,
		getQueuedTracks,
		player,
		reorderQueue,
		insertIntoQueue,
	} from "$lib/ts/audio/audioManager.svelte";
	import { dragState, endDrag } from "$lib/ts/app-states/state_drag.svelte";
	import { library } from "$lib/ts/library.svelte";

	// VARIABLES
	let upcomingTracks = $derived(getQueuedTracks());

	let dragOverIndex = $state<number | null>(null);
	let dragOverPosition = $state<"above" | "below">("below");
	let dragOverAppend = $state(false);

	// FUNCTIONS
	function formatTotalRemainingTime(): string {
		const queueMs = getQueuedTracks().reduce((acc, t) => acc + (t.duration_ms ?? 0), 0);
		const currentRemaining = (player.duration - player.currentTime) * 1000;
		const totalMs = queueMs + Math.max(0, currentRemaining);

		const totalSecs = Math.floor(totalMs / 1000);
		const h = Math.floor(totalSecs / 3600);
		const m = Math.floor((totalSecs % 3600) / 60);
		const s = totalSecs % 60;

		return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
	}

	function isQueueDrag(): boolean {
		return dragState.payload?.sourceQueueIndex !== undefined && !isNowPlayingDrag();
	}

	function isNowPlayingDrag(): boolean {
		return (
			dragState.payload !== null &&
			dragState.payload.sourceQueueIndex === undefined &&
			dragState.payload.sourcePlaylistUid === null &&
			dragState.payload.uids.length === 1 &&
			player.track?.uid === dragState.payload.uids[0]
		);
	}

	function handleRowDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
		dragOverAppend = false;
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		dragOverIndex = index;
		dragOverPosition = e.clientY < rect.top + rect.height / 2 ? "above" : "below";
	}

	function handleRowDragLeave(e: DragEvent) {
		const related = e.relatedTarget as HTMLElement | null;
		if (related && (e.currentTarget as HTMLElement).contains(related)) return;
		dragOverIndex = null;
	}

	function handleScrollAreaDragOver(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
		if (dragOverIndex === null) {
			dragOverAppend = true;
		}
	}

	function handleScrollAreaDragLeave(e: DragEvent) {
		const related = e.relatedTarget as HTMLElement | null;
		if (related && (e.currentTarget as HTMLElement).contains(related)) return;
		dragOverAppend = false;
	}

	function handleRowDrop(e: DragEvent, dropIndex: number) {
		e.preventDefault();
		e.stopPropagation();

		const toDisplayIndex = dragOverPosition === "above" ? dropIndex : dropIndex + 1;

		if (isQueueDrag()) {
			const fromDisplayIndex = dragState.payload!.sourceQueueIndex!;
			reorderQueue(fromDisplayIndex, toDisplayIndex);
		} else {
			const uids = getDropUids(e);
			const tracks = resolveUidsToTracks(uids);
			if (tracks.length > 0) {
				insertIntoQueue(tracks, toDisplayIndex - 1);
			}
		}

		dragOverIndex = null;
		dragOverAppend = false;
		endDrag();
	}

	function handleScrollAreaDrop(e: DragEvent) {
		e.preventDefault();

		if (dragOverIndex !== null) return;

		if (isQueueDrag()) {
			const fromDisplayIndex = dragState.payload!.sourceQueueIndex!;
			reorderQueue(fromDisplayIndex, getQueuedTracks().length - 1);
		} else {
			const uids = getDropUids(e);
			const tracks = resolveUidsToTracks(uids);
			if (tracks.length > 0) {
				insertIntoQueue(tracks);
			}
		}

		dragOverAppend = false;
		endDrag();
	}

	function getDropUids(e: DragEvent): string[] {
		const raw = e.dataTransfer?.getData("text/plain") ?? "";
		return raw.split(",").map(u => u.trim()).filter(Boolean);
	}

	function resolveUidsToTracks(uids: string[]) {
		return uids
			.map(uid => library.tracks.find(t => t.uid === uid))
			.filter((t): t is NonNullable<typeof t> => t !== undefined);
	}
</script>

<Tabs.Content value="queue" class="p-1 space-y-2">
	<p class="text-sm text-foreground">Now Playing</p>
	<div
		class="rounded-md border-2"
		draggable="false"
	>
		{#if player.track}
			<QueueTrackItem track={player.track} isNowPlaying={true} />
		{/if}
	</div>

	<p class="text-sm text-foreground">Next up:</p>

	<div class="flex justify-between">
		<p class="text-xs text-muted-foreground">{formatTotalRemainingTime()} Remaining</p>
		<button class="text-xs text-muted-foreground cursor-pointer hover:underline" onclick={clearQueue}>Clear</button>
	</div>

	<ScrollArea
		class="min-h-0 min-w-0 h-[254px]"
		ondragover={handleScrollAreaDragOver}
		ondragleave={handleScrollAreaDragLeave}
		ondrop={handleScrollAreaDrop}
	>
		<div class="flex flex-col gap-0.5 pb-2">
			{#each upcomingTracks as track, i (track.uid)}
				<div
					class="relative"
					role="row"
					tabindex={i}
					ondragover={(e) => handleRowDragOver(e, i)}
					ondragleave={handleRowDragLeave}
					ondrop={(e) => handleRowDrop(e, i)}
				>
					{#if dragOverIndex === i && dragOverPosition === "above"}
						<div class="absolute top-0 left-0 right-0 h-0.5 bg-primary z-10 pointer-events-none"></div>
					{/if}

					<QueueTrackItem
						{track}
						displayIndex={i}
						isNowPlaying={false}
					/>

					{#if dragOverIndex === i && dragOverPosition === "below"}
						<div class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary z-10 pointer-events-none"></div>
					{/if}
				</div>
			{/each}

			{#if dragOverAppend}
				<div class="h-0.5 bg-primary mx-2 rounded pointer-events-none"></div>
			{/if}
		</div>
	</ScrollArea>
</Tabs.Content>