<script lang="ts">
	import ArtworkDisplay from "./ArtworkDisplay.svelte";
	import { setSelection } from "$lib/ts/session.svelte";
	import Button from "../ui/button/button.svelte";
	import type { AudioCatagories } from "$lib/ts/util/types";

	import { playTrackByUid, queueTracksFromUid } from "$lib/ts/audio/audioManager.svelte";

	import { Play } from "lucide-svelte";

	let { title, subTitle, artworkUid, type } = $props<{ title: string; subTitle: string | null; artworkUid: string; type: AudioCatagories }>();

	function play(e: MouseEvent) {
		e.stopPropagation();
		if (type === "track") {
			playTrackByUid(artworkUid);
		} else {
			queueTracksFromUid(artworkUid, true);
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