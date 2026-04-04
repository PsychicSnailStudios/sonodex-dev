<script lang="ts">

	// COMPONENTS
	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
   
	// CUSTOM COMPONENTS
	import QueueTrackItem from "$lib/components/app/now-playing/QueueTrackItem.svelte";

	// SCRIPTS
	import { clearQueue, getQueuedTracks, player } from "$lib/ts/audio/audioManager.svelte";

	// VARIABLES
	let upcomingTracks = $derived(getQueuedTracks());

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
</script>

<Tabs.Content value="queue" class="p-1 space-y-2">
	<p class="text-sm text-foreground">Now Playing</p>
	<div class="rounded-md border-2">
		<QueueTrackItem track={player.track!} />
	</div>
	
	<p class="text-sm text-foreground">Next up: </p>
	
	<div class="flex justify-between">
		<p class="text-xs text-muted-foreground">{formatTotalRemainingTime()} Remaining</p>
		<button class="text-xs text-muted-foreground cursor-pointer hover:underline" onclick={clearQueue}>Clear</button>
	</div>

	<ScrollArea class="min-h-0 min-w-0 h-[254px]">
		<div class="flex flex-col gap-0.5">
			{#each upcomingTracks as track}
				<QueueTrackItem track={track} />
			{/each}
		</div>
	</ScrollArea>

</Tabs.Content>
