<script lang="ts">
	import ArtworkDisplay from "./ArtworkDisplay.svelte";
	import { setSelection } from "$lib/ts/session.svelte";
	import { playTrackByUid, queueTracksFromUid } from "$lib/ts/audio/audioManager.svelte";
	import { Play } from "lucide-svelte";
	import Button from "../ui/button/button.svelte";
	import type { Playlist } from "$lib/ts/util/types";

	let {
		playlist,
		indent = 0,
		ondragover,
		ondragleave,
		ondrop,
		highlighted = false,
	} = $props<{
		playlist: Playlist;
		indent?: number;
		ondragover?: (e: DragEvent) => void;
		ondragleave?: (e: DragEvent) => void;
		ondrop?: (e: DragEvent) => void;
		highlighted?: boolean;
	}>();

	function play(e: MouseEvent) {
		e.stopPropagation();
		queueTracksFromUid(playlist.uid, true);
	}
</script>

<div
	class="flex items-center gap-2 py-1.5 pr-2 rounded-md hover:bg-muted/50 group cursor-default"
	class:ring-1={highlighted}
	class:ring-primary={highlighted}
	style="padding-left: {indent}px"
	role="region"
	aria-label="Playlist"
	{ondragover}
	{ondragleave}
	{ondrop}
>
	<button
		class="flex items-center gap-2 flex-1 min-w-0 text-left"
		onclick={() => setSelection(playlist.uid, "playlist")}
	>
		<div class="size-8 rounded shrink-0 overflow-hidden bg-muted">
			<ArtworkDisplay uid={playlist.uid} type="playlist" />
		</div>
		<div class="flex flex-col min-w-0 flex-1">
			<span class="text-sm truncate leading-tight">{playlist.title}</span>
			{#if (playlist as any).owner}
				<span class="text-xs text-muted-foreground truncate leading-tight">{(playlist as any).owner}</span>
			{/if}
		</div>
	</button>

	<Button
		variant="ghost"
		size="icon"
		class="size-6 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
		onclick={play}
	>
		<Play class="size-3" />
	</Button>
</div>