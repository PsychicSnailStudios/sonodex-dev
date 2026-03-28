<script lang="ts">
   import { invoke } from "@tauri-apps/api/core";
	import { currentlyPlaying } from "$lib/ts/audio/audioManager.svelte";
	import { setSelection } from "$lib/ts/session.svelte";
	import { formatDuration, formatRating, getArtworkColor, parseAlbum, parseArtists } from "$lib/ts/util/helpers";
	import { clearQueue, getQueuedTracks, player, getPlayedTracks } from "$lib/ts/audio/audioManager.svelte";
	import { getArtistUidFromName } from "$lib/ts/library.svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Button } from "$lib/components/ui/button";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
   import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";
   import { addTrackToQueue } from "$lib/ts/audio/audioManager.svelte";

	import { Rows4, BadgePlus, Square } from "lucide-svelte";

	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
	import TrackTable from "$lib/components/app-ui/track-table/TrackTable.svelte";
   import AddToPlaylist from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";
	import ScrollingText from "$lib/components/app-ui/ScrollingText.svelte";
    import { copySelectedNameToClipboard } from "$lib/ts/app/trackSelection.svelte";

	let showQueue = $state(false);

	let upcomingTracks = $derived(getQueuedTracks());
	let recentTracks = $derived(getPlayedTracks());

	let color = $state("rgb(30, 30, 30)")

	$effect(() => {
		const track = player.track;
		if (!track?.uid) return;

		invoke("get_track_artwork", { uid: track.uid }).then((bytes) => {
			if (bytes) getArtworkColor(bytes as number[], 0.3).then((c) => color = c);
		});
	});

	function formatTotalRemaining(): string {
		const queueMs = getQueuedTracks().reduce((acc, t) => acc + (t.duration_ms ?? 0), 0);
		const currentRemaining = (player.duration - player.currentTime) * 1000;
		const totalMs = queueMs + Math.max(0, currentRemaining);

		const totalSecs = Math.floor(totalMs / 1000);
		const h = Math.floor(totalSecs / 3600);
		const m = Math.floor((totalSecs % 3600) / 60);
		const s = totalSecs % 60;

		return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
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
			<Tabs.Content value="queue">
				<p class="text-xs text-muted-foreground">Now Playing</p>
				<ContextMenu.Root>
					<ContextMenu.Trigger>
						<div class="flex gap-2 p-2">
							<ArtworkDisplay uid={player.track!.uid} size={30} type="track" />
							<div class="min-w-0 grid">
								<span role="button" tabindex="0"
								onclick={() => setSelection(player.track!.uid, "track")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(player.track!.uid, "track"); }} class="text-sm truncate cursor-pointer hover:underline">
									{player.track!.title}
								</span>
								<span role="button" tabindex="0"
								onclick={() => setSelection(getArtistUidFromName(player.track!.album_artist!), "artist")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(getArtistUidFromName(player.track!.album_artist!), "artist"); }} class="text-xs text-muted-foreground truncate cursor-pointer hover:underline">
									{parseArtists(player.track!.artists)}
								</span>
							</div>
						</div>
					</ContextMenu.Trigger>
					<ContextMenu.Content>
						<ContextMenu.Group>
							<ContextMenu.Item onSelect={() => copySelectedNameToClipboard(player.track!)}>Copy Track & Artist Name</ContextMenu.Item>
							<ContextMenu.Item onSelect={() => addTrackToQueue(player.track!)}>Add to Queue</ContextMenu.Item>
						</ContextMenu.Group>
						<ContextMenu.Separator />
						<ContextMenu.Group>
							<ContextMenu.Item>
								<TrackPlaylistEditButton track={player.track!} isButton={false} />
							</ContextMenu.Item>
						</ContextMenu.Group>
					</ContextMenu.Content>
				</ContextMenu.Root>

				<div class="flex justify-between p-2">
					<p class="text-xs text-muted-foreground">Next up from: </p>
					<p class="text-xs text-muted-foreground">{formatTotalRemaining()} Remaining</p>
					<button class="text-xs text-muted-foreground" onclick={clearQueue}>Clear</button>
				</div>

				<ScrollArea class="min-h-0 min-w-0 h-[290px]">
					<div class="flex flex-col gap-0.5">
						{#each upcomingTracks as track}
						<ContextMenu.Root>
							<ContextMenu.Trigger>
								<div class="flex gap-2 p-2">
									<ArtworkDisplay uid={track.uid} size={30} type="track" />
									<div class="min-w-0 grid">
										<span role="button" tabindex="0" onclick={() => setSelection(track.uid, "track")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(track.uid, "track"); }} class="text-sm truncate cursor-pointer hover:underline">
											{track.title}
										</span>
										<span role="button" tabindex="0"
										onclick={() => setSelection(getArtistUidFromName(player.track!.album_artist!), "artist")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(getArtistUidFromName(player.track!.album_artist!), "artist"); }} class="text-xs text-muted-foreground truncate cursor-pointer hover:underline">
											{parseArtists(player.track!.artists)}
										</span>
									</div>
								</div>
								</ContextMenu.Trigger>
								<ContextMenu.Content>
									<ContextMenu.Group>
										<ContextMenu.Item onSelect={() => copySelectedNameToClipboard(track)}>Copy Track & Artist Name</ContextMenu.Item>
										<ContextMenu.Item onSelect={() => addTrackToQueue(track)}>Add to Queue</ContextMenu.Item>
									</ContextMenu.Group>
									<ContextMenu.Separator />
									<ContextMenu.Group>
										<ContextMenu.Item>
											<TrackPlaylistEditButton track={track} isButton={false} />
										</ContextMenu.Item>
									</ContextMenu.Group>
								</ContextMenu.Content>
							</ContextMenu.Root>
						{/each}
					</div>
				</ScrollArea>

			</Tabs.Content>
			<Tabs.Content value="recent">
				<ScrollArea class="min-h-0 min-w-0 h-[200px]">
					<div>
						{#each recentTracks as track}
						
						<ContextMenu.Root>
							<ContextMenu.Trigger>
								<div class="flex gap-2 p-2">
									<ArtworkDisplay uid={track.uid} size={30} type="track" />
									<div class="min-w-0 grid">
										<span role="button" tabindex="0" onclick={() => setSelection(track.uid, "track")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(track.uid, "track"); }} class="text-sm truncate cursor-pointer hover:underline">
											{track.title}
										</span>
										<span role="button" tabindex="0"
										onclick={() => setSelection(getArtistUidFromName(player.track!.album_artist!), "artist")} onkeydown={(e) => { if (e.key === 'Enter') setSelection(getArtistUidFromName(player.track!.album_artist!), "artist"); }} class="text-xs text-muted-foreground truncate cursor-pointer hover:underline">
											{parseArtists(player.track!.artists)}
										</span>
									</div>
								</div>
								</ContextMenu.Trigger>
								<ContextMenu.Content>
									<ContextMenu.Group>
										<ContextMenu.Item onSelect={() => copySelectedNameToClipboard(track)}>Copy Track & Artist Name</ContextMenu.Item>
										<ContextMenu.Item onSelect={() => addTrackToQueue(track)}>Add to Queue</ContextMenu.Item>
									</ContextMenu.Group>
									<ContextMenu.Separator />
									<ContextMenu.Group>
										<ContextMenu.Item>
											<TrackPlaylistEditButton track={track} isButton={false} />
										</ContextMenu.Item>
									</ContextMenu.Group>
								</ContextMenu.Content>
							</ContextMenu.Root>
						{/each}
					</div>
				</ScrollArea>
			</Tabs.Content>
			<Tabs.Content value="lyrics">
				<ScrollArea class="min-h-0 min-w-0 h-[200px]">
					<p>no lyrics</p>
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
