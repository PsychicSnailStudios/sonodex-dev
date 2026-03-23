import type { Playlist, Track } from "./types";
import { invoke } from "@tauri-apps/api/core";

export async function addTrackToPlaylist(playlist: Playlist, track: Track) {
	let current: { uid: string; name: string }[] = playlist.tracks ? JSON.parse(playlist.tracks) : []

	let already = current.some((t) => t.uid === track.uid)
	if (already) return

	let updated = [...current, { uid: track.uid!, name: track.title ?? "Unknown Title" }]

	await invoke("update_playlist_by_uid", {
		uid: String(playlist.uid),
		update: {
			tracks: JSON.stringify(updated),
		},
	})
}