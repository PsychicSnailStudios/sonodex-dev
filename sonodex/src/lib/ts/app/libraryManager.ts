import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { library, loadLibrary } from "$lib/ts/library.svelte";
import type { Track, Album, Artist, DuplicateGroup } from "$lib/ts/util/types";
import { showWarning } from "$lib/ts/app/dialogManager.svelte";

// ─── Track Actions ───────────────────────────────────────────────────────────

export async function removeTrackFromLibrary(uid: string): Promise<void> {

	const confirmed = await showWarning({ title: "Delete Track?", description: "This action cannot be undone",});
		if (!confirmed) return;

	await invoke("remove_track_from_library", { uid });
	await loadLibrary();
}

export async function enrichTrack(uid: string): Promise<void> {
	await invoke("enrich_track", { uid });
}

export async function replaceTrackPath(uid: string): Promise<void> {
	const selected = await open({
		multiple: false,
		filters: [
			{
				name: "Audio",
				extensions: ["mp3", "flac", "m4a", "aac", "wav", "aiff", "ogg"],
			},
		],
	});
	if (!selected) return;
	const newPath = typeof selected === "string" ? selected : selected[0];
	await invoke("replace_track_path", { uid, newPath });
	await loadLibrary();
}

export async function openTrackInExplorer(path: string): Promise<void> {
	await invoke("open_in_explorer", { path });
}

// ─── Album Actions ───────────────────────────────────────────────────────────

export async function removeAlbum(uid: string): Promise<void> {
	await invoke("delete_album_entry", { uid });
	await loadLibrary();
}

export async function enrichAlbumTracks(uid: string): Promise<void> {
	const album = library.albums.find((a) => a.uid === uid);
	if (!album) return;

	let tracks: Array<{ uid: string }> = [];
	try {
		tracks = album.tracks ? JSON.parse(album.tracks as string) : [];
	} catch {
		tracks = [];
	}

	for (const t of tracks) {
		await invoke("enrich_track", { uid: t.uid });
	}
}

// ─── Artist Actions ──────────────────────────────────────────────────────────

export async function removeArtist(uid: string): Promise<void> {
	await invoke("delete_artist_entry", { uid });
	await loadLibrary();
}

// ─── Duplicate Actions ───────────────────────────────────────────────────────

export async function getDuplicates(): Promise<DuplicateGroup[]> {
	return await invoke<DuplicateGroup[]>("get_duplicates");
}

export async function keepTrack(
	keepUid: string,
	group: DuplicateGroup
): Promise<void> {
	const toRemove = group.tracks.filter((t) => t.uid !== keepUid);
	for (const track of toRemove) {
		await invoke("remove_track_from_library", { uid: track.uid });
	}
	await loadLibrary();
}

export async function deleteTrackFile(
	uid: string,
	path: string
): Promise<void> {

	const confirmed = await showWarning({ title: "Delete Track?", description: "This action cannot be undone",});
		if (!confirmed) return;
		
	await invoke("delete_track_file", { uid, path });
	await loadLibrary();
}

export async function mergeKeepFirst(group: DuplicateGroup): Promise<void> {
	if (group.tracks.length < 2) return;
	await keepTrack(group.tracks[0].uid, group);
}