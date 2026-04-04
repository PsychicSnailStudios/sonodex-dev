import { invoke } from "@tauri-apps/api/core";
import { library } from "$lib/ts/library.svelte";
import { setHoveredPlaylist, setDragActive, setDragPayload, endDrag } from "$lib/ts/app-states/state_drag.svelte";
import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";

export function startPlaylistDrag(e: DragEvent, uid: string) {
	if (e.dataTransfer) {
		e.dataTransfer.setData("text/plain", uid);
		e.dataTransfer.effectAllowed = "move";
	}
	setDragActive(true);
	setDragPayload({ type: "tracks", uids: [uid], sourcePlaylistUid: null });
}

export async function dropOnPlaylist(e: DragEvent, playlistUid: string) {
	e.preventDefault();
	setHoveredPlaylist(null);
	const raw = e.dataTransfer?.getData("text/plain");
	if (!raw) return;
	const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
	if (uids.every((u) => u.startsWith("p-"))) {
		endDrag();
		return;
	}
	await addTracksToPlaylist(playlistUid, uids);
	endDrag();
}

export async function movePlaylists(playlistUids: string[], targetFolder: string | null) {
	for (const uid of playlistUids) {
		await invoke("update_playlist_entry", {
			uid,
			update: { folder: targetFolder ?? "" },
		});
	}
	library.playlists = await invoke("get_playlists");
}
