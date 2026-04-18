<script lang="ts">

	// COMPONENTS
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

	import TrackPlaylistContent from "$lib/components/app-ui/context-menus/AddTrackToPlaylist.svelte";

	// CUSTOM COMPONENTS
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";

	// SCRIPTS
   import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { addTrackToQueue, removeFromQueue } from "$lib/ts/audio/audioManager.svelte";
   import { openEditModal } from "$lib/ts/app/editModal.svelte";
   import { copySelectedNameToClipboard } from "$lib/ts/app/trackSelection.svelte";

   import type { Track } from "$lib/ts/util/types";

	// PROPS
	let { track, inQueue = false, displayIndex = null } = $props<{ track: Track; inQueue?: boolean; displayIndex?: number | null }>();

	let content = $state<ReturnType<typeof TrackPlaylistContent> | null>(null);

</script>

<ContextMenu.Content>
	<ContextMenu.Group>
		<ContextMenu.Item onSelect={() => copySelectedNameToClipboard(track)}>Copy Track & Artist Name</ContextMenu.Item>
		<ContextMenu.Item
			onSelect={() => addTrackToQueue(track)}
			class={track.path.match(/^[a-z]+-[0-9a-f-]{36}$/) ? 'opacity-50 pointer-events-none' : ''}>
			Add to Queue
		</ContextMenu.Item>
		{#if inQueue}
			<ContextMenu.Item onSelect={() => removeFromQueue(displayIndex)}>Remove from Queue</ContextMenu.Item>
		{/if}
	</ContextMenu.Group>
	<ContextMenu.Separator />
	<ContextMenu.Group>
		<DropdownMenu.Sub>
			<DropdownMenu.SubTrigger>Add to playlist</DropdownMenu.SubTrigger>
			<DropdownMenu.SubContent class="w-64">
				<TrackPlaylistContent bind:this={content} {track} />
			</DropdownMenu.SubContent>
		</DropdownMenu.Sub>
	</ContextMenu.Group>
	<ContextMenu.Separator />
	<ContextMenu.Group>
		<ContextMenu.Item onSelect={() => setSelection(track.uid, "track")}>Go to Track</ContextMenu.Item>
		<ContextMenu.Item onSelect={() => openEditModal({ type: "track", uid: track!.uid })}>Edit Metadata</ContextMenu.Item>
	</ContextMenu.Group>
</ContextMenu.Content>