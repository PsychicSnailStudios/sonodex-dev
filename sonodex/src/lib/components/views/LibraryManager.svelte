<script lang="ts">
	import { Trash, CloudDownload, FolderInput } from "lucide-svelte";

	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";

	import SearchBar from "$lib/components/app-ui/search/SearchBar.svelte";
	import TrackRow from "$lib/components/app-ui/library-manager/TrackRow.svelte";
	import AlbumRow from "$lib/components/app-ui/library-manager/AlbumRow.svelte";
	import ArtistRow from "$lib/components/app-ui/library-manager/ArtistRow.svelte";
	import DuplicateGroupCard from "$lib/components/app-ui/library-manager/DuplicateGroupCard.svelte";
	import AddToAlbumDialog from "$lib/components/dialogs/AddToAlbumDialog.svelte";
	import TagManager from "$lib/components/app-ui/library-manager/TagManager.svelte";

	import {
		removeTracksFromLibrary,
		enrichTracks,
		removeAlbums,
		enrichAlbums,
		removeArtists,
		enrichArtists,
		getDuplicates,
	} from "$lib/ts/app/libraryManager";
	import { library } from "$lib/ts/library.svelte";
	import type { DuplicateGroup } from "$lib/ts/util/types";
	import { scanState } from "$lib/ts/app-states/state_session.svelte";
	import { enrichAllAlbums, enrichAllArtists, enrichAllTracks } from "$lib/ts/app/enrichment";

	// ─── Search ───────────────────────────────────────────────────────────────────
	let trackSearch = $state("");
	let albumSearch = $state("");
	let artistSearch = $state("");
	let ghosts = $state(false);

	// ─── Duplicates ───────────────────────────────────────────────────────────────
	let duplicates = $state<DuplicateGroup[]>([]);
	let duplicatesLoaded = $state(false);

	// ─── Add to album dialog ──────────────────────────────────────────────────────
	let addToAlbumOpen = $state(false);

	// ─── Selections ───────────────────────────────────────────────────────────────
	let trackSelections = $state<Record<string, boolean>>({});
	let albumSelections = $state<Record<string, boolean>>({});
	let artistSelections = $state<Record<string, boolean>>({});

	let lastTrackIndex = $state<number | null>(null);
	let lastAlbumIndex = $state<number | null>(null);
	let lastArtistIndex = $state<number | null>(null);

	// ─── Filtered lists ───────────────────────────────────────────────────────────
	const filteredTracks = $derived(
		library.tracks.filter((t) => {
			const matchesGhost = ghosts ? t.path === "" : true;
			if (trackSearch.trim() === "") return matchesGhost;
			const q = trackSearch.toLowerCase();
			const matches =
				(t.title?.toLowerCase() ?? "").includes(q) ||
				(t.artists?.toLowerCase() ?? "").includes(q) ||
				(t.album_artist?.toLowerCase() ?? "").includes(q) ||
				(t.albums?.toLowerCase() ?? "").includes(q);
			return matches && matchesGhost;
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

	// ─── Track selection derived ──────────────────────────────────────────────────
	const selectedTrackUids = $derived(
		Object.entries(trackSelections).filter(([, v]) => v).map(([k]) => k)
	);
	const anyTracksSelected = $derived(selectedTrackUids.length > 0);
	const allTracksSelected = $derived(
		filteredTracks.length > 0 && filteredTracks.every((t) => trackSelections[t.uid])
	);

	// ─── Album selection derived ──────────────────────────────────────────────────
	const selectedAlbumUids = $derived(
		Object.entries(albumSelections).filter(([, v]) => v).map(([k]) => k)
	);
	const anyAlbumsSelected = $derived(selectedAlbumUids.length > 0);
	const allAlbumsSelected = $derived(
		filteredAlbums.length > 0 && filteredAlbums.every((a) => albumSelections[a.uid])
	);

	// ─── Artist selection derived ─────────────────────────────────────────────────
	const selectedArtistUids = $derived(
		Object.entries(artistSelections).filter(([, v]) => v).map(([k]) => k)
	);
	const anyArtistsSelected = $derived(selectedArtistUids.length > 0);
	const allArtistsSelected = $derived(
		filteredArtists.length > 0 && filteredArtists.every((a) => artistSelections[a.uid])
	);

	// ─── Shift-select helpers ─────────────────────────────────────────────────────
	function shiftSelectRange<T extends { uid: string }>(
		list: T[],
		clickedIndex: number,
		lastIndex: number | null,
		selections: Record<string, boolean>
	): Record<string, boolean> {
		const next = { ...selections };
		const from = lastIndex ?? clickedIndex;
		const lo = Math.min(from, clickedIndex);
		const hi = Math.max(from, clickedIndex);
		for (let i = lo; i <= hi; i++) {
			next[list[i].uid] = true;
		}
		return next;
	}

	function toggleTrack(uid: string, index: number) {
		trackSelections = { ...trackSelections, [uid]: !(trackSelections[uid] ?? false) };
		lastTrackIndex = index;
	}

	function shiftTrack(index: number) {
		trackSelections = shiftSelectRange(filteredTracks, index, lastTrackIndex, trackSelections);
		lastTrackIndex = index;
	}

	function toggleAlbum(uid: string, index: number) {
		albumSelections = { ...albumSelections, [uid]: !(albumSelections[uid] ?? false) };
		lastAlbumIndex = index;
	}

	function shiftAlbum(index: number) {
		albumSelections = shiftSelectRange(filteredAlbums, index, lastAlbumIndex, albumSelections);
		lastAlbumIndex = index;
	}

	function toggleArtist(uid: string, index: number) {
		artistSelections = { ...artistSelections, [uid]: !(artistSelections[uid] ?? false) };
		lastArtistIndex = index;
	}

	function shiftArtist(index: number) {
		artistSelections = shiftSelectRange(filteredArtists, index, lastArtistIndex, artistSelections);
		lastArtistIndex = index;
	}

	// ─── Bulk actions ─────────────────────────────────────────────────────────────
	async function bulkRemoveTracks() {
		await removeTracksFromLibrary(selectedTrackUids);
		trackSelections = {};
	}
	async function bulkEnrichTracks() {
		await enrichTracks(selectedTrackUids);
	}

	async function bulkRemoveAlbums() {
		await removeAlbums(selectedAlbumUids);
		albumSelections = {};
	}
	async function bulkEnrichAlbums() {
		await enrichAlbums(selectedAlbumUids);
	}

	async function bulkRemoveArtists() {
		await removeArtists(selectedArtistUids);
		artistSelections = {};
	}
	async function bulkEnrichArtists() {
		await enrichArtists(selectedArtistUids);
	}

	// ─── Duplicates ───────────────────────────────────────────────────────────────
	async function loadDuplicates() {
		duplicatesLoaded = false;
		duplicates = await getDuplicates();
		duplicatesLoaded = true;
	}

	function onDuplicateResolved() {
		loadDuplicates();
	}

	async function enrichAll() {
		scanState.enriching = true;
		scanState.enrichDone = 0;
		scanState.enrichTotal = 0;
		scanState.enrichErrors = 0;
		try {
			enrichAllTracks();
			enrichAllAlbums();
			enrichAllArtists();
		} catch (e) {
			scanState.status = `Enrich error: ${e}`;
			scanState.enriching = false;
		}
	}
	async function enrich(type: "tracks" | "albums" | "artists") {
		scanState.enriching = true;
		scanState.enrichDone = 0;
		scanState.enrichTotal = 0;
		scanState.enrichErrors = 0;
		try {
			switch (type) {
				case "tracks": enrichAllTracks(); break;
				case "albums": enrichAllAlbums(); break;
				case "artists": enrichAllArtists(); break;
			}
		} catch (e) {
			scanState.status = `Enrich error: ${e}`;
			scanState.enriching = false;
		}
	}
</script>

<AddToAlbumDialog bind:open={addToAlbumOpen} trackUids={selectedTrackUids} />

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
									Ghosts only
								</Button>
								<SearchBar bind:search={trackSearch} searchCount={filteredTracks.length} />
							</div>
						</div>
						<div class="flex gap-2 justify-between items-center">
							{#if anyTracksSelected}
								<div class="flex gap-2">
									<Checkbox
										checked={allTracksSelected}
										onCheckedChange={(v) => {
											const next: Record<string, boolean> = {};
											filteredTracks.forEach(t => next[t.uid] = !!v);
											trackSelections = next;
										}}
									/>
									<span class="text-xs text-muted-foreground">{selectedTrackUids.length} selected</span>
								</div>
								<div>
									<Tooltip.Root>
										<Tooltip.Trigger
											class={buttonVariants({ variant: "destructive", size: "sm" })}
											onclick={bulkRemoveTracks}
										>
											<Trash class="w-4 h-4 mr-1" /> Remove
										</Tooltip.Trigger>
										<Tooltip.Content><p>Remove selected from library</p></Tooltip.Content>
									</Tooltip.Root>
									<Tooltip.Root>
										<Tooltip.Trigger
											class={buttonVariants({ variant: "outline", size: "sm" })}
											onclick={bulkEnrichTracks}
										>
											<CloudDownload class="w-4 h-4 mr-1" /> Enrich
										</Tooltip.Trigger>
										<Tooltip.Content><p>Fetch metadata for selected</p></Tooltip.Content>
									</Tooltip.Root>
									<Tooltip.Root>
										<Tooltip.Trigger
											class={buttonVariants({ variant: "outline", size: "sm" })}
											onclick={() => (addToAlbumOpen = true)}
										>
											<FolderInput class="w-4 h-4 mr-1" /> Add to Album
										</Tooltip.Trigger>
										<Tooltip.Content><p>Add selected to an album</p></Tooltip.Content>
									</Tooltip.Root>
								</div>
							{/if}
						</div>
						<div class="flex flex-col gap-2">
							{#each filteredTracks as track, i (track.uid)}
								<TrackRow
									{track}
									selected={trackSelections[track.uid] ?? false}
									onToggle={() => toggleTrack(track.uid, i)}
									onShiftClick={() => shiftTrack(i)}
								/>
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
						<div class="flex gap-2 justify-between items-center">
							{#if anyAlbumsSelected}
								<div class="flex gap-2">
									<Checkbox
										checked={allAlbumsSelected}
										onCheckedChange={(v) => {
											const next: Record<string, boolean> = {};
											filteredAlbums.forEach(a => next[a.uid] = !!v);
											albumSelections = next;
										}}
									/>
									<span class="text-xs text-muted-foreground">{selectedAlbumUids.length} selected</span>
								</div>
								<div class="flex gap-2">
									<Tooltip.Root>
										<Tooltip.Trigger
											class={buttonVariants({ variant: "destructive", size: "sm" })}
											onclick={bulkRemoveAlbums}
										>
											<Trash class="w-4 h-4 mr-1" /> Remove
										</Tooltip.Trigger>
										<Tooltip.Content><p>Remove selected albums</p></Tooltip.Content>
									</Tooltip.Root>
									<Tooltip.Root>
										<Tooltip.Trigger
											class={buttonVariants({ variant: "outline", size: "sm" })}
											onclick={bulkEnrichAlbums}
										>
											<CloudDownload class="w-4 h-4 mr-1" /> Enrich
										</Tooltip.Trigger>
										<Tooltip.Content><p>Fetch metadata for selected</p></Tooltip.Content>
									</Tooltip.Root>
								</div>
							{/if}
						</div>
						<div class="flex flex-col gap-2">
							{#each filteredAlbums as album, i (album.uid)}
								<AlbumRow
									{album}
									selected={albumSelections[album.uid] ?? false}
									onToggle={() => toggleAlbum(album.uid, i)}
									onShiftClick={() => shiftAlbum(i)}
								/>
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
						<div class="flex gap-2 justify-between items-center">
							{#if anyArtistsSelected}
								<div class="flex gap-2">
									<Checkbox
										checked={allArtistsSelected}
										onCheckedChange={(v) => {
											const next: Record<string, boolean> = {};
											filteredArtists.forEach(a => next[a.uid] = !!v);
											artistSelections = next;
										}}
									/>
									<span class="text-xs text-muted-foreground">{selectedArtistUids.length} selected</span>
								</div>
								<div class="flex gap-2">
									<Tooltip.Root>
										<Tooltip.Trigger
											class={buttonVariants({ variant: "destructive", size: "sm" })}
											onclick={bulkRemoveArtists}
										>
											<Trash class="w-4 h-4 mr-1" /> Remove
										</Tooltip.Trigger>
										<Tooltip.Content><p>Remove selected artists</p></Tooltip.Content>
									</Tooltip.Root>
									<Tooltip.Root>
										<Tooltip.Trigger
											class={buttonVariants({ variant: "outline", size: "sm" })}
											onclick={bulkEnrichArtists}
										>
											<CloudDownload class="w-4 h-4 mr-1" /> Enrich
										</Tooltip.Trigger>
										<Tooltip.Content><p>Fetch metadata for selected</p></Tooltip.Content>
									</Tooltip.Root>
								</div>
							{/if}
						</div>
						<div class="flex flex-col gap-2">
							{#each filteredArtists as artist, i (artist.uid)}
								<ArtistRow
									{artist}
									selected={artistSelections[artist.uid] ?? false}
									onToggle={() => toggleArtist(artist.uid, i)}
									onShiftClick={() => shiftArtist(i)}
								/>
							{/each}
						</div>
					</div>
				</Tabs.Content>

				<Tabs.Content value="tags">
					<TagManager />
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