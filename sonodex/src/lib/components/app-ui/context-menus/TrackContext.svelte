<script lang="ts">

	// COMPONENTS
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";

	// CUSTOM COMPONENTS
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";

	// SCRIPTS
   import { setSelection } from "$lib/ts/session.svelte";
   import { addTrackToQueue } from "$lib/ts/audio/audioManager.svelte";
   import { openEditModal } from "$lib/ts/app/editModal.svelte";
   import { copySelectedNameToClipboard } from "$lib/ts/app/trackSelection.svelte";

   import type { Track } from "$lib/ts/util/types";

	// PROPS
	let { track } = $props<{ track: Track }>();

</script>

<ContextMenu.Content>
	<ContextMenu.Group>
		<ContextMenu.Item onSelect={() => copySelectedNameToClipboard(track)}>Copy Track & Artist Name</ContextMenu.Item>
		<ContextMenu.Item
			onSelect={() => addTrackToQueue(track)}
			class={track.path.match(/^[a-z]+-[0-9a-f-]{36}$/) ? 'opacity-50 pointer-events-none' : ''}>
			Add to Queue
		</ContextMenu.Item>
	</ContextMenu.Group>
	<ContextMenu.Separator />
	<ContextMenu.Group>
		<ContextMenu.Item>
			<TrackPlaylistEditButton track={track} isButton={false} />
		</ContextMenu.Item>
	</ContextMenu.Group>
	<ContextMenu.Separator />
	<ContextMenu.Group>
		<ContextMenu.Item onSelect={() => setSelection(track.uid, "track")}>Go to Track</ContextMenu.Item>
		<ContextMenu.Item onSelect={() => openEditModal({ type: "track", uid: track!.uid })}>Edit Metadata</ContextMenu.Item>
	</ContextMenu.Group>
</ContextMenu.Content>