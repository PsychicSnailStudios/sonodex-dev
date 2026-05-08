import { invoke } from "@tauri-apps/api/core";
import type { Track, Album, Artist, Playlist, Lyrics, Library } from "$lib/ts/util/types";
import { SortState } from "$lib/ts/app/sortConfig.svelte";
import { parseUidType } from "$lib/ts/util/helpers";
import { loadTags } from '$lib/ts/tagManager.svelte';
import { prefetchArtwork } from "$lib/ts/app/artworkPrefetch";

// ─── Main library state ───────────────────────────────────────────────────────

export const library = $state({
	tracks: [] as Track[],
	albums: [] as Album[],
	artists: [] as Artist[],
	playlists: [] as Playlist[],
	lyrics: [] as Lyrics[],
	loaded: false,
});

// ─── Federated library registry state ────────────────────────────────────────

export const libraryStore = $state({
	libraries: [] as Library[],
	loaded: false,
});

export async function loadLibraryRegistry(): Promise<void> {
	libraryStore.libraries = await invoke<Library[]>("get_libraries");
	libraryStore.loaded = true;
}

// Returns the Library record for a given uid, or null if not found.
export function getLibraryByUid(uid: string): Library | null {
	return libraryStore.libraries.find(l => l.uid === uid) ?? null;
}

// Returns true if the given source_lib_uid belongs to the default library.
export function isDefaultLibrary(sourceLibUid: string | null): boolean {
	if (!sourceLibUid) return true;
	const lib = getLibraryByUid(sourceLibUid);
	return lib?.is_default ?? false;
}

// Returns true if the entity is editable — either it came from the local
// default library, or it has a local override, or the source lib has write permission.
export function isEntityWritable(
	sourceLibUid: string | null,
	isLocalOverride: boolean | null
): boolean {
	if (!sourceLibUid) return true;
	if (isLocalOverride) return true;
	const lib = getLibraryByUid(sourceLibUid);
	return lib?.has_write_permission ?? false;
}

// ─── Load ─────────────────────────────────────────────────────────────────────

export async function loadLibrary() {
	await invoke("rebuild_merged_cmd");
	library.tracks = await invoke("get_tracks");
	library.albums = await invoke("get_albums");
	library.artists = await invoke("get_artists");
	library.playlists = await invoke("get_playlists");
	await loadTags();

	library.loaded = true;

	prefetchArtwork(library.albums.map(a => a.uid), "album");
	prefetchArtwork(library.artists.map(a => a.uid), "artist");
	prefetchArtwork(library.playlists.map(p => p.uid), "playlist");

	// Keep library registry in sync
	if (!libraryStore.loaded) {
		await loadLibraryRegistry();
	}
}

export async function reloadLibrary(type: "tracks" | "all" | "tags" | "albums" | "artists" | "playlists" | "lyrics") {
	switch (type) {
		case "tracks":
			library.tracks = await invoke("get_tracks");
			break;
		case "albums":
			library.albums = await invoke("get_albums");
			prefetchArtwork(library.albums.map(a => a.uid), "album");
			break;
		case "artists":
			library.artists = await invoke("get_artists");
			prefetchArtwork(library.artists.map(a => a.uid), "artist");
			break;
		case "playlists":
			library.playlists = await invoke("get_playlists");
			prefetchArtwork(library.playlists.map(p => p.uid), "playlist");
			break;
		case "lyrics":
			const allLyrics = await Promise.all(
				library.tracks.map(t => invoke<Lyrics | null>("get_track_lyrics", { uid: t.uid }))
			);
			library.lyrics = allLyrics.filter((l): l is Lyrics => l !== null);
			break;
		case "tags":
			await loadTags();
			break;
		case "all":
			await loadLibrary();
			break;
	}
}

export async function reloadSingle(uid: string) {
	const type = parseUidType(uid);

	switch (type) {
		case "track": {
			const newTrack = await invoke<Track | null>("get_track", { uid });
			const trackIdx = library.tracks.findIndex(t => t.uid === uid);
			if (trackIdx >= 0 && newTrack) library.tracks[trackIdx] = newTrack;

			const updatedLyrics = await invoke<Lyrics | null>("get_track_lyrics", { uid });
			const existingIdx = library.lyrics.findIndex(l => l.track_uid === uid);
			if (updatedLyrics) {
				if (existingIdx >= 0) library.lyrics[existingIdx] = updatedLyrics;
				else library.lyrics.push(updatedLyrics);
			} else if (existingIdx >= 0) {
				library.lyrics.splice(existingIdx, 1);
			}
			break;
		}
		case "album": {
			const newAlbum = await invoke<Album | null>("get_album", { uid });
			const albumIdx = library.albums.findIndex(a => a.uid === uid);
			if (albumIdx >= 0 && newAlbum) library.albums[albumIdx] = newAlbum;
			break;
		}
		case "artist": {
			const newArtist = await invoke<Artist | null>("get_artist", { uid });
			const artistIdx = library.artists.findIndex(a => a.uid === uid);
			if (artistIdx >= 0 && newArtist) library.artists[artistIdx] = newArtist;
			break;
		}
		case "playlist": {
			const newPlaylist = await invoke<Playlist | null>("get_playlist", { uid });
			const playlistIdx = library.playlists.findIndex(p => p.uid === uid);
			if (playlistIdx >= 0 && newPlaylist) library.playlists[playlistIdx] = newPlaylist;
			break;
		}
	}
}

export async function getLyrics(trackUid: string): Promise<Lyrics | null> {
	return await invoke<Lyrics | null>("get_track_lyrics", { uid: trackUid });
}

// ─── Lookups ──────────────────────────────────────────────────────────────────

export function getArtistUidFromName(name: string): string {
	const lower = name.toLowerCase();
	const byName = library.artists.find((a) => a.name.toLowerCase() === lower);
	if (byName) return byName.uid;

	const byAka = library.artists.find((a) => {
		try {
			const akas: string[] = JSON.parse(a.aka ?? "[]");
			return akas.some((aka) => aka.toLowerCase() === lower);
		} catch {
			return false;
		}
	});
	return byAka?.uid ?? "";
}

export function getAlbumUidFromName(name: string): string {
	const album = library.albums.find((a) => a.title === name);
	return album?.uid ?? "";
}

export function getAlbumTracks(albumUid: string, sort?: SortState): Track[] {
	const album = library.albums.find(a => a.uid === albumUid);
	if (!album) return [];

	const entries: { uid: string; track_number: number | null }[] = JSON.parse(album.tracks ?? "[]");
	const uidSet = new Set(entries.map(e => e.uid));
	const trackNumberMap = new Map(entries.map(e => [e.uid, e.track_number]));

	const matched = library.tracks.filter(t => uidSet.has(t.uid));

	return sortTracks(matched, sort ?? new SortState("number", "asc"), trackNumberMap);
}

export function getPlaylistTracks(playlistUid: string, sort?: SortState): Track[] {
	const playlist = library.playlists.find(p => p.uid === playlistUid);
	if (!playlist) return [];

	const entries: { uid: string; order: number }[] = JSON.parse(playlist.tracks ?? "[]");
	const orderMap = new Map(entries.map(e => [e.uid, e.order]));
	const uidSet = new Set(entries.map(e => e.uid));

	const matched = library.tracks.filter(t => uidSet.has(t.uid));

	return sortTracks(matched, sort ?? new SortState("number", "asc"), undefined, orderMap);
}

// ─── Sort ─────────────────────────────────────────────────────────────────────

function sortTracks(
	tracks: Track[],
	sort: SortState,
	trackNumberMap?: Map<string, number | null>,
	orderMap?: Map<string, number>
): Track[] {
	const copy = [...tracks];
	const dir = sort.direction === "desc" ? -1 : 1;

	copy.sort((a, b) => {
		let cmp = 0;

		switch (sort.field) {
			case "number":
				if (trackNumberMap) {
					const na = trackNumberMap.get(a.uid) ?? Infinity;
					const nb = trackNumberMap.get(b.uid) ?? Infinity;
					cmp = na - nb;
				} else if (orderMap) {
					const oa = orderMap.get(a.uid) ?? Infinity;
					const ob = orderMap.get(b.uid) ?? Infinity;
					cmp = oa - ob;
				}
				break;
			case "title":
				cmp = (a.title ?? "").localeCompare(b.title ?? "");
				break;
			case "artist":
				cmp = (a.album_artist ?? "").localeCompare(b.album_artist ?? "");
				break;
			case "album": {
				const aa = JSON.parse(a.albums ?? "[]")[0]?.name ?? "";
				const ab = JSON.parse(b.albums ?? "[]")[0]?.name ?? "";
				cmp = aa.localeCompare(ab);
				break;
			}
			case "year":
				cmp = (a.year ?? "").localeCompare(b.year ?? "");
				break;
			case "duration":
				cmp = (a.duration_ms ?? 0) - (b.duration_ms ?? 0);
				break;
			case "rating":
				cmp = (a.rating ?? 0) - (b.rating ?? 0);
				break;
			case "label":
				cmp = (a.label ?? "").localeCompare(b.label ?? "");
				break;
			default:
				if (orderMap) {
					const oa = orderMap.get(a.uid) ?? Infinity;
					const ob = orderMap.get(b.uid) ?? Infinity;
					cmp = oa - ob;
				} else if (trackNumberMap) {
					const na = trackNumberMap.get(a.uid) ?? Infinity;
					const nb = trackNumberMap.get(b.uid) ?? Infinity;
					cmp = na - nb;
				}
				break;
		}

		return cmp * dir;
	});

	return copy;
}