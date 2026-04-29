<script lang="ts">
	import { Trash, CloudDownload, FolderInput, Loader2 } from "lucide-svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { open } from "@tauri-apps/plugin-dialog";
	import { onMount } from "svelte";

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

	import Fuse from "fuse.js";
	import {
		removeTracksFromLibrary,
		enrichTracks,
		removeAlbums,
		enrichAlbums,
		removeArtists,
		enrichArtists,
		getDuplicates,
	} from "$lib/ts/app/libraryManager";
	import { library, loadLibrary } from "$lib/ts/library.svelte";
	import type { DuplicateGroup } from "$lib/ts/util/types";
	import { scanState } from "$lib/ts/app-states/state_session.svelte";
	import { enrichAllAlbums, enrichAllArtists, enrichAllTracks } from "$lib/ts/app/enrichment";
   import { parseAlbum, parseArtists } from "$lib/ts/util/helpers";

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

	// ─── Paths ────────────────────────────────────────────────────────────────────
	let paths = $state<{ id: number; path: string }[]>([]);
	let newPath = $state("");
	let removingPath = $state<string | null>(null);

	onMount(async () => {
		await loadPaths();

		await listen("scan:progress", async (event: any) => {
			scanState.loading = true;
			scanState.progress = event.payload.scanned;
			scanState.total = event.payload.total;
			scanState.status = `Scanning... ${scanState.progress} / ${scanState.total}`;
		});

		await listen("scan:done", async () => {
			scanState.status = "Scan done.";
			scanState.loading = false;
			scanState.progress = 0;
			scanState.total = 0;
			await loadLibrary();
		});

		await listen("scan:error", (event: any) => {
			scanState.status = `Scan error: ${event.payload}`;
			scanState.loading = false;
		});
	});

	async function loadPaths() {
		const result = await invoke("get_paths");
		paths = result as { id: number; path: string }[];
	}

	async function browsePath() {
		const selected = await open({ directory: true, multiple: false });
		if (selected) newPath = selected as string;
	}

	async function addPath() {
		if (!newPath.trim()) return;
		scanState.loading = true;
		scanState.status = "Scanning...";
		scanState.progress = 0;
		scanState.total = 0;
		try {
			await invoke("add_path", { path: newPath.trim() });
			newPath = "";
			await loadPaths();
		} catch (e) {
			scanState.status = `Error: ${e}`;
			scanState.loading = false;
		}
	}

	async function removePath(path: string) {
		removingPath = path;
		await invoke("remove_path", { path });
		await loadPaths();
		await loadLibrary();
		scanState.status = `Removed ${path}`;
		removingPath = null;
	}

	async function rescan() {
		scanState.loading = true;
		scanState.status = "Rescanning...";
		scanState.progress = 0;
		scanState.total = 0;
		await invoke("rescan");
	}

	let tracksFuseInstance: Fuse<(typeof library.tracks)[0]> | null = $state(null);
	let lastTracksRef: typeof library.tracks | null = null;

	function getTracksFuse() {
		if (tracksFuseInstance && lastTracksRef === library.tracks) return tracksFuseInstance;
		lastTracksRef = library.tracks;
		tracksFuseInstance = new Fuse(library.tracks, {
			keys: [
				{ name: "title",        weight: 0.5,  getFn: (t) => t.title ?? ""                    },
				{ name: "artists",      weight: 0.25, getFn: (t) => parseArtists(t.artists ?? "[]")  },
				{ name: "album_artist", weight: 0.15, getFn: (t) => t.album_artist ?? ""             },
				{ name: "albums",       weight: 0.1,  getFn: (t) => parseAlbum(t.albums ?? "[]")     },
				{ name: "tags",         weight: 0.05, getFn: (t) => t.tags ?? ""                     },
				{ name: "genres",       weight: 0.05, getFn: (t) => t.genres ?? ""                   },
			],
			threshold: 0.35,
			ignoreLocation: true,
			includeScore: false,
			useExtendedSearch: false,
			minMatchCharLength: 2,
		});
		return tracksFuseInstance;
	}

	const filteredTracks = $derived(
		(() => {
			const pool = ghosts ? library.tracks.filter((t) => t.path === "") : library.tracks;
			return trackSearch.trim().length < 2
				? pool
				: getTracksFuse().search(trackSearch).map((r) => r.item).filter((t) => !ghosts || t.path === "");
		})()
	);

	let albumFuseInstance: Fuse<(typeof library.albums)[0]> | null = $state(null);
	let lastAlbumsRef: typeof library.albums | null = null;

	function getAlbumFuse() {
		if (albumFuseInstance && lastAlbumsRef === library.albums) return albumFuseInstance;
		lastAlbumsRef = library.albums;
		albumFuseInstance = new Fuse(library.albums, {
			keys: [
				{ name: "title",        weight: 0.5,  getFn: (t) => t.title ?? ""                    },
				{ name: "artists",      weight: 0.25, getFn: (t) => parseArtists(t.artists ?? "[]")  },
				{ name: "album_artist", weight: 0.15, getFn: (t) => t.album_artist ?? ""             },
				{ name: "year",         weight: 0.1,  getFn: (t) => t.release_date ?? ""             },
				{ name: "tags",         weight: 0.05, getFn: (t) => t.tags ?? ""                     },
				{ name: "genres",       weight: 0.05, getFn: (t) => t.genres ?? ""                   },
			],
			threshold: 0.35,
			ignoreLocation: true,
			includeScore: false,
			useExtendedSearch: false,
			minMatchCharLength: 2,
		});
		return albumFuseInstance;
	}

	const filteredAlbums = $derived(
		albumSearch.trim().length < 2
			? library.albums
			: getAlbumFuse().search(albumSearch).map((r) => r.item)
	);

	let artistFuseInstance: Fuse<(typeof library.artists)[0]> | null = $state(null);
	let lastArtistsRef: typeof library.artists | null = null;

	function getArtistFuse() {
		if (artistFuseInstance && lastArtistsRef === library.artists) return artistFuseInstance;
		lastArtistsRef = library.artists;
		artistFuseInstance = new Fuse(library.artists, {
			keys: [
				{ name: "name",   weight: 0.5,  getFn: (t) => t.name ?? ""   },
				{ name: "akas",   weight: 0.35, getFn: (t) => t.aka ?? ""    },
				{ name: "tags",   weight: 0.05, getFn: (t) => t.tags ?? ""   },
				{ name: "genres", weight: 0.05, getFn: (t) => t.genres ?? "" },
			],
			threshold: 0.35,
			ignoreLocation: true,
			includeScore: false,
			useExtendedSearch: false,
			minMatchCharLength: 2,
		});
		return artistFuseInstance;
	}

	const filteredArtists = $derived(
		artistSearch.trim().length < 2
			? library.artists
			: getArtistFuse().search(artistSearch).map((r) => r.item)
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

	async function bulkRemoveAlbums() {
		await removeAlbums(selectedAlbumUids);
		albumSelections = {};
	}

	async function bulkRemoveArtists() {
		await removeArtists(selectedArtistUids);
		artistSelections = {};
	}

	async function bulkEnrichTracks() {
		scanState.enriching = true;
		scanState.enrichDone = 0;
		scanState.enrichTotal = 0;
		scanState.enrichErrors = 0;
		try {
			await enrichTracks(selectedTrackUids);
		} catch (e) {
			scanState.status = `Enrich error: ${e}`;
		} finally {
			scanState.enriching = false;
		}
	}

	async function bulkEnrichAlbums() {
		scanState.enriching = true;
		scanState.enrichDone = 0;
		scanState.enrichTotal = 0;
		scanState.enrichErrors = 0;
		try {
			await enrichAlbums(selectedAlbumUids);
		} catch (e) {
			scanState.status = `Enrich error: ${e}`;
		} finally {
			scanState.enriching = false;
		}
	}

	async function bulkEnrichArtists() {
		scanState.enriching = true;
		scanState.enrichDone = 0;
		scanState.enrichTotal = 0;
		scanState.enrichErrors = 0;
		try {
			await enrichArtists(selectedArtistUids);
		} catch (e) {
			scanState.status = `Enrich error: ${e}`;
		} finally {
			scanState.enriching = false;
		}
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

	// ─── Duplicates ───────────────────────────────────────────────────────────────
	async function loadDuplicates() {
		duplicatesLoaded = false;
		duplicates = await getDuplicates();
		duplicatesLoaded = true;
	}

	function onDuplicateResolved() {
		loadDuplicates();
	}
</script>

<AddToAlbumDialog bind:open={addToAlbumOpen} trackUids={selectedTrackUids} />

<div class="flex flex-col gap-2 p-2 border-2 rounded-md h-full w-full overflow-hidden">
	<h2 class="h2">Library Manager</h2>

	<ScrollArea class="h-full w-full min-h-0 min-w-0">
		<div class="flex flex-col gap-4 p-2 pr-4">
			<Tabs.Root value="paths" class="flex flex-col min-h-0 flex-1">
				<Tabs.List class="w-full">
					<Tabs.Trigger value="paths" class="flex-1">Library</Tabs.Trigger>
					<Tabs.Trigger value="tracks" class="flex-1">Tracks</Tabs.Trigger>
					<Tabs.Trigger value="albums" class="flex-1">Albums</Tabs.Trigger>
					<Tabs.Trigger value="artists" class="flex-1">Artists</Tabs.Trigger>
					<Tabs.Trigger value="tags" class="flex-1">Tags</Tabs.Trigger>
					<Tabs.Trigger value="duplicates" class="flex-1" onclick={loadDuplicates}>Duplicates</Tabs.Trigger>
				</Tabs.List>
				

				<!-- PATHS -->
				<Tabs.Content value="paths">
					<div class="flex flex-col gap-4 pt-2">
						<div class="flex flex-col gap-2">
							<h4 class="text-sm font-semibold">Add Library Path</h4>
							<div class="flex gap-2">
								<input
									bind:value={newPath}
									placeholder="C:\Music or \\NAS\Music"
									class="flex-1 border rounded px-3 py-2 text-sm bg-background"
								/>
								<Button variant="outline" onclick={browsePath}>Browse</Button>
								<Button onclick={addPath} disabled={scanState.loading}>
									{#if scanState.loading}
										<Loader2 class="animate-spin w-4 h-4 mr-1" />
									{/if}
									Add & Scan
								</Button>
							</div>
						</div>

						<div class="flex flex-col gap-2">
							<h4 class="text-sm font-semibold">Watched Paths ({paths.length})</h4>
							{#each paths as p}
								<div class="flex items-center justify-between border rounded px-3 py-2 text-sm">
									<span class="truncate mr-2">{p.path}</span>
									<Button
										variant="destructive"
										size="sm"
										disabled={scanState.loading || removingPath === p.path}
										onclick={() => removePath(p.path)}
									>
										{#if removingPath === p.path}
											<Loader2 class="animate-spin w-4 h-4 mr-1" />
										{/if}
										Remove
									</Button>
								</div>
							{:else}
								<p class="text-sm text-muted-foreground">No paths added yet.</p>
							{/each}
						</div>

						<Button onclick={rescan} disabled={scanState.loading} class="w-fit">
							{#if scanState.loading}
								<Loader2 class="animate-spin w-4 h-4 mr-1" />
							{/if}
							Rescan All
						</Button>
					</div>
				</Tabs.Content>

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
											disabled={scanState.enriching}
										>
											{#if scanState.enriching}
												<Loader2 class="w-4 h-4 mr-1 animate-spin" />
											{:else}
												<CloudDownload class="w-4 h-4 mr-1" />
											{/if}
											Enrich
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
											disabled={scanState.enriching}
										>
											{#if scanState.enriching}
												<Loader2 class="w-4 h-4 mr-1 animate-spin" />
											{:else}
												<CloudDownload class="w-4 h-4 mr-1" />
											{/if}
											Enrich
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
											disabled={scanState.enriching}
										>
											{#if scanState.enriching}
												<Loader2 class="w-4 h-4 mr-1 animate-spin" />
											{:else}
												<CloudDownload class="w-4 h-4 mr-1" />
											{/if}
											Enrich
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