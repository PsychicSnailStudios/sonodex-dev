<script lang="ts">
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import Button from "../../ui/button/button.svelte";
	import ScrollArea from "../../ui/scroll-area/scroll-area.svelte";
   import type { Track } from "$lib/ts/util/types";
   import { addTrackToQueue } from "$lib/ts/audio/audioManager.svelte";
   import TrackPlaylistEditButton from "../TrackPlaylistEditButton.svelte";
   import { copySelectedNameToClipboard } from "$lib/ts/app/trackSelection.svelte";

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
			<DropdownMenu.Item onSelect={() => copySelectedNameToClipboard(track)}>Copy Track & Artist Name</DropdownMenu.Item>
			<DropdownMenu.Item onSelect={() => addTrackToQueue(track)}>Add to Queue</DropdownMenu.Item>
		</DropdownMenu.Group>
		<DropdownMenu.Separator />
		<DropdownMenu.Group>
			<DropdownMenu.Item>
				<TrackPlaylistEditButton track={track} isButton={false} />
			</DropdownMenu.Item>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>