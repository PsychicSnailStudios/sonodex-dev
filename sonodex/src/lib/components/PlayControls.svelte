<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Slider } from "$lib/components/ui/slider/index.js";
	import { CirclePlay, CirclePause, SkipBack, SkipForward, Shuffle, Repeat, Repeat1, Volume, Volume2, VolumeX, Volume1, VolumeOff } from "lucide-svelte";
	import { player, togglePlay, seek, skipBack, skipNext, toggleLoop, toggleShuffle, setVolume, toggleMute } from "$lib/audioManager.svelte";


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

	<div class="app-playbar-buttons-wrapper grid">
		<span></span>
		<div class="flex justify-center items-center gap-2">
			<Button variant="ghost" size="icon" onclick={toggleShuffle}>
				{#if player.shuffleType === 0}
					<Shuffle class="text-muted-foreground" />
				{:else if player.shuffleType === 1}
					<Shuffle />
				{:else}
					<Shuffle />
				{/if}
			</Button>
			<Button variant="ghost" size="icon" onclick={skipBack}>
				<SkipBack />
			</Button>
			<Button variant="ghost" size="lg" onclick={togglePlay}>
				{#if player.isPlaying}
					<CirclePause size={32} />
				{:else}
					<CirclePlay size={32} />
				{/if}
			</Button>
			<Button variant="ghost" size="icon" onclick={skipNext}>
				<SkipForward />
			</Button>
			<Button variant="ghost" size="icon" onclick={toggleLoop}>
				{#if player.loopType === 0}
					<Repeat class="text-muted-foreground" />
				{:else if player.loopType === 1}
					<Repeat1 />
				{:else}
					<Repeat />
				{/if}
			</Button>
		</div>

		<div class="flex justify-center items-center">
			<Button variant="ghost" size="icon" onclick={toggleMute}>
				{#if player.volume === 0 || player.muted}
					<VolumeOff />
				{:else if player.volume < 0.5}
					<Volume1 />
				{:else if player.volume < 0.75}
					<Volume2 />
				{:else}
					<Volume />
				{/if}
			</Button>
			<Slider
				type="single"
				value={player.volume}
				min={0}
				max={1}
				step={0.01}
				onValueChange={(v) => setVolume(v)}
				onValueCommit={(v) => setVolume(v)}
				class="w-[100px] app-tracking-slider"
			/>
		</div>
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

.app-playbar-buttons-wrapper {
	grid-template-columns: 120px 1fr 120px;
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