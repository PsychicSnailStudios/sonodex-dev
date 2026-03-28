import { library } from "$lib/ts/library.svelte";
import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
import { invoke } from "@tauri-apps/api/core";

export type DragPayload = {
	type: "tracks";
	uids: string[];
	sourcePlaylistUid: string | null;
};

let active = $state(false);
let payload = $state<DragPayload | null>(null);
let hoveredPlaylistUid = $state<string | null>(null);
let hoveredFolderPath = $state<string | null>(null);

let _folderHoverTimer: ReturnType<typeof setTimeout> | null = null;
let _folderHoverKey: string | null = null;

let _breadcrumbHoverTimer: ReturnType<typeof setTimeout> | null = null;
let _breadcrumbHoverKey: string = "";

export const dragState = {
	get active() { return active; },
	get payload() { return payload; },
	get hoveredPlaylistUid() { return hoveredPlaylistUid; },
	get hoveredFolderPath() { return hoveredFolderPath; },
};

export function startDrag(p: DragPayload) {
	active = true;
	payload = p;
}

export function endDrag() {
	active = false;
	payload = null;
	hoveredPlaylistUid = null;
	hoveredFolderPath = null;
	_clearFolderTimer();
	_clearBreadcrumbTimer();
}

export function setHoveredPlaylist(uid: string | null) {
	hoveredPlaylistUid = uid;
}

function _clearFolderTimer() {
	if (_folderHoverTimer !== null) {
		clearTimeout(_folderHoverTimer);
		_folderHoverTimer = null;
	}
	_folderHoverKey = null;
}

function _clearBreadcrumbTimer() {
	if (_breadcrumbHoverTimer !== null) {
		clearTimeout(_breadcrumbHoverTimer);
		_breadcrumbHoverTimer = null;
	}
	_breadcrumbHoverKey = "";
}

export function onFolderDragOver(
	path: string,
	onNavigate: (path: string | null) => void
) {
	if (!active) return;
	hoveredFolderPath = path;

	if (_folderHoverKey === path) return;
	_clearFolderTimer();
	_folderHoverKey = path;
	_folderHoverTimer = setTimeout(() => {
		onNavigate(path);
		_folderHoverKey = null;
		_folderHoverTimer = null;
	}, 700);
}

export function onFolderDragExit(path: string) {
	if (hoveredFolderPath === path) hoveredFolderPath = null;
	if (_folderHoverKey === path) _clearFolderTimer();
}

export function onBreadcrumbDragOver(
	path: string | null,
	onNavigate: (path: string | null) => void
) {
	if (!active) return;
	const key = path ?? "__root__";

	if (_breadcrumbHoverKey === key) return;
	_clearBreadcrumbTimer();
	_breadcrumbHoverKey = key;
	_breadcrumbHoverTimer = setTimeout(() => {
		onNavigate(path);
		_breadcrumbHoverKey = "";
		_breadcrumbHoverTimer = null;
	}, 700);
}

export function onBreadcrumbDragExit() {
	_clearBreadcrumbTimer();
}

export async function dropOnPlaylist(e: DragEvent, playlistUid: string) {
	e.preventDefault();
	setHoveredPlaylist(null);
	const raw = e.dataTransfer?.getData("text/plain");
	if (!raw) return;
	const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
	if (!uids.every((u) => u.startsWith("p-"))) {
		await addTracksToPlaylist(playlistUid, uids);
	}
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