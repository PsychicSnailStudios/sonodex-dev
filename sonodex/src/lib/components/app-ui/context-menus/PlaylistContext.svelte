<script lang="ts">
	import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
	import { movePlaylists } from "$lib/ts/drag-n-drop/dragdrop_playlists";

	let { folderPaths, uid } = $props<{ folderPaths: string[]; uid: string }>();
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
</ContextMenu.Content>