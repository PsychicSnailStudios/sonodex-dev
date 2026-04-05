<script lang="ts">
	
	// APP
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	
	// COMPONENTS
   import { Pencil } from "lucide-svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
   import NavButtons from "$lib/components/app-ui/NavButtons.svelte";
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";
   import TrackRating from "$lib/components/app-ui/TrackRating.svelte";
	import TagList from "$lib/components/app-ui/TagList.svelte";

	// SCRIPTS
	import { getArtistUidFromName, library } from "$lib/ts/library.svelte";
	import { selection, setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { formatDuration, parseAlbumEntries, parseArtists, parseTags } from '$lib/ts/util/helpers';
   import { playTrackByObject } from "$lib/ts/audio/audioManager.svelte";
	import type { Lyrics } from "$lib/ts/util/types";

	// VARIABLES
	let track = $derived(library.tracks.find(t => t.uid === selection.uid) ?? null);
	let lyrics: Lyrics | null = $state(null);
	let tags: string[] | null = $state(null);

	let artistUID = getArtistUidFromName(track?.album_artist ?? "Unknown Artist");
	
	// APP FUNCTIONS
	onMount(async () => {
		lyrics = await invoke("get_track_lyrics", { uid: selection.uid });
		tags = track?.tags ? parseTags(track.tags) : null;
	});
</script>

<div class="flex flex-col gap-4 p-4 border-2 h-full w-full overflow-hidden rounded-md">
		<NavButtons />
	
		{#if track}
			<div class="flex gap-4 items-center">
				<ArtworkDisplay uid={track.uid} size={160} type="track" />

				<div class="flex flex-col gap-1">
					<h2 class="text-2xl font-bold">{track.title ?? "Unknown Title"}</h2>
					<div class="flex gap-2 text-sm text-muted-foreground flex-wrap">
						<button onclick={() => setSelection(getArtistUidFromName(track.album_artist), "artist")} class="text-sm truncate cursor-pointer hover:underline">
							{parseArtists(track.artists)}
						</button>
						<span>|</span>
						<div>
							{#each parseAlbumEntries(track.albums) as album, i}
								<button onclick={() => setSelection(album.uid, "album")} class="text-sm truncate cursor-pointer hover:underline">
									{album.name}
									{#if i < parseAlbumEntries(track.albums).length - 1}<span>,</span>{/if}
								</button>
							{/each}
						</div>
						<span>|</span>
						<span>{track.year ?? "—"}</span>
						<span>|</span>
						<span>{formatDuration(track.duration_ms)}</span>
					</div>
					<div>
						<TrackRating uid={track.uid} rating={track.rating} />
					</div>
				</div>
			</div>

			<div class="flex gap-1 flex-wrap">
				<Button variant="default" onclick={() => playTrackByObject(track)}>Play</Button>
				<TrackPlaylistEditButton track={track} />
				<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "track", uid: track!.uid })}><Pencil /></Button>
			</div>

			<Tabs.Root value="credits" class="flex flex-col min-h-0 flex-1">
				<Tabs.List>
					<Tabs.Trigger value="lyrics">Lyrics</Tabs.Trigger>
					<Tabs.Trigger value="tags">Tags</Tabs.Trigger>
					<Tabs.Trigger value="credits">Credits</Tabs.Trigger>
					<Tabs.Trigger value="explore">Featured On</Tabs.Trigger>
				</Tabs.List>

				<Tabs.Content value="lyrics" class="flex-1 overflow-y-auto mt-2">
					{#if lyrics?.instrumental}
						<p class="text-muted-foreground text-sm">This track is instrumental.</p>
					{:else if lyrics?.plain}
						<pre class="text-sm whitespace-pre-wrap font-sans leading-relaxed">{lyrics.plain}</pre>
					{:else}
						<p class="text-muted-foreground text-sm">No lyrics available.</p>
					{/if}
				</Tabs.Content>

				<Tabs.Content value="tags" class="flex-1 overflow-y-auto mt-2">
					{#if tags}
						<TagList tags={tags} canEdit={true} />
					{:else}
						<p class="text-muted-foreground text-sm">Track has no tags.</p>
					{/if}
				</Tabs.Content>

				<Tabs.Content value="credits" class="flex-1 overflow-y-auto mt-2">
					<h4 class="text-foreground text-lg">Main Artist</h4>

					{#if track.album_artist}
						<button
							onclick={() => setSelection(artistUID, "artist")}
							class="flex items-center gap-2 flex-row cursor-pointer p-2 rounded-md bg-muted/50 hover:bg-muted">

							<ArtworkDisplay uid={artistUID} type="artist" size={40} />
							{track.album_artist}
						</button>
					{:else}
						<p class="text-muted-foreground text-sm">No album artist available.</p>
					{/if}

					<h4 class="text-foreground text-lg mt-4">Featured Artists</h4>
					{#if track.artists}
						<!-- {#each track.artists as artist}
							<button
								onclick={() => setSelection(artistUID, "artist")}
								class="flex items-center gap-2 flex-row cursor-pointer p-2 rounded-md bg-muted/50 hover:bg-muted">

								<ArtworkDisplay uid={artistUID} type="artist" size={40} />
								{artist}
							</button>
						{/each} -->
					{/if}

					<h4 class="text-foreground text-lg mt-4">Credits</h4>
					{#if track.credits}
						<pre class="text-sm whitespace-pre-wrap font-sans">{track.credits}</pre>
					{:else}
						<p class="text-muted-foreground text-sm">No credits available.</p>
					{/if}
				</Tabs.Content>

				<Tabs.Content value="explore" class="flex-1 overflow-y-auto mt-2">
					<p class="text-muted-foreground text-sm">Nothing here yet.</p>
				</Tabs.Content>
			</Tabs.Root>
		{:else}
			<span class="text-muted-foreground text-sm">Loading...</span>
		{/if}
</div>
