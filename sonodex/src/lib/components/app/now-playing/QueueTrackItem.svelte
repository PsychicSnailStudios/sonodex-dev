<script lang="ts">

	// COMPONENTS
   import * as ContextMenu from "$lib/components/ui/context-menu/index.js";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
   import TrackContext from "$lib/components/app-ui/context-menus/TrackContext.svelte";

	// SCRIPTS
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { getArtistUidFromName } from "$lib/ts/library.svelte";
	import { parseArtists } from "$lib/ts/util/helpers";
	import type { Track } from "$lib/ts/util/types";
   import ArtistsList from "$lib/components/app-ui/ArtistsList.svelte";

	// PROPS
	let { track } = $props<{ track: Track }>();

</script>

<ContextMenu.Root>
	<ContextMenu.Trigger>
		<div class="flex gap-2 p-2">
			<ArtworkDisplay uid={track.uid} size={30} type="track" />
			<div class="min-w-0 grid">
				<span role="button" tabindex="0" onclick={() => setSelection(track.uid, "track")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(track.uid, "track"); }} class="text-sm truncate cursor-pointer hover:underline">
					{track.title}
				</span>
				<div class="text-xs text-muted-foreground truncate cursor-pointer hover:underline">
					<ArtistsList artists={track.artists} />
				</div>
			</div>
		</div>
	</ContextMenu.Trigger>
	<TrackContext track={track} />
</ContextMenu.Root>