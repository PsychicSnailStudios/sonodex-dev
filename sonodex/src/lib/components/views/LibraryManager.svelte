<script lang="ts">
	import { onMount } from "svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";
	import TrackRow from "$lib/components/app-ui/library-manager/TrackRow.svelte";
	import AlbumRow from "$lib/components/app-ui/library-manager/AlbumRow.svelte";
	import ArtistRow from "$lib/components/app-ui/library-manager/ArtistRow.svelte";
	import DuplicateGroupCard from "$lib/components/app-ui/library-manager/DuplicateGroupCard.svelte";

	import { library } from "$lib/ts/library.svelte";
	import { getDuplicates } from "$lib/ts/app/libraryManager";
	import type { DuplicateGroup } from "$lib/ts/types";

	let trackSearch = $state("");
	let albumSearch = $state("");
	let artistSearch = $state("");
	let ghosts = $state(false);

	let duplicates = $state<DuplicateGroup[]>([]);
	let duplicatesLoaded = $state(false);

	async function loadDuplicates() {
		duplicatesLoaded = false;
		duplicates = await getDuplicates();
		duplicatesLoaded = true;
	}

	function onDuplicateResolved() {
		loadDuplicates();
	}

	const filteredTracks = $derived(
		trackSearch.trim() === ""
			? library.tracks.filter((t) => {
					if (ghosts) return t.path === "";
					return true;
				})
			: library.tracks.filter((t) => {
					const q = trackSearch.toLowerCase();
					const title = t.title?.toLowerCase() ?? "";
					const artists = t.artists?.toLowerCase() ?? "";
					const album_artist = t.album_artist?.toLowerCase() ?? "";
					const albums = t.albums?.toLowerCase() ?? "";
					const matches = title.includes(q) || artists.includes(q) || album_artist.includes(q) || albums.includes(q);
					if (ghosts) return matches && t.path === "";
					return matches;
				})
	);

	const filteredAlbums = $derived(
		albumSearch.trim() === ""
			? library.albums
			: library.albums.filter((a) => {
					const q = albumSearch.toLowerCase();
					return (
						a.title?.toLowerCase().includes(q) ||
						(a.album_artist?.toLowerCase().includes(q) ?? false)
					);
				})
	);

	const filteredArtists = $derived(
		artistSearch.trim() === ""
			? library.artists
			: library.artists.filter((a) => a.name?.toLowerCase().includes(artistSearch.toLowerCase()))
	);
</script>

<div class="flex flex-col gap-2 p-2 border-2 rounded-md h-full w-full overflow-hidden">
	<h2 class="h2">Library Manager</h2>

	<ScrollArea class="h-full w-full min-h-0 min-w-0">
		<div class="flex flex-col gap-4 p-2 pr-4">
			<Tabs.Root value="tracks" class="flex flex-col min-h-0 flex-1">
				<Tabs.List class="w-full">
					<Tabs.Trigger value="tracks" class="flex-1">Tracks</Tabs.Trigger>
					<Tabs.Trigger value="albums" class="flex-1">Albums</Tabs.Trigger>
					<Tabs.Trigger value="artists" class="flex-1">Artists</Tabs.Trigger>
					<Tabs.Trigger value="tags" class="flex-1">Tags</Tabs.Trigger>
					<Tabs.Trigger value="duplicates" class="flex-1" onclick={loadDuplicates}>Duplicates</Tabs.Trigger>
				</Tabs.List>

				<!-- TRACKS -->
				<Tabs.Content value="tracks">
					<div class="flex flex-col gap-2 pt-2">
						<div class="flex gap-2 justify-between items-center">
							<span class="text-sm text-muted-foreground">{filteredTracks.length} {filteredTracks.length === 1 ? "track" : "tracks"}</span>
							<div class="flex gap-2 items-center">
								<Button
									variant={ghosts ? "secondary" : "outline"}
									size="sm"
									onclick={() => (ghosts = !ghosts)}
								>
									Missing only
								</Button>
								<SearchBar bind:search={trackSearch} searchCount={filteredTracks.length} />
							</div>
						</div>
						<div class="flex flex-col gap-2">
							{#each filteredTracks as track (track.uid)}
								<TrackRow {track} />
							{/each}
						</div>
					</div>
				</Tabs.Content>

				<!-- ALBUMS -->
				<Tabs.Content value="albums">
					<div class="flex flex-col gap-2 pt-2">
						<div class="flex gap-2 justify-between items-center">
							<span class="text-sm text-muted-foreground">{filteredAlbums.length} {filteredAlbums.length === 1 ? "album" : "albums"}</span>
							<SearchBar bind:search={albumSearch} searchCount={filteredAlbums.length} />
						</div>
						<div class="flex flex-col gap-2">
							{#each filteredAlbums as album (album.uid)}
								<AlbumRow {album} />
							{/each}
						</div>
					</div>
				</Tabs.Content>

				<!-- ARTISTS -->
				<Tabs.Content value="artists">
					<div class="flex flex-col gap-2 pt-2">
						<div class="flex gap-2 justify-between items-center">
							<span class="text-sm text-muted-foreground">{filteredArtists.length} {filteredArtists.length === 1 ? "artist" : "artists"}</span>
							<SearchBar bind:search={artistSearch} searchCount={filteredArtists.length} />
						</div>
						<div class="flex flex-col gap-2">
							{#each filteredArtists as artist (artist.uid)}
								<ArtistRow {artist} />
							{/each}
						</div>
					</div>
				</Tabs.Content>

				<Tabs.Content value="tags">
					<p>Tags</p>
				</Tabs.Content>

				<!-- DUPLICATES -->
				<Tabs.Content value="duplicates">
					<div class="flex flex-col gap-2 pt-2">
						{#if !duplicatesLoaded}
							<p class="text-sm text-muted-foreground">Loading…</p>
						{:else if duplicates.length === 0}
							<p class="text-sm text-muted-foreground">No duplicates found.</p>
						{:else}
							<span class="text-sm text-muted-foreground">{duplicates.length} duplicate {duplicates.length === 1 ? "group" : "groups"}</span>
							<div class="flex flex-col gap-3">
								{#each duplicates as group, i (i)}
									<DuplicateGroupCard {group} onresolved={onDuplicateResolved} />
								{/each}
							</div>
						{/if}
					</div>
				</Tabs.Content>
			</Tabs.Root>
		</div>
	</ScrollArea>
</div>