import { invoke } from "@tauri-apps/api/core";
import { SortState } from "$ts/util/sortConfig.svelte";
import { parseUidType } from "$ts/util/parsers";
import { loadTags } from '$ts/store/tagManager.svelte';
import { prefetchArtwork } from "$ts/library/artworkLoader";

import type { Track, Album, Artist, Playlist, Lyrics, Library } from "$ts/util/types";

// ─── Main library state ───────────────────────────────────────────────────────

export const library = $state({
	tracks: [] as Track[],
	albums: [] as Album[],
	artists: [] as Artist[],
	playlists: [] as Playlist[],
	lyrics: [] as Lyrics[],
	loaded: false,

	trackMap: new Map<string, Track>(),
	albumMap: new Map<string, Album>(),
	artistMap: new Map<string, Artist>(),
	playlistMap: new Map<string, Playlist>(),
	lyricsMap: new Map<string, Lyrics>(),
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

export function getLibraryByUid(uid: string): Library | null {
	return libraryStore.libraries.find(l => l.uid === uid) ?? null;
}

// ─── Map builders ─────────────────────────────────────────────────────────────

function buildAllMaps() {
	buildTrackMap();
	buildAlbumMap();
	buildArtistMap();
	buildPlaylistMap();
	buildLyricsMap();
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
			buildTrackMap();
			break;
		case "albums":
			library.albums = await invoke("get_albums");
			buildAlbumMap();
			prefetchArtwork(library.albums.map(a => a.uid), "album");
			break;
		case "artists":
			library.artists = await invoke("get_artists");
			buildArtistMap();
			prefetchArtwork(library.artists.map(a => a.uid), "artist");
			break;
		case "playlists":
			library.playlists = await invoke("get_playlists");
			buildPlaylistMap();
			prefetchArtwork(library.playlists.map(p => p.uid), "playlist");
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
			const trackIdx = library.tracks.findIndex(t => t.uid === uid);
			if (trackIdx >= 0 && newTrack) {
				library.tracks[trackIdx] = newTrack;
				library.trackMap.set(uid, newTrack);
			}

			const updatedLyrics = await invoke<Lyrics | null>("get_track_lyrics", { uid });
			const existingIdx = library.lyrics.findIndex(l => l.track_uid === uid);
			if (updatedLyrics) {
				if (existingIdx >= 0) library.lyrics[existingIdx] = updatedLyrics;
				else library.lyrics.push(updatedLyrics);
				library.lyricsMap.set(uid, updatedLyrics);
			} else if (existingIdx >= 0) {
				library.lyrics.splice(existingIdx, 1);
				library.lyricsMap.delete(uid);
			}
			break;
		}
		case "album": {
			const newAlbum = await invoke<Album | null>("get_album", { uid });
			const albumIdx = library.albums.findIndex(a => a.uid === uid);
			if (albumIdx >= 0 && newAlbum) {
				library.albums[albumIdx] = newAlbum;
				library.albumMap.set(uid, newAlbum);
			}
			break;
		}
		case "artist": {
			const newArtist = await invoke<Artist | null>("get_artist", { uid });
			const artistIdx = library.artists.findIndex(a => a.uid === uid);
			if (artistIdx >= 0 && newArtist) {
				library.artists[artistIdx] = newArtist;
				library.artistMap.set(uid, newArtist);
			}
			break;
		}
		case "playlist": {
			const newPlaylist = await invoke<Playlist | null>("get_playlist", { uid });
			const playlistIdx = library.playlists.findIndex(p => p.uid === uid);
			if (playlistIdx >= 0 && newPlaylist) {
				library.playlists[playlistIdx] = newPlaylist;
				library.playlistMap.set(uid, newPlaylist);
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

export function getTrackArrayFromUID(uid: string, sort?: SortState): Track[] {
    const type = parseUidType(uid);
    const isAlbum = type === "album";

    if (!isAlbum && type !== "playlist") return [];

    const container = isAlbum ? library.albumMap.get(uid) : library.playlistMap.get(uid);
    if (!container?.tracks) return [];

    const entries: any[] = JSON.parse(container.tracks);
    const len = entries.length;
    if (len === 0) return [];

    const uids = new Array<string>(len);
    const metaMap = new Map<string, number>();

    for (let i = 0; i < len; i++) {
        const entry = entries[i];
        const trackUid = entry.uid;
        uids[i] = trackUid;
        metaMap.set(trackUid, isAlbum ? entry.track_number : entry.order);
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

    const sortData = tracks.map((track) => {
        let value: string | number;

        switch (field) {
            case "number":
                value = (trackNumberMap ? trackNumberMap.get(track.uid) : orderMap?.get(track.uid)) ?? Infinity;
                break;
            case "title":
                value = track.title ?? "";
                break;
            case "artist":
                value = track.album_artist!.name.toString() ?? "";
                break;
            case "album":
                value = track.albums![0].name ?? "";
                break;
            case "year":
                value = track.year ?? "";
                break;
            case "duration":
                value = track.duration_ms ?? 0;
                break;
            case "rating":
                value = track.rating ?? 0;
                break;
            case "label":
                value = track.label ?? "";
                break;
            default:
                value = (orderMap ? orderMap.get(track.uid) : trackNumberMap?.get(track.uid)) ?? Infinity;
        }
        return { track, value };
    });

    sortData.sort((a, b) => {
        let cmp: number;
        if (typeof a.value === "number" && typeof b.value === "number") {
            cmp = a.value - b.value;
        } else {
            cmp = String(a.value).localeCompare(String(b.value));
        }
        return cmp * dir;
    });

    return sortData.map(d => d.track);
}