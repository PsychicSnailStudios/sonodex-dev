<script lang="ts">

	// APP
   import { checkPermissions, invoke } from "@tauri-apps/api/core";
   import { onMount } from "svelte";
	
	// COMPONENTS
	import { Rows4, BadgePlus, Square } from "lucide-svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
	import { Button } from "$lib/components/ui/button";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
   import AddToPlaylist from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";
	import ScrollingText from "$lib/components/app-ui/ScrollingText.svelte";
	import Queue from "$lib/components/app/now-playing/Queue.svelte";
	import RecentlyPlayed from "$lib/components/app/now-playing/RecentlyPlayed.svelte";

	// SCRIPTS
	import { setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { getArtistUidFromName } from "$lib/ts/library.svelte";
	import { currentlyPlaying, player } from "$lib/ts/audio/audioManager.svelte";
	import { getArtworkColor, parseArtists } from "$lib/ts/util/helpers";
   import type { Lyrics } from "$lib/ts/util/types";

	// VARIABLES
	let showQueue = $state(false);
	let lyrics: Lyrics | null = $state(null);

	let color = $state("var(--muted)");

	// APP FUNCTIONS
	$effect(() => {
		color = "var(--muted)";

		const track = player.track;
		if (!track?.uid) return;

		invoke("get_track_artwork", { uid: track.uid }).then((bytes) => {
			if (bytes) getArtworkColor(bytes as number[], 0.3).then((c) => color = c);
		});
	});

	async function updateLyrics() {
		lyrics = await invoke("get_track_lyrics", { uid: currentlyPlaying.track.uid });
	}

</script>

<div class="app-now-playing-wrapper flex flex-col gap-1 bg-muted rounded-md">
	{#if showQueue}
	<div class="app-queue p-2 rounded-md h-[450px]">
		<Tabs.Root value="queue">
			<Tabs.List>
				<Tabs.Trigger value="queue">Queue</Tabs.Trigger>
				<Tabs.Trigger value="recent">Recently Played</Tabs.Trigger>
				<Tabs.Trigger value="lyrics">Lyrics</Tabs.Trigger>
			</Tabs.List>
			<Queue />
			<RecentlyPlayed />
			<Tabs.Content value="lyrics">
				<ScrollArea class="min-h-0 min-w-0 h-[200px]">
					{#if lyrics?.instrumental}
						<p class="text-muted-foreground text-sm">This track is instrumental.</p>
					{:else if lyrics?.plain}
						<pre class="text-sm whitespace-pre-wrap font-sans leading-relaxed">{lyrics.plain}</pre>
					{:else}
						<p class="text-muted-foreground text-sm">No lyrics available.</p>
					{/if}
				</ScrollArea>
			</Tabs.Content>
		</Tabs.Root>
	</div>
	{/if}

	<div class="app-now-playing bg-muted grid gap-3 p-2 rounded-md items-center" style="background: linear-gradient(90deg, {color} 0%, transparent 75%)">
		{#if currentlyPlaying.track !== null}
			<ArtworkDisplay uid={currentlyPlaying.uid} type="track" size={64} />

			<div class="flex flex-col min-w-0">
				<ScrollingText
					text={currentlyPlaying.track?.title!}
					class="text-sm font-medium cursor-pointer hover:underline"
					onclick={() => setSelection(currentlyPlaying.track?.uid!, "track")}
					
				/>
				<ScrollingText
					text={parseArtists(currentlyPlaying.track?.artists ?? null)}
					class="text-xs text-muted-foreground cursor-pointer hover:underline"
					onclick={() => setSelection(getArtistUidFromName(currentlyPlaying.track?.album_artist!), "artist")}
					hoverOnly
				/>
			</div>

			<div class="flex flex-col">
				<Button variant="ghost" size="icon" onclick={() => showQueue = !showQueue}>
					{#if showQueue}
						<Square />
					{:else}
						<Rows4 />
					{/if}
				</Button>
				<AddToPlaylist track={currentlyPlaying.track!} />
			</div>
		{/if}
	</div>
</div>

<style>
.app-now-playing {
	grid-template-columns: auto 1fr auto;
	max-height: 85px;
	height: 85px;
}
</style>
