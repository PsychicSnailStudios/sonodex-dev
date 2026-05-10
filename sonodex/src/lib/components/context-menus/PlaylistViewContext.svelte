<script lang="ts">

	// COMPONENTS
	import * as ContextMenu from "$shadcn/context-menu/index.js";

	// CUSTOM COMPONENTS
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";

	// SCRIPTS
   import { setSelection } from "$ts/store/session.svelte";
   import { addTrackToQueue } from "$ts/audio/audioManager.svelte";
   import { openEditModal } from "$ts/ui/editModal.svelte";
   import { copySelectedNameToClipboard } from "$ts/store/trackSelection.svelte";

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