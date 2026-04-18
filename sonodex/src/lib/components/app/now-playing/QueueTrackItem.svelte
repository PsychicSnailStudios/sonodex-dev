<script lang="ts">
	// COMPONENTS
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import TrackContext from "$lib/components/app-ui/context-menus/TrackContext.svelte";

	// SCRIPTS
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { startDrag, endDrag } from "$lib/ts/app-states/state_drag.svelte";
	import ArtistsList from "$lib/components/app-ui/ArtistsList.svelte";
	import type { Track } from "$lib/ts/util/types";

	// PROPS
	let {
		track,
		displayIndex = null,
		isNowPlaying = false,
		onDragStart = null,
		onDragEnd = null,
	} = $props<{
		track: Track;
		displayIndex?: number | null;
		isNowPlaying?: boolean;
		onDragStart?: ((e: DragEvent) => void) | null;
		onDragEnd?: (() => void) | null;
	}>();

	// FUNCTIONS
	function handleDragStart(e: DragEvent) {
		if (!e.dataTransfer) return;
		e.dataTransfer.effectAllowed = "move";
		e.dataTransfer.setData("text/plain", track.uid);

		if (isNowPlaying) {
			startDrag({ type: "tracks", uids: [track.uid], sourcePlaylistUid: null });
		} else {
			startDrag({
				type: "tracks",
				uids: [track.uid],
				sourcePlaylistUid: null,
				sourceQueueIndex: displayIndex ?? undefined,
			});
		}

		onDragStart?.(e);
	}

	function handleDragEnd() {
		endDrag();
		onDragEnd?.();
	}
</script>

<ContextMenu.Root>
	<ContextMenu.Trigger>
		<div
			class="flex gap-2 p-2 items-center cursor-grab active:cursor-grabbing rounded-md hover:bg-muted/50 transition-colors"
			draggable="true"
			ondragstart={handleDragStart}
			ondragend={handleDragEnd}
			role="row"
			tabindex="0"
		>
			<ArtworkDisplay uid={track.uid} size={36} type="track" />
			<div class="min-w-0 grid flex-1">
				<span
					role="button"
					tabindex="0"
					onclick={() => setSelection(track.uid, "track")}
					onkeydown={(e) => { if (e.key === 'Enter') setSelection(track.uid, "track"); }}
					class="text-sm truncate cursor-pointer hover:underline"
				>
					{track.title}
				</span>
				<div class="text-xs text-muted-foreground truncate">
					<ArtistsList artists={track.artists} />
				</div>
			</div>
		</div>
	</ContextMenu.Trigger>
	<TrackContext track={track} />
</ContextMenu.Root>