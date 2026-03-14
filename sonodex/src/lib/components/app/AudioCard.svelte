<script lang="ts">
	import ArtworkDisplay from "./ArtworkDisplay.svelte";
	import { setSelection } from "$lib/session.svelte";

	let { title, subTitle, artworkId } = $props<{ title: string; subTitle: string | null; artworkId: number }>();

</script>

<button class="flex flex-col gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default justify-start"
	  onclick={() => setSelection(artworkId, "album")}>
	
	<ArtworkDisplay id={artworkId} size={150} type="album" />

	<div class="min-w-0">
		<p class="text-sm font-medium truncate">{title}</p>
		<p class="text-xs text-muted-foreground truncate">{subTitle}</p>
	</div>

</button>