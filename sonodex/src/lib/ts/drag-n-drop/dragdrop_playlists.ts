import { invoke } from "@tauri-apps/api/core";
import { library } from "$lib/ts/library.svelte";
import { setHoveredPlaylist, setDragActive, setDragPayload, endDrag } from "$lib/ts/app-states/state_drag.svelte";
import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
import { getDirectPlaylists } from "$lib/ts/app/playlistLibrary.svelte";
import { playlistOrder } from "$lib/ts/app/playlistOrderStore.svelte";
import type { Playlist } from "$lib/ts/util/types";
import type { PlaylistSortField } from "$lib/ts/app/playlistLibrary.svelte";

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

export async function doPlaylistReorder(
	draggedUid: string,
	targetIndex: number,
	side: "before" | "after",
	playlists: Playlist[],
	sortField: PlaylistSortField,
	sortDir: "asc" | "desc",
	targetFolderPath: string | null
) {
	const current = getDirectPlaylists(playlists, targetFolderPath, sortField, sortDir);
	const fromIndex = current.findIndex((p) => p.uid === draggedUid);
	if (fromIndex === -1) {
		await movePlaylists([draggedUid], targetFolderPath);
		const updated = getDirectPlaylists(playlists, targetFolderPath, sortField, sortDir);
		const newFrom = updated.findIndex((p) => p.uid === draggedUid);
		if (newFrom === -1) return;
		const insertAt = side === "before" ? targetIndex : targetIndex + 1;
		const uids = updated.map((p) => p.uid);
		uids.splice(newFrom, 1);
		const adjusted = newFrom < insertAt ? insertAt - 1 : insertAt;
		uids.splice(Math.max(0, Math.min(adjusted, uids.length)), 0, draggedUid);
		playlistOrder.setPlaylists(targetFolderPath, uids);
		return;
	}
	const insertAt = side === "before" ? targetIndex : targetIndex + 1;
	const uids = current.map((p) => p.uid);
	uids.splice(fromIndex, 1);
	const adjusted = fromIndex < insertAt ? insertAt - 1 : insertAt;
	uids.splice(Math.max(0, Math.min(adjusted, uids.length)), 0, draggedUid);
	playlistOrder.setPlaylists(targetFolderPath, uids);
}