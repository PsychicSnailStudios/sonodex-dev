<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Slider } from "$lib/components/ui/slider/index.js";

	import { player, togglePlay, seek, skipBack } from "$lib/player.svelte";

	import { Play, Pause, SkipBack, SkipForward, Shuffle, Repeat2 } from "lucide-svelte";

	function formatTime(seconds: number): string {
		if (!seconds || isNaN(seconds)) return "0:00";
		const m = Math.floor(seconds / 60);
		const s = Math.floor(seconds % 60);
		return `${m}:${s.toString().padStart(2, "0")}`;
	}

	let sliderValue = $derived([player.duration > 0 ? (player.currentTime / player.duration) * 100 : 0]);

	function onSeek(val: number[]) {
		if (player.duration) seek((val[0] / 100) * player.duration);
	}
</script>

<div class="app-playbar bg-muted grid p-2 gap-1 rounded-md">

	<div class="flex items-center gap-2">
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
		<Button variant="ghost" size="icon">
			<SkipForward />
		</Button>
		<Button variant="ghost" size="icon">
			<Repeat2 />
		</Button>
	</div>

	<div class="app-bar grid gap-2 items-center">
		<span class="text-xs text-muted-foreground">{formatTime(player.currentTime)}</span>
		<Slider type="single" value={sliderValue} max={100} step={0.1} onValueChange={onSeek} />
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
</style>