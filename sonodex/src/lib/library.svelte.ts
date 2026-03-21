import { invoke } from "@tauri-apps/api/core";
import type { Track, Album, Artist, Playlist } from "$lib/types";

export const library = $state({
	tracks: [] as Track[],
	albums: [] as Album[],
	artists: [] as Artist[],
	playlists: [] as Playlist[],
	loaded: false,
});

export async function loadLibrary() {
	library.tracks = await invoke("get_tracks");
	library.albums = await invoke("get_albums");
	library.artists = await invoke("get_artists");
	library.playlists = await invoke("get_playlists");
	library.loaded = true;
}

export function getArtistUidFromName(name: string): string {
	const artist = library.artists.find((a) => a.name === name);
	return artist?.uid ?? "";
}

export function getAlbumUidFromName(name: string): string {
	const album = library.albums.find((a) => a.title === name);
	return album?.uid ?? "";
}