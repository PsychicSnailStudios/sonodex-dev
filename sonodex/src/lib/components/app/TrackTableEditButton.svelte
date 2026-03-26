<script lang="ts">
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import Button from "../ui/button/button.svelte";
	import ScrollArea from "../ui/scroll-area/scroll-area.svelte";

	import { BadgePlus, CirclePlus, CircleCheck } from "lucide-svelte";

   import type { Track } from "$lib/ts/util/types";
   import { addTrackToQueue } from "$lib/ts/audio/audioManager.svelte";
	import { library } from "$lib/library.svelte";
	import { removeTrackFromPlaylist } from "$lib/playlistManager.svelte";
   import TrackPlaylistEditButton from "./TrackPlaylistEditButton.svelte";

	let { track } = $props<{ track: Track; }>()
	
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon">...</Button>
		{/snippet}
	</DropdownMenu.Trigger>

	<DropdownMenu.Content>
		<DropdownMenu.Group>
			<!-- <div>
				<ScrollArea class="h-[250px]">
					<div>
						
					</div>
				</ScrollArea>
			</div> -->
			<DropdownMenu.Item onSelect={() => addTrackToQueue(track)}>Add to Queue</DropdownMenu.Item>
			<!-- <DropdownMenu.Item onSelect={() => removeTrackFromPlaylist(track)}>Remove from Playlist</DropdownMenu.Item> -->
			<TrackPlaylistEditButton track={track} isButton={false} />
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>