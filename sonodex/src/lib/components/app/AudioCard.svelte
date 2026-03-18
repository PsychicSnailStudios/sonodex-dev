<script lang="ts">
	import ArtworkDisplay from "./ArtworkDisplay.svelte";
	import { setSelection } from "$lib/session.svelte";
   import Button from "../ui/button/button.svelte";
	import type { AudioCatagories } from "$lib/types";

	import { playTrackById } from "$lib/audioManager.svelte";

	import { Play } from "lucide-svelte";

	let { title, subTitle, artworkId, type } = $props<{ title: string; subTitle: string | null; artworkId: number, type: AudioCatagories }>();

</script>

<button class="app-audio-card text-left flex flex-col gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default justify-start"
	  onclick={() => setSelection(artworkId, type)}>
	
	<ArtworkDisplay id={artworkId} size={150} type={type} />

	<div class="min-w-0 grid">
		<p class="text-sm font-medium truncate">{title}</p>
		<p class="text-xs text-muted-foreground truncate">{subTitle}</p>
	</div>

	<Button class="app-hidden-button" variant="outline" onclick={() => playTrackById(artworkId)}>
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