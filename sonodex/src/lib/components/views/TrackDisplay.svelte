<script lang="ts">
	import type { Track } from "$lib/types";

	import * as Tabs from "$lib/components/ui/tabs/index.js";

	import TrackArtwork from "$lib/components/app/TrackArtwork.svelte";

	let { track } = $props<{ track: Track }>();
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div>
		<TrackArtwork id={track[0].id} width={100} height={100} />

		<div>
			<h2 class="heading">{track[0].title}</h2>
			<div>
				<span>{track[0].album_artist}</span>
				<span>{track[0].albums}</span>
				<span>{track[0].year}</span>
				<span>{track[0].duration_ms}</span>
				<span>{track[0].rating}</span>
			</div>
		</div>

	</div>

	<Tabs.Root value="account" class="w-[400px]">
		<Tabs.List>
			<Tabs.Trigger value="account">Account</Tabs.Trigger>
			<Tabs.Trigger value="password">Password</Tabs.Trigger>
		</Tabs.List>
		<Tabs.Content value="account">
			Make changes to your account here.
		</Tabs.Content>
		<Tabs.Content value="password">Change your password here.</Tabs.Content>
	</Tabs.Root>

</div>