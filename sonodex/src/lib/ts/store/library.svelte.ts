import { invoke } from "@tauri-apps/api/core";
import { SortState } from "$ts/util/sortConfig.svelte";
import { parseUidType } from "$ts/util/parsers";
import { loadTags } from '$ts/store/tagManager.svelte';
import { prefetchArtwork } from "$ts/library/artworkLoader";
import { warmupFuseIndexes } from "$ts/store/fuseStore.svelte";

import type { Track, Album, Artist, Playlist, Lyrics, Library } from "$ts/util/types";

// ─── Main library state ───────────────────────────────────────────────────────

// Large collections use $state.raw to avoid Svelte deep-proxying thousands of objects.
// Primitive flags use plain $state so components can react to loaded/etc changes.
let _tracks = $state.raw<Track[]>([]);
let _albums = $state.raw<Album[]>([]);
let _artists = $state.raw<Artist[]>([]);
let _playlists = $state.raw<Playlist[]>([]);
let _lyrics = $state.raw<Lyrics[]>([]);
let _loaded = $state(false);
let _trackMap = $state.raw(new Map<string, Track>());
let _albumMap = $state.raw(new Map<string, Album>());
let _artistMap = $state.raw(new Map<string, Artist>());
let _playlistMap = $state.raw(new Map<string, Playlist>());
let _lyricsMap = $state.raw(new Map<string, Lyrics>());
let _artistAlbumIndex = $state.raw(new Map<string, Set<string>>());

export const library = {
	get tracks() { return _tracks; },
	set tracks(v) { _tracks = v; },
	get albums() { return _albums; },
	set albums(v) { _albums = v; },
	get artists() { return _artists; },
	set artists(v) { _artists = v; },
	get playlists() { return _playlists; },
	set playlists(v) { _playlists = v; },
	get lyrics() { return _lyrics; },
	set lyrics(v) { _lyrics = v; },
	get loaded() { return _loaded; },
	set loaded(v) { _loaded = v; },
	get trackMap() { return _trackMap; },
	set trackMap(v) { _trackMap = v; },
	get albumMap() { return _albumMap; },
	set albumMap(v) { _albumMap = v; },
	get artistMap() { return _artistMap; },
	set artistMap(v) { _artistMap = v; },
	get playlistMap() { return _playlistMap; },
	set playlistMap(v) { _playlistMap = v; },
	get lyricsMap() { return _lyricsMap; },
	set lyricsMap(v) { _lyricsMap = v; },
	get artistAlbumIndex() { return _artistAlbumIndex; },
	set artistAlbumIndex(v) { _artistAlbumIndex = v; },
};

// ─── Federated library registry state ────────────────────────────────────────

export const libraryStore = $state({
	libraries: [] as Library[],
	loaded: false,
});

export async function loadLibraryRegistry(): Promise<void> {
	libraryStore.libraries = await invoke<Library[]>("get_libraries");
	libraryStore.loaded = true;
}

export function getLibraryByUid(uid: string): Library | null {
	return libraryStore.libraries.find(l => l.uid === uid) ?? null;
}

function buildAllMaps() {
	buildTrackMap();
	buildAlbumMap();
	buildArtistMap();
	buildPlaylistMap();
	buildLyricsMap();
	buildArtistAlbumIndex();
}

function buildTrackMap() {
	library.trackMap = new Map(library.tracks.map(t => [t.uid, t]));
}

function buildAlbumMap() {
	library.albumMap = new Map(library.albums.map(a => [a.uid, a]));
}

function buildArtistMap() {
	library.artistMap = new Map(library.artists.map(a => [a.uid, a]));
}

function buildPlaylistMap() {
	library.playlistMap = new Map(library.playlists.map(p => [p.uid, p]));
}

function buildLyricsMap() {
	library.lyricsMap = new Map(library.lyrics.map(l => [l.track_uid, l]));
}

function buildArtistAlbumIndex() {
	const index = new Map<string, Set<string>>();

	// Index by album_artist uid on albums directly
	for (const album of library.albums) {
		if (!album.album_artist) continue;
		const key = album.album_artist.uid.toString();
		let set = index.get(key);
		if (!set) { set = new Set(); index.set(key, set); }
		set.add(album.uid);
	}

	// Also index featured appearances via tracks
	for (const track of library.tracks) {
		if (!track.artists || !track.albums) continue;
		for (const artist of track.artists) {
			const key = artist.uid.toString();
			let set = index.get(key);
			if (!set) { set = new Set(); index.set(key, set); }
			for (const a of track.albums) set.add(a.uid);
		}
	}

	library.artistAlbumIndex = index;
}

// ─── Load ─────────────────────────────────────────────────────────────────────

export async function loadLibrary() {
	library.tracks = await invoke("get_tracks");
	library.albums = await invoke("get_albums");
	library.artists = await invoke("get_artists");
	library.playlists = await invoke("get_playlists");
	library.lyrics = await invoke("get_all_lyrics");
	await loadTags();

	buildAllMaps();
	library.loaded = true;

	setTimeout(() => {
		prefetchArtwork(library.albums.map(a => a.uid), "album");
		prefetchArtwork(library.artists.map(a => a.uid), "artist");
		prefetchArtwork(library.playlists.map(p => p.uid), "playlist");
		warmupFuseIndexes();
	}, 0);

	// Keep library registry in sync
	if (!libraryStore.loaded) {
		await loadLibraryRegistry();
	}
}

export async function reloadLibrary(type: "tracks" | "all" | "tags" | "albums" | "artists" | "playlists" | "lyrics") {
	switch (type) {
		case "tracks":
			library.tracks = await invoke("get_tracks");
			buildTrackMap();
			buildArtistAlbumIndex();
			break;
		case "albums":
			library.albums = await invoke("get_albums");
			buildAlbumMap();
			buildArtistAlbumIndex();
			setTimeout(() => prefetchArtwork(library.albums.map(a => a.uid), "album"), 0);
			break;
		case "artists":
			library.artists = await invoke("get_artists");
			buildArtistMap();
			setTimeout(() => prefetchArtwork(library.artists.map(a => a.uid), "artist"), 0);
			break;
		case "playlists":
			library.playlists = await invoke("get_playlists");
			buildPlaylistMap();
			setTimeout(() => prefetchArtwork(library.playlists.map(p => p.uid), "playlist"), 0);
			break;
		case "lyrics":
			library.lyrics = await invoke("get_all_lyrics");
			buildLyricsMap();
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
			if (newTrack) {
				const trackIdx = library.tracks.findIndex(t => t.uid === uid);
				if (trackIdx >= 0) {
					const newTracks = [...library.tracks];
					newTracks[trackIdx] = newTrack;
					library.tracks = newTracks;
				}
				const newMap = new Map(library.trackMap);
				newMap.set(uid, newTrack);
				library.trackMap = newMap;
			}

			const updatedLyrics = await invoke<Lyrics | null>("get_track_lyrics", { uid });
			const existingIdx = library.lyrics.findIndex(l => l.track_uid === uid);
			const newLyrics = [...library.lyrics];
			const newLyricsMap = new Map(library.lyricsMap);
			if (updatedLyrics) {
				if (existingIdx >= 0) newLyrics[existingIdx] = updatedLyrics;
				else newLyrics.push(updatedLyrics);
				newLyricsMap.set(uid, updatedLyrics);
			} else if (existingIdx >= 0) {
				newLyrics.splice(existingIdx, 1);
				newLyricsMap.delete(uid);
			}
			library.lyrics = newLyrics;
			library.lyricsMap = newLyricsMap;
			break;
		}
		case "album": {
			const newAlbum = await invoke<Album | null>("get_album", { uid });
			if (newAlbum) {
				const albumIdx = library.albums.findIndex(a => a.uid === uid);
				if (albumIdx >= 0) {
					const newAlbums = [...library.albums];
					newAlbums[albumIdx] = newAlbum;
					library.albums = newAlbums;
				}
				const newMap = new Map(library.albumMap);
				newMap.set(uid, newAlbum);
				library.albumMap = newMap;
				buildArtistAlbumIndex();
			}
			break;
		}
		case "artist": {
			const newArtist = await invoke<Artist | null>("get_artist", { uid });
			if (newArtist) {
				const artistIdx = library.artists.findIndex(a => a.uid === uid);
				if (artistIdx >= 0) {
					const newArtists = [...library.artists];
					newArtists[artistIdx] = newArtist;
					library.artists = newArtists;
				}
				const newMap = new Map(library.artistMap);
				newMap.set(uid, newArtist);
				library.artistMap = newMap;
			}
			break;
		}
		case "playlist": {
			const newPlaylist = await invoke<Playlist | null>("get_playlist", { uid });
			if (newPlaylist) {
				const playlistIdx = library.playlists.findIndex(p => p.uid === uid);
				if (playlistIdx >= 0) {
					const newPlaylists = [...library.playlists];
					newPlaylists[playlistIdx] = newPlaylist;
					library.playlists = newPlaylists;
				}
				const newMap = new Map(library.playlistMap);
				newMap.set(uid, newPlaylist);
				library.playlistMap = newMap;
			}
			break;
		}
	}
}

export async function getLyrics(trackUid: string): Promise<Lyrics | null> {
	return await invoke<Lyrics | null>("get_track_lyrics", { uid: trackUid });
}

// ─── Lookups ──────────────────────────────────────────────────────────────────

export function getTrack(trackUid: string): Track | undefined {
	return library.trackMap.get(trackUid);
}

export function getAlbum(albumUid: string): Album | undefined {
	return library.albumMap.get(albumUid);
}

export function getArtist(artistUid: string): Artist | undefined {
	return library.artistMap.get(artistUid);
}

export function getPlaylist(playlistUid: string): Playlist | undefined {
	return library.playlistMap.get(playlistUid);
}

export function getLyricsForTrack(trackUid: string): Lyrics | undefined {
	return library.lyricsMap.get(trackUid);
}

export function getArtistAlbums(artistUid: string, akaUids?: string[]): Album[] {
	const allUids = new Set<string>();
	const artistAlbumSet = library.artistAlbumIndex.get(artistUid);
	if (artistAlbumSet) for (const uid of artistAlbumSet) allUids.add(uid);
	if (akaUids) {
		for (const aka of akaUids) {
			const akaSet = library.artistAlbumIndex.get(aka);
			if (akaSet) for (const uid of akaSet) allUids.add(uid);
		}
	}
	const result: Album[] = [];
	for (const uid of allUids) {
		const album = library.albumMap.get(uid);
		if (album) result.push(album);
	}
	return result;
}

const parsedTracksCache = new Map<string, { raw: string; entries: Array<{ uid: string; track_number?: number; order?: number }> }>();

export function getTrackArrayFromUID(uid: string, sort?: SortState): Track[] {
    const type = parseUidType(uid);
    const isAlbum = type === "album";

    if (!isAlbum && type !== "playlist") return [];

    const container = isAlbum ? library.albumMap.get(uid) : library.playlistMap.get(uid);
    if (!container?.tracks) return [];

    const raw = container.tracks;
    let cached = parsedTracksCache.get(uid);
    if (!cached || cached.raw !== raw) {
        cached = { raw, entries: JSON.parse(raw) };
        parsedTracksCache.set(uid, cached);
    }
    const entries = cached.entries;
    const len = entries.length;
    if (len === 0) return [];

    const uids = new Array<string>(len);
    const metaMap = new Map<string, number>();

    for (let i = 0; i < len; i++) {
        const entry = entries[i];
        const trackUid = entry.uid;
        uids[i] = trackUid;
        metaMap.set(trackUid, isAlbum ? (entry.track_number ?? Infinity) : (entry.order ?? Infinity));
    }

    const matched = getTrackArray(uids);
    const activeSort = sort ?? new SortState("number", "asc");

    return sortTracks(
        matched,
        activeSort,
        isAlbum ? metaMap : undefined,
        !isAlbum ? metaMap : undefined
    );
}

export function getTrackArray(trackUids: string[]): Track[] {
    const len = trackUids.length;
    const tracks: Track[] = [];
    
    for (let i = 0; i < len; i++) {
        const track = library.trackMap.get(trackUids[i]);
        if (track) tracks.push(track);
    }
    return tracks;
}

function sortTracks(
    tracks: Track[],
    sort: SortState,
    trackNumberMap?: Map<string, number | null>,
    orderMap?: Map<string, number>
): Track[] {
    const dir = sort.direction === "desc" ? -1 : 1;
    const field = sort.field;

    function getValue(track: Track): string | number {
        switch (field) {
            case "number":
                return (trackNumberMap ? trackNumberMap.get(track.uid) : orderMap?.get(track.uid)) ?? Infinity;
            case "title":
                return track.title ?? "";
            case "artist":
                return track.album_artist?.name.toString() ?? "";
            case "album":
                return track.albums?.[0]?.name ?? "";
            case "year":
                return track.year ?? "";
            case "duration":
                return track.duration_ms ?? 0;
            case "rating":
                return track.rating ?? 0;
            case "label":
                return track.label ?? "";
            default:
                return (orderMap ? orderMap.get(track.uid) : trackNumberMap?.get(track.uid)) ?? Infinity;
        }
    }

    const indices = Array.from({ length: tracks.length }, (_, i) => i);
    indices.sort((ai, bi) => {
        const a = getValue(tracks[ai]);
        const b = getValue(tracks[bi]);
        let cmp: number;
        if (typeof a === "number" && typeof b === "number") {
            cmp = a - b;
        } else {
            cmp = String(a).localeCompare(String(b));
        }
        return cmp * dir;
    });
    return indices.map(i => tracks[i]);
}