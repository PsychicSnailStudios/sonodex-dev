<script lang="ts">
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import { movePlaylists } from "$lib/ts/drag-n-drop/dragdrop_playlists";
	import { invoke } from "@tauri-apps/api/core";
	import { library } from "$lib/ts/library.svelte";

	let { folderPaths, uid } = $props<{ folderPaths: string[]; uid: string }>();

	async function deletePlaylist() {
		await invoke("delete_playlist_entry", { uid });
		library.playlists = await invoke("get_playlists");
	}
</script>

<ContextMenu.Content>
	<ContextMenu.Sub>
		<ContextMenu.SubTrigger>Move to folder</ContextMenu.SubTrigger>
		<ContextMenu.SubContent>
			<ContextMenu.Item onclick={() => movePlaylists([uid], null)}>
				Home (no folder)
			</ContextMenu.Item>
			{#if folderPaths.length > 0}
				<ContextMenu.Separator />
				{#each folderPaths as fp}
					<ContextMenu.Item onclick={() => movePlaylists([uid], fp)}>
						{fp}
					</ContextMenu.Item>
				{/each}
			{/if}
		</ContextMenu.SubContent>
	</ContextMenu.Sub>
	<ContextMenu.Separator />
	<ContextMenu.Item class="text-destructive" onclick={deletePlaylist}>
		Delete
	</ContextMenu.Item>
</ContextMenu.Content>