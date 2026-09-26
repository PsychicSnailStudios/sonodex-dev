<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";

	import ArtworkDisplay from "$lib/components/custom/ArtworkDisplay.svelte";

	import { getAlbum, getTrack } from "$ts/store/library.svelte";
	import { setSelection } from "$ts/store/session.svelte";
	import { parseAlbumEntries } from "$ts/util/parsers";
	import type { Album } from "$ts/util/types";
    import MediaGrid from "$lib/layouts/MediaGrid.svelte";

	type Scrobble = {
		uid: string;
		timestamp: number;
		track_uid: string;
		artist_uid: string;
		duration_played: number;
		did_seek: boolean;
		did_pause: boolean;
		track_name: string | null;
		track_artist: string | null;
		track_album: string | null;
		shuffle: boolean | null;
		skipped: boolean | null;
		offline: boolean | null;
		playing_local: boolean | null;
	};

	const RESUME_LIMIT = 8;
	const SCROBBLE_SCAN_LIMIT = 300;
	const MIN_DURATION_MS = 20_000;

	let recentAlbums = $state<Album[]>([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			recentAlbums = await loadRecentAlbums();
		} catch (e) {
			console.error("Jump Back In failed to load", e);
		} finally {
			loading = false;
		}
	});

	async function loadRecentAlbums(): Promise<Album[]> {
		const scrobbles = await invoke<Scrobble[]>("get_scrobbles");
		const trackAlbumCache = new Map<string, string | null>();
		const seen = new Set<string>();
		const albumUids: string[] = [];

		for (const s of scrobbles.slice(0, SCROBBLE_SCAN_LIMIT)) {
			if (albumUids.length >= RESUME_LIMIT) break;
			if (!s.track_uid || s.duration_played < MIN_DURATION_MS) continue;

			let albumUid = trackAlbumCache.get(s.track_uid);
			if (albumUid === undefined) {
				const track = await getTrack(s.track_uid);
				albumUid = parseAlbumEntries(track?.albums ?? null)[0]?.uid ?? null;
				trackAlbumCache.set(s.track_uid, albumUid);
			}

			if (albumUid && !seen.has(albumUid)) {
				seen.add(albumUid);
				albumUids.push(albumUid);
			}
		}

		const albums = await Promise.all(albumUids.map((uid) => getAlbum(uid)));
		return albums.filter((a): a is Album => a !== null);
	}
</script>

<div class="flex flex-col gap-3">

	<div class="flex items-center justify-between gap-2 flex-wrap shrink-0">
		<h2 class="text-lg font-semibold tracking-tight">Jump Back In</h2>
	</div>

	{#if loading}
		<div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2">
			{#each Array(RESUME_LIMIT) as _}
				<div class="flex items-center gap-3 p-2 rounded-md bg-muted/50 animate-pulse">
					<div class="w-12 h-12 rounded-sm bg-muted/60 shrink-0"></div>
					<div class="flex flex-col gap-1 flex-1 min-w-0">
						<div class="h-3 w-3/4 rounded bg-muted/60"></div>
						<div class="h-2 w-1/2 rounded bg-muted/60"></div>
					</div>
				</div>
			{/each}
		</div>
	{:else if recentAlbums.length === 0}
		<p class="text-xs text-muted-foreground text-center py-8">No recently played albums yet.</p>
	{:else}
		<div class="app-music-grid grid gap-3 pr-4 pl-4 pb-4">
			{#each recentAlbums as album (album.uid)}
				<button
					onclick={() => setSelection(album.uid)}
					class="flex items-center gap-3 cursor-pointer p-2 rounded-md bg-muted/50 hover:bg-muted text-left">
					<ArtworkDisplay entity={album} size={48} />
					<div class="flex flex-col min-w-0">
						<span class="text-sm font-medium truncate">{album.title}</span>
						<span class="text-xs text-muted-foreground truncate">{album.album_artist ?? ""}</span>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
.app-music-grid {
	grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
}
</style>