<script lang="ts">

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Button } from "$lib/components/ui/button";

	import { Rows4, BadgePlus } from "lucide-svelte";

	import { playing } from "$lib/session.svelte";

	import ArtworkDisplay from "$lib/components/app/ArtworkDisplay.svelte";
</script>

<div class="flex flex-col gap-1">
	<div class="app-queue bg-muted p-2 rounded-md h-[300px]">
		<Tabs.Root value="queue">
			<Tabs.List>
				<Tabs.Trigger value="queue">Queue</Tabs.Trigger>
				<Tabs.Trigger value="recent">Recently Played</Tabs.Trigger>
				<Tabs.Trigger value="lyrics">Lyrics</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="queue">
				<div class="flex justify-between p-2">
					<p>Next From: </p>
					<p>00:00:00 Remaining </p>
				</div>
			</Tabs.Content>
			<Tabs.Content value="recent">
				
			</Tabs.Content>
			<Tabs.Content value="lyrics">
				
			</Tabs.Content>
		</Tabs.Root>
	</div>

	<div class="app-now-playing bg-muted grid gap-3 p-2 rounded-md items-center">
		<ArtworkDisplay id={playing.id} size={64} />

		<div class="flex flex-col">
			<p class="text-sm font-medium truncate">Song Name</p>
			<p class="text-xs text-muted-foreground truncate">Artist Name</p>
		</div>

		<div class="flex flex-col">
			<Button variant="ghost" size="icon">
				<Rows4 />
			</Button>
			<Button variant="ghost" size="icon">
				<BadgePlus />
			</Button>
		</div>
	</div>
</div>

<style>
.app-now-playing {
	grid-template-columns: auto 1fr auto;
	max-height: 85px;
}
</style>