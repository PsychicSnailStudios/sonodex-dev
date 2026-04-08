import { invoke } from "@tauri-apps/api/core";

export async function enrichTrack(uid: string): Promise<void> {
	await invoke("enrich_track", { uid });
}

export async function enrichAlbum(uid: string): Promise<void> {
	await invoke("enrich_album", { uid });
}

export async function enrichArtist(uid: string): Promise<void> {
	await invoke("enrich_artist", { uid });
}

export async function fetchLyrics(uid: string): Promise<void> {
	await invoke("fetch_track_lyrics", { uid });
}
