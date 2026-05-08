<script lang="ts">
	import { Folder } from "lucide-svelte";
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import DefultPlaylistArt from "$lib/components/app-ui/playlist/DefultPlaylistArt.svelte";

	import { getPlaylistTracks } from "$ts/store/library.svelte";

	let { folderPath, artUids } = $props<{ folderPath: string; artUids: string[] }>();
</script>

<div class="flex flex-col gap-1 p-2">
	<div class="w-full aspect-square rounded-md overflow-hidden">
		{#if artUids.length >= 4}
			<div class="grid grid-cols-2 w-full h-full gap-2 p-2">
				{#each artUids.slice(0, 4) as uid}
					<ArtworkDisplay {uid} type="playlist">
						<DefultPlaylistArt tracks={getPlaylistTracks(uid)} />
					</ArtworkDisplay>
				{/each}
			</div>
		{:else if artUids.length > 0}
			<div class="grid grid-cols-2 w-full h-full gap-2 p-2">
				{#each artUids as uid}
					<ArtworkDisplay {uid} type="playlist">
						<DefultPlaylistArt tracks={getPlaylistTracks(uid)} />
					</ArtworkDisplay>
				{/each}
			</div>
		{:else}
			<div class="w-full h-full flex items-center justify-center">
				<Folder class="size-12 text-muted-foreground/40" />
			</div>
		{/if}
	</div>
	<div class="px-1">
		<p class="text-sm font-medium truncate">{folderPath}</p>
		<p class="text-xs text-muted-foreground">Folder</p>
	</div>
</div>