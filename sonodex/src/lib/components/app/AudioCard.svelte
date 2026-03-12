<script lang="ts">
	import TrackArtwork from "./TrackArtwork.svelte";

	let { data } = $props<{ album: { title: string; subTitle: string | null; artworkId: number } }>();

</script>

<div class="flex flex-col gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default">
	
	<TrackArtwork id={data.artworkId} />

	<div class="min-w-0">
		<p class="text-sm font-medium truncate">{data.title}</p>
		<p class="text-xs text-muted-foreground truncate">{data.subTitle}</p>
	</div>

</div>