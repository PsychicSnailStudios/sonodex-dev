<script lang="ts">
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import { movePlaylists } from "$lib/ts/app/dragState.svelte";

	let { path, onCreatePlaylist, onCreateFolder } = $props<{
		path: string;
		onCreatePlaylist: (folder: string) => void;
		onCreateFolder: (parent: string) => void;
	}>();
</script>

<ContextMenu.Content>
	<ContextMenu.Item onclick={() => onCreatePlaylist(path)}>
		New playlist inside
	</ContextMenu.Item>
	<ContextMenu.Item onclick={() => onCreateFolder(path)}>
		New folder inside
	</ContextMenu.Item>
</ContextMenu.Content>