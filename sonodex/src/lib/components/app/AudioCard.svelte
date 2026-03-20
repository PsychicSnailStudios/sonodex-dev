<script lang="ts">
	import ArtworkDisplay from "./ArtworkDisplay.svelte";
	import { setSelection } from "$lib/session.svelte";
   import Button from "../ui/button/button.svelte";
	import type { AudioCatagories } from "$lib/types";

	import { playTrackById, QueueTracksById, queueTracksFromId } from "$lib/audioManager.svelte";
	import { library } from "$lib/library.svelte";

	import { Play } from "lucide-svelte";

	let { title, subTitle, artworkId, type } = $props<{ title: string; subTitle: string | null; artworkId: number, type: AudioCatagories }>();

	function play() {
		if (type === "track") {
			playTrackById(artworkId);
		}
		else {
			queueTracksFromId(artworkId, type, true);
		}
	}
</script>

<button class="app-audio-card text-left flex flex-col gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default justify-start"
	  onclick={() => setSelection(artworkId, type)}>
	
	<ArtworkDisplay id={artworkId} type={type} />

	<div class="min-w-0 grid">
		<p class="text-sm font-medium truncate">{title}</p>
		<p class="text-xs text-muted-foreground truncate">{subTitle}</p>
	</div>

	<Button class="app-hidden-button" variant="outline" onclick={() => play()}>
		<Play />
	</Button>

</button>

<style>
.app-audio-card:hover button {
		display: block;
}

.app-hidden-button {
	display: none;
}
</style>