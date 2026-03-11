<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";

	import TrackArtwork from "../TrackArtwork.svelte";

	type Track = {
		id: number;
		path: string;
		last_modified: number;
		title: string | null;
		artists: string | null;
		album_artist: string | null;
		albums: string | null;
		genres: string | null;
		year: string | null;
		rating: number | null;
		tags: string | null;
		duration_ms: number | null;
		bpm: number | null;
		key: string | null;
	};

	let tracks: Track[] = $state([]);
	let track: Track = $state();

	onMount(async () => {
		tracks = await invoke("get_tracks");
		track = tracks[0];
	});
</script>

<div class="flex flex-col gap-2 p-4 border-2 h-full w-full overflow-hidden rounded-md">

	<div>

		<TrackArtwork id={track.id} />

		<div>
			<h2 class="heading">{track.title}</h2>
			<div>
				<span>{track.album_artist}</span>
				<span>{track.albums}</span>
				<span>{track.year}</span>
				<span>{track.duration_ms}</span>
				<span>{track.rating}</span>
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

<style>

</style>