<script lang="ts">

	// COMPONENTS
	import Button from "$lib/components/ui/button/button.svelte";
	
	// CUSTOM COMPONENTS
	import { Play } from "lucide-svelte";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";

	// SCRIPTS
	import { setSelection } from "$lib/ts/session.svelte";
	import { playTrackByUid, queueTracksByObject } from "$lib/ts/audio/audioManager.svelte";
   import { getAlbumTracks, getPlaylistTracks } from "$lib/ts/library.svelte";
	import type { AudioCatagories } from "$lib/ts/util/types";
	
	// PROPS
	let { title, subTitle, artworkUid, type } = $props<{ title: string; subTitle: string | null; artworkUid: string; type: AudioCatagories }>();
	
	// FUNCTIONS
	function play(e: MouseEvent) {
		e.stopPropagation();
		if (type === "track") {
			playTrackByUid(artworkUid);
		} else if (type === "playlist") {
			queueTracksByObject(getPlaylistTracks(artworkUid), true);
		}
		else {
			queueTracksByObject(getAlbumTracks(artworkUid), true);
		}
	}
</script>

<button
	class="w-full text-left flex flex-col gap-2 p-2 rounded-md bg-background border hover:border-primary transition-colors cursor-default justify-start group"
	onclick={() => setSelection(artworkUid, type)}
>
	<div class="relative w-full">
		<ArtworkDisplay uid={artworkUid} type={type} />
		<div class="absolute inset-0 flex items-end justify-end p-1 opacity-0 group-hover:opacity-100 transition-opacity">
			<Button variant="secondary" size="icon" onclick={play}>
				<Play />
			</Button>
		</div>
	</div>

	<div class="min-w-0 grid">
		<p class="text-sm font-medium truncate">{title}</p>
		<p class="text-xs text-muted-foreground truncate">{subTitle}</p>
	</div>
</button>