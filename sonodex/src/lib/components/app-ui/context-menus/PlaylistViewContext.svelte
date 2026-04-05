<script lang="ts">

	// COMPONENTS
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";

	// CUSTOM COMPONENTS
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";

	// SCRIPTS
   import { setSelection } from "$lib/ts/app-states/state_session.svelte";
   import { addTrackToQueue } from "$lib/ts/audio/audioManager.svelte";
   import { openEditModal } from "$lib/ts/app/editModal.svelte";
   import { copySelectedNameToClipboard } from "$lib/ts/app/trackSelection.svelte";

	// PROPS
	let {
		path,
		onCreatePlaylist,
		onCreateFolder,
	} = $props<{
		path: string | null;
		onCreatePlaylist: (folder: string) => void;
		onCreateFolder: (parent: string) => void;
	}>();

</script>

<ContextMenu.Content>
	<ContextMenu.Group>
		<ContextMenu.Item onSelect={() => onCreatePlaylist(path)}>New Playlist</ContextMenu.Item>
		<ContextMenu.Item onSelect={() => onCreateFolder(path)}>New Folder</ContextMenu.Item>		
	</ContextMenu.Group>
</ContextMenu.Content>