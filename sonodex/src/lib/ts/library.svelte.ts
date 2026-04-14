import { invoke } from "@tauri-apps/api/core";
import type { Track, Album, Artist, Playlist, Lyrics } from "$lib/ts/util/types";
import { SortState } from "$lib/ts/app/sortConfig.svelte";
import { parseUidType } from "$lib/ts/util/helpers";
import { loadTags } from '$lib/ts/tagManager.svelte';

export const library = $state({
	tracks: [] as Track[],
	albums: [] as Album[],
	artists: [] as Artist[],
	playlists: [] as Playlist[],
	lyrics: [] as Lyrics[],
	loaded: false,
});

export async function loadLibrary() {
	library.tracks = await invoke("get_tracks");
	library.albums = await invoke("get_albums");
	library.artists = await invoke("get_artists");
	library.playlists = await invoke("get_playlists");
	await loadTags();
	library.loaded = true;
}

export async function reloadLibrary(type: "tracks" | "all" | "tags" | "albums" | "artists" | "playlists" | "lyrics") {
	switch (type) {
		case "tracks":
			library.tracks = await invoke("get_tracks");
			break;
		case "albums":
			library.albums = await invoke("get_albums");
			break;
		case "artists":
			library.artists = await invoke("get_artists");
			break;
		case "playlists":
			library.playlists = await invoke("get_playlists");
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
	let type = parseUidType(uid);

	switch (type) {
		case "track":
			let newTrack = await invoke("get_track", { uid });
			library.tracks.find(a => a.uid === uid) === newTrack;

			const updatedLyrics = await invoke<Lyrics | null>("get_track_lyrics", { uid });
			const existingIdx = library.lyrics.findIndex(l => l.track_uid === uid);
			if (updatedLyrics) {
				if (existingIdx >= 0) library.lyrics[existingIdx] = updatedLyrics;
				else library.lyrics.push(updatedLyrics);
			} else if (existingIdx >= 0) {
				library.lyrics.splice(existingIdx, 1);
			}
			break;
		case "album":
			let newAlbum = await invoke("get_album", { uid });
			library.albums.find(a => a.uid === uid) === newAlbum;
			break;
		case "artist":
			let newArtist = await invoke("get_artist", { uid });
			library.artists.find(a => a.uid === uid) === newArtist;
			break;
		case "playlist":
			let newPlaylist = await invoke("get_playlist", { uid });
			library.playlists.find(a => a.uid === uid) === newPlaylist;
			break;
	}
}

export async function getLyrics(trackUid: string): Promise<Lyrics | null> {
	return await invoke<Lyrics | null>("get_track_lyrics", { uid: trackUid });
}

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