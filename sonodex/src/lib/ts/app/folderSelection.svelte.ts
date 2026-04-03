import { library } from "$lib/ts/library.svelte";

let currentPath = $state<string | null>(null);
let _knownFolders = $state<Set<string>>(new Set());
let _seeded = false;

function _seedFromLibrary() {
	if (_seeded) return;
	_seeded = true;
	for (const p of library.playlists ?? []) {
		const f = (p as any).folder;
		if (f && f !== "") _knownFolders.add(f);
	}
}

export const folderSelection = {
	get currentPath() { return currentPath; },
	get knownFolders(): Set<string> {
		_seedFromLibrary();
		return _knownFolders;
	},
};

export function registerFolder(path: string) {
	_seedFromLibrary();
	_knownFolders = new Set([..._knownFolders, path]);
}

export function removeFolder(path: string) {
	_seedFromLibrary();
	const next = new Set(_knownFolders);
	for (const k of next) {
		if (k === path || k.startsWith(path + "/")) next.delete(k);
	}
	_knownFolders = next;
}

export function navigateTo(path: string | null) {
	currentPath = path;
}

export function navigateInto(segment: string) {
	currentPath = currentPath ? `${currentPath}/${segment}` : segment;
}

export function breadcrumbs(): { label: string; path: string | null }[] {
	const crumbs: { label: string; path: string | null }[] = [
		{ label: "Playlists", path: null },
	];
	if (!currentPath) return crumbs;
	let acc = "";
	for (const part of currentPath.split("/")) {
		acc = acc ? `${acc}/${part}` : part;
		crumbs.push({ label: part, path: acc });
	}
	return crumbs;
}