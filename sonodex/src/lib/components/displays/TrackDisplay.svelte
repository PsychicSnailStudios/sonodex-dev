<script lang="ts">

	// APP
	import { invoke } from "@tauri-apps/api/core";
	
	// COMPONENTS
   import { Pencil } from "lucide-svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
	import { Button } from "$lib/components/ui/button/index.js";

	// CUSTOM COMPONENTS
	import ArtworkDisplay from "$lib/components/app-ui/ArtworkDisplay.svelte";
   import NavButtons from "$lib/components/app-ui/NavButtons.svelte";
   import TrackPlaylistEditButton from "$lib/components/app-ui/TrackPlaylistEditButton.svelte";
   import TrackRating from "$lib/components/app-ui/TrackRating.svelte";
	import TagList from "$lib/components/app-ui/TagList.svelte";
   import ArtistsList from "$lib/components/app-ui/ArtistsList.svelte";
   import LyricsViewer from "$lib/components/app-ui/LyricsViewer.svelte";
	import DownloadButton from "$lib/components/app-ui/DownloadButton.svelte";

	// SCRIPTS
	import { getAlbumUidFromName, getArtistUidFromName, getLyrics, library } from "$lib/ts/library.svelte";
	import { currentTrackTab, selection, setSelection } from "$lib/ts/app-states/state_session.svelte";
	import { openEditModal } from "$lib/ts/app/editModal.svelte";
	import { formatDuration, getArtworkColor, parseAlbumEntries, parseTags } from '$lib/ts/util/helpers';
   import { playTrackByObject } from "$lib/ts/audio/audioManager.svelte";

	// VARIABLES
	let track = $derived(library.tracks.find(t => t.uid === selection.uid) ?? null);
	let fetchingLyrics = $state(false);
	let genres = $derived(track?.genres ? parseTags(track.genres) : null);
	let tags = $derived(track?.tags ? parseTags(track.tags) : null);
	let color = $state("rgb(30, 30, 30)")

	// All albums this track appears on
	let featuredOnAlbums = $derived.by(() => {
		if (!track?.albums) return [];
		return parseAlbumEntries(track.albums)
			.map(entry => library.albums.find(a => a.uid === entry.uid))
			.filter((a): a is NonNullable<typeof a> => a != null);
	});

	// Featured artists = all artists minus the album artist
	let featuredArtists = $derived.by(() => {
		if (!track?.artists) return [];
		const all: string[] = JSON.parse(track.artists);
		const main = track.album_artist?.toLowerCase() ?? "";
		return all.filter(a => a.toLowerCase() !== main);
	});

	let featuredArtistUIDs = $derived(
		featuredArtists.map(name => ({ name, uid: getArtistUidFromName(name) }))
	);

	let artistUID = $derived(getArtistUidFromName(track?.album_artist ?? "Unknown Artist"));

	$effect(() => {
		const uid = selection.uid;
		if (!uid) return;

		color = "var(--muted)";

		invoke("get_track_artwork", { uid: uid }).then((trackBytes) => {
			if (trackBytes) getArtworkColor(trackBytes as number[], 0.3).then((c) => color = c);
		});
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
						<ArtistsList artists={track.artists} />
						<span>|</span>
						<div>
							{#if track.albums}
							{@const albumList = parseAlbumEntries(track.albums)}
							{#each albumList as album, i}
								<button onclick={() => setSelection(getAlbumUidFromName(album.name), "album")} class="text-sm truncate cursor-pointer hover:underline">
									{album.name}{i < albumList.length - 1 ? "," : ""}
								</button>
							{/each}
							{/if}

						</div>
						{#if track.year}
							<span>|</span>
							<span>{track.year}</span>
						{/if}
						<span>|</span>
						<span>{formatDuration(track.duration_ms)}</span>
						<span>|</span>
						<span>{track.format ?? "Unknown Format"}</span>
						<span>|</span>
						<span>{track.bitrate ?? "0"}</span><span>kbps</span>
					</div>
					<div>
						<TrackRating uid={track.uid} rating={track.rating} tags={track.tags} />
					</div>
				</div>
			</div>

			<div class="flex gap-2 justify-between items-center flex-wrap p-2 rounded-md"
				  style="background: {color};">
				<Button variant="default" onclick={() => playTrackByObject(track)}>Play</Button>
				<div>
					<TrackPlaylistEditButton track={track} />
					<Button variant="ghost" size="icon" onclick={() => openEditModal({ type: "track", uid: track!.uid })}><Pencil /></Button>
					<DownloadButton uid={track.uid} variant="ghost" />
				</div>
			</div>

			<Tabs.Root bind:value={currentTrackTab.id} class="flex flex-col min-h-0 flex-1">
				<Tabs.List>
					<Tabs.Trigger value="lyrics">Lyrics</Tabs.Trigger>
					<Tabs.Trigger value="tags">Tags</Tabs.Trigger>
					<Tabs.Trigger value="credits">Credits</Tabs.Trigger>
					<Tabs.Trigger value="explore">Featured On</Tabs.Trigger>
					<Tabs.Trigger value="paths">File Paths</Tabs.Trigger>
				</Tabs.List>

				<Tabs.Content value="lyrics" class="flex-1 overflow-y-auto mt-2">
					<ScrollArea class="min-h-0 min-w-0 h-full pl-4">
						<LyricsViewer uid={track.uid} />
					</ScrollArea>
				</Tabs.Content>

				<Tabs.Content value="tags" class="flex-1 overflow-y-auto mt-2">
					{#if tags}
						<TagList uid={track.uid} tags={genres} canEdit={false} />
						<TagList uid={track.uid} tags={tags} canEdit={true} />
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
					{#if featuredArtists.length > 0}
						{#each featuredArtistUIDs as { name: artistName, uid: featArtistUID }}
							<button
								onclick={() => setSelection(featArtistUID, "artist")}
								class="flex items-center gap-2 flex-row cursor-pointer p-2 mb-2 rounded-md bg-muted/50 hover:bg-muted">

								<ArtworkDisplay uid={featArtistUID} type="artist" size={40} />
								{artistName}
							</button>
						{/each}
					{:else}
						<p class="text-muted-foreground text-sm">No featured artists.</p>
					{/if}

					<h4 class="text-foreground text-lg mt-4">Credits</h4>
					{#if track.credits}
						<pre class="text-sm whitespace-pre-wrap font-sans">{track.credits}</pre>
					{:else}
						<p class="text-muted-foreground text-sm">No credits available.</p>
					{/if}

					{#if track.label}
						<h4 class="text-foreground text-lg mt-4">Label</h4>
						<pre class="text-sm whitespace-pre-wrap font-sans">{track.label}</pre>
					{/if}
				</Tabs.Content>

				<Tabs.Content value="explore" class="flex-1 overflow-y-auto mt-2">
					{#if featuredOnAlbums.length > 0}
						<div class="flex flex-col gap-2 mt-2">
							{#each featuredOnAlbums as album}
								<button
									onclick={() => setSelection(album.uid, "album")}
									class="flex items-center gap-3 cursor-pointer p-2 rounded-md bg-muted/50 hover:bg-muted text-left">
									<ArtworkDisplay uid={album.uid} type="album" size={48} />
									<div class="flex flex-col">
										<span class="text-sm font-medium">{album.title}</span>
										<span class="text-xs text-muted-foreground">{album.album_artist}</span>
									</div>
								</button>
							{/each}
						</div>
					{:else}
						<p class="text-muted-foreground text-sm">No albums found.</p>
					{/if}
				</Tabs.Content>

				<Tabs.Content value="paths" class="flex-1 overflow-y-auto mt-2">
					<div class="flex flex-col gap-2 mt-2">
						<span class="text-muted-foreground text-sm"><b>Local Path:</b> {track.path}</span>
						<span class="text-muted-foreground text-sm"><b>Remote Path:</b> {track.remote_path}</span>
					</div>
				</Tabs.Content>
			</Tabs.Root>
		{:else}
			<span class="text-muted-foreground text-sm">Loading...</span>
		{/if}
</div>