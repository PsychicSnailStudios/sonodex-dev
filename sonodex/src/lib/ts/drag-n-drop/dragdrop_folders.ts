import { invoke } from "@tauri-apps/api/core";
import { library } from "$lib/ts/library.svelte";
import { dragState, setHoveredFolder, setDragActive, setDragPayload, endDrag } from "$lib/ts/app-states/state_drag.svelte";
import { isDraggingFolderType } from "$lib/ts/drag-n-drop/dragdrop";
import { registerFolder, removeFolder } from "$lib/ts/app/folderSelection.svelte";
import { addTracksToPlaylist } from "$lib/ts/audio/playlistManager.svelte";
import { movePlaylists } from "$lib/ts/drag-n-drop/dragdrop_playlists";

let _folderHoverTimer: ReturnType<typeof setTimeout> | null = null;
let _folderHoverKey: string | null = null;

let _breadcrumbHoverTimer: ReturnType<typeof setTimeout> | null = null;
let _breadcrumbHoverKey: string = "";

export function resetFolderTimers() {
	_clearFolderTimer();
	_clearBreadcrumbTimer();
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
	if (!dragState.active) return;
	setHoveredFolder(path);

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
	if (dragState.hoveredFolderPath === path) setHoveredFolder(null);
	if (_folderHoverKey === path) _clearFolderTimer();
}

export function onBreadcrumbDragOver(
	path: string | null,
	onNavigate: (path: string | null) => void
) {
	if (!dragState.active) return;
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

export function startFolderDrag(e: DragEvent, path: string) {
	if (e.dataTransfer) {
		e.dataTransfer.setData("text/plain", path);
		e.dataTransfer.setData("application/x-sonodex-folder", "1");
		e.dataTransfer.effectAllowed = "move";
	}
	setDragActive(true);
	setDragPayload({ type: "tracks", uids: [path], sourcePlaylistUid: null });
}

export async function nestFolder(draggedPath: string, targetPath: string) {
	if (draggedPath === targetPath) return;
	if (targetPath.startsWith(draggedPath + "/")) return;
	const folderName = draggedPath.split("/").at(-1)!;
	const newPath = `${targetPath}/${folderName}`;
	await invoke("rename_playlist_folder", { oldPath: draggedPath, newPath });
	removeFolder(draggedPath);
	registerFolder(newPath);
	library.playlists = await invoke("get_playlists");
}

export async function moveFolderToRoot(draggedPath: string) {
	const folderName = draggedPath.split("/").at(-1)!;
	if (draggedPath === folderName) return;
	await invoke("rename_playlist_folder", { oldPath: draggedPath, newPath: folderName });
	removeFolder(draggedPath);
	registerFolder(folderName);
	library.playlists = await invoke("get_playlists");
}

export async function dropOnFolder(
	e: DragEvent,
	targetPath: string,
	getDirectPlaylists: (folder: string) => { uid: string }[]
) {
	e.preventDefault();
	e.stopPropagation();
	onFolderDragExit(targetPath);
	const raw = e.dataTransfer?.getData("text/plain");
	if (!raw) { endDrag(); return; }

	if (isDraggingFolderType(e)) {
		await nestFolder(raw.trim(), targetPath);
	} else {
		const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
		if (uids.every((u) => u.startsWith("p-"))) {
			await movePlaylists(uids, targetPath);
		} else {
			const first = getDirectPlaylists(targetPath)[0];
			if (first) await addTracksToPlaylist(first.uid, uids);
		}
	}
	endDrag();
}

export async function dropOnRoot(e: DragEvent) {
	e.preventDefault();
	const raw = e.dataTransfer?.getData("text/plain");
	if (!raw) { endDrag(); return; }

	if (isDraggingFolderType(e)) {
		await moveFolderToRoot(raw.trim());
	} else {
		const uids = raw.split(",").map((u) => u.trim()).filter(Boolean);
		if (uids.every((u) => u.startsWith("p-"))) {
			await movePlaylists(uids, null);
		}
	}
	endDrag();
}
