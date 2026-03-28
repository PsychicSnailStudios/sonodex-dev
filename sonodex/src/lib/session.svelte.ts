import { invoke } from "@tauri-apps/api/core";
import { clamp } from "$lib/ts/util/helpers";

type Selection = {
	uid: string;
	type: "track" | "album" | "artist" | "playlist" | "none";
}

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

export function setSelection(uid: string, type: "track" | "album" | "artist" | "playlist" | "none") {
	selection.uid = uid;
	selection.type = type;

	if (viewHistory.length - 1 !== viewIndex) {
		viewHistory = viewHistory.slice(0, viewIndex + 1);
	}

	viewHistory = [...viewHistory, { uid, type }];
	viewIndex = viewHistory.length - 1;
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

	selection.uid = viewHistory[viewIndex].uid
	selection.type = viewHistory[viewIndex].type;
}

export function canGoBack() {
	return viewIndex > 0;
}
export function canGoForward() {
	return viewIndex < viewHistory.length - 1;
}