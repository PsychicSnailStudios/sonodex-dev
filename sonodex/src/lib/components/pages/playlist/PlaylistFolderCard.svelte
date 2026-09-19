<script lang="ts">
	import { Folder } from "lucide-svelte";
	import ArtworkDisplay from "$lib/components/custom/ArtworkDisplay.svelte";
	import DefultPlaylistArt from "$lib/components/pages/playlist/DefultPlaylistArt.svelte";
	import { getTrackArrayFromUID } from "$ts/store/library.svelte";
	import type { Track } from "$ts/util/types";

	let { folderPath, artUids } = $props<{ folderPath: string; artUids: string[] }>();

	// Each uid needs its own resolved tracks for the DefaultPlaylistArt fallback.
	// We load them async and store in a map keyed by uid.
	let tracksByUid = $state<Map<string, Track[]>>(new Map());

	$effect(() => {
		const uidsToLoad = artUids.slice(0, 4);
		for (const uid of uidsToLoad) {
			if (!tracksByUid.has(uid)) {
				getTrackArrayFromUID(uid).then(tracks => {
					tracksByUid = new Map(tracksByUid).set(uid, tracks);
				});
			}
		}
	});
</script>

<div class="flex flex-col gap-1 p-2">
	<div class="w-full aspect-square rounded-md overflow-hidden">
		{#if artUids.length >= 4}
			<div class="grid grid-cols-2 w-full h-full gap-2 p-2">
				{#each artUids.slice(0, 4) as uid}
					<ArtworkDisplay {uid}>
						<DefultPlaylistArt tracks={tracksByUid.get(uid) ?? []} />
					</ArtworkDisplay>
				{/each}
			</div>
		{:else if artUids.length > 0}
			<div class="grid grid-cols-2 w-full h-full gap-2 p-2">
				{#each artUids as uid}
					<ArtworkDisplay {uid}>
						<DefultPlaylistArt tracks={tracksByUid.get(uid) ?? []} />
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