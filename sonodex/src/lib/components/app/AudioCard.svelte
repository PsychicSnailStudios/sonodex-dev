<script lang="ts">
	import TrackArtwork from "./TrackArtwork.svelte";

	let { title, subTitle, artworkId } = $props<{ album: { title: string; subTitle: string | null; artworkId: number } }>();

</script>

<div class="flex flex-col gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
	
	<TrackArtwork id={artworkId} width={100} height={100} />

	<div class="min-w-0">
		<p class="text-sm font-medium truncate">{title}</p>
		<p class="text-xs text-muted-foreground truncate">{subTitle}</p>
	</div>

</div>