<script lang="ts">

	// COMPONENTS
   import { Ellipsis } from "lucide-svelte";

	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import Button from "$lib/components/ui/button/button.svelte";

	// CUSTOM COMPONENTS
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";

	// SCRIPTS
   import { addTrackToQueue } from "$lib/ts/audio/audioManager.svelte";
   import { copySelectedNameToClipboard } from "$lib/ts/app/trackSelection.svelte";
   import type { Track } from "$lib/ts/util/types";
	
	// PROPS
	let { track } = $props<{ track: Track; }>()
	
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon-sm">
				<Ellipsis class="w-4 h-4"/>
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>

	<DropdownMenu.Content>

		<DropdownMenu.Group>
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