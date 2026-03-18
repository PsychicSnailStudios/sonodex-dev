<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Slider } from "$lib/components/ui/slider/index.js";
	import { Play, Pause, SkipBack, SkipForward, Shuffle, Repeat2 } from "lucide-svelte";
	import { player, togglePlay, seek, skipBack, skipNext } from "$lib/audioManager.svelte";


	function formatTime(seconds: number): string {
		if (!seconds || isNaN(seconds)) return "0:00";
		const m = Math.floor(seconds / 60);
		const s = Math.floor(seconds % 60);
		return `${m}:${s.toString().padStart(2, "0")}`;
	}

	let seeking = $state(false);
	let seekValue = $state(0);
 
	$effect(() => {
		if (!seeking) {
			seekValue = player.currentTime;
		}
	});
 
	function onSliderChange(value: number[]) {
		seekValue = value[0];
	}
 
	function onSliderCommit(value: number[]) {
		seek(value[0]);
		seeking = false;
	}
 
	function onSliderStart() {
		seeking = true;
	}

</script>

<div class="app-playbar bg-muted grid p-2 gap-1 rounded-md">

	<div class="flex justify-center items-center gap-2">
		<Button variant="ghost" size="icon">
			<Shuffle />
		</Button>
		<Button variant="ghost" size="icon" onclick={skipBack}>
			<SkipBack />
		</Button>
		<Button variant="ghost" size="icon" onclick={togglePlay}>
			{#if player.isPlaying}
				<Pause />
			{:else}
				<Play />
			{/if}
		</Button>
		<Button variant="ghost" size="icon" onclick={skipNext}>
			<SkipForward />
		</Button>
		<Button variant="ghost" size="icon">
			<Repeat2 />
		</Button>
	</div>

	<div class="app-bar grid gap-2 items-center">
		<span class="text-xs text-muted-foreground">{formatTime(player.currentTime)}</span>
		
		<div class="flex-1" onpointerdown={onSliderStart} aria-hidden="true" tabindex="-1">
			<Slider
				type="single"
				value={seeking ? seekValue : player.currentTime}
				min={0}
				max={player.duration || 1}
				step={0.1}
				disabled={!player.track}
				onValueChange={(v) => onSliderChange([v])}
				onValueCommit={(v) => onSliderCommit([v])}
				class="w-full app-tracking-slider"
			/>
		</div>

		<span class="text-xs text-muted-foreground">{formatTime(player.duration)}</span>
	</div>

</div>

<style>
.app-playbar {
	max-height: 85px;
	height: 85px;
}

.app-bar {
	grid-template-columns: auto 1fr auto;
}

:global(.app-tracking-slider span[data-slider-track]) {
	background-color: var(--foreground) !important;
}

:global([data-slider-thumb]) {
	opacity: 0;
}

:global([data-slider-thumb]:hover) {
	opacity: 1;
}
</style>