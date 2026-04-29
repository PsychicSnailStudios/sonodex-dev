import { SortState } from "$lib/ts/app/sortConfig.svelte";
import { createColumnState } from "$lib/ts/app/columnConfig.svelte";
import { clamp } from "$lib/ts/util/helpers";
import type { SortField, SortDirection } from "$lib/ts/app/sortConfig.svelte";
import type { ColumnKey } from "$lib/ts/app/columnConfig.svelte";
import { profileState } from "$lib/ts/profiles.svelte";

type Selection = {
	uid: string;
	type: "track" | "album" | "artist" | "playlist" | "none";
}

export let activeView = $state({ "id": "home"});
export let currentArtistTab = $state({ "id": "home"});
export let currentTrackTab = $state({ "id": "lyrics"});
export let offlineMode = $state({ "offline": false});
let viewIndex = $state(0);
let viewHistory = $state([] as Selection[]);

export let selection = $state({ "uid": "", "type": "none"} as Selection);

export const scanState = $state({
	loading: false,
	progress: 0,
	total: 0,
	status: "",
	enriching: false,
	enrichDone: 0,
	enrichTotal: 0,
	enrichErrors: 0,
});

function sessionKey(profileUid: string) {
	return `sonodex:session:${profileUid}`;
}

function viewKey(profileUid: string, key: string) {
	return `sonodex:view:${profileUid}:${key}`;
}

// SAVE/LOAD
export function loadSessionState(profileUid: string) {
	try {
		const raw = localStorage.getItem(sessionKey(profileUid));
		if (!raw) return;
		const saved = JSON.parse(raw);
		setSelection(saved.selection.uid, saved.selection.type);
		setView(saved.activeView.id);
	} catch {}
}

export function saveSessionState(profileUid: string) {
	try {
		localStorage.setItem(sessionKey(profileUid), JSON.stringify({
			selection: selection,
			activeView: activeView
		}));
	} catch {}
}

export function createPersistedViewState(
	key: string,
	defaults: {
		sortField?: SortField | null;
		sortDir?: SortDirection;
		compact?: boolean;
		colPreset?: string;
	} = {}
) {
	const profileUid = profileState.active?.uid ?? "default";
	const storageKey = viewKey(profileUid, key);

	let saved: any = null;
	try {
		const raw = localStorage.getItem(storageKey);
		if (raw) saved = JSON.parse(raw);
	} catch {}

	const sort = new SortState(
		saved?.sortField ?? defaults.sortField ?? null,
		saved?.sortDir ?? defaults.sortDir ?? "asc"
	);

	const cols = createColumnState(saved?.colPreset ?? defaults.colPreset ?? "default");

	if (saved?.cols) {
		for (const col of Object.keys(saved.cols) as ColumnKey[]) {
			cols.visible[col] = saved.cols[col];
		}
	}

	let compact = $state(saved?.compact ?? defaults.compact ?? false);

	$effect(() => {
		JSON.stringify(cols.visible);
		sort.field;
		sort.direction;
		compact;
		save();
	});

	function save() {
		try {
			localStorage.setItem(storageKey, JSON.stringify({
				sortField: sort.field,
				sortDir: sort.direction,
				cols: cols.visible,
				compact,
			}));
		} catch {}
	}

	return {
		sort,
		cols,
		get compact() { return compact; },
		set compact(v: boolean) { compact = v; },
		save,
	};
}

// SELECTION
export function setSelection(uid: string, type: "track" | "album" | "artist" | "playlist" | "none") {
	selection.uid = uid;
	selection.type = type;

	if (viewHistory.length - 1 !== viewIndex) {
		viewHistory = viewHistory.slice(0, viewIndex + 1);
	}

	viewHistory = [...viewHistory, { uid, type }];
	viewIndex = viewHistory.length - 1;
}
export function setView(view: string) {
	activeView.id = view;
}
export function clearSelection() {
	selection.uid = "";
	selection.type = "none";
	viewHistory = [];
	viewIndex = 0;
}
export function moveSelection(index: number) {
	viewIndex += index;
	viewIndex = clamp(viewIndex, 0, viewHistory.length - 1);

	selection.uid = viewHistory[viewIndex].uid;
	selection.type = viewHistory[viewIndex].type;
}

export function canGoBack() {
	return viewIndex > 0;
}
export function canGoForward() {
	return viewIndex < viewHistory.length - 1;
}