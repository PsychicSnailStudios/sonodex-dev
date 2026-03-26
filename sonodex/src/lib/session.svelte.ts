import { invoke } from "@tauri-apps/api/core";

export let selection = $state({
	uid: "" as string,
	type: "none" as "track" | "album" | "artist" | "playlist" | "none",
});

export function setSelection(uid: string, type: "track" | "album" | "artist" | "playlist" | "none") {
	selection.uid = uid;
	selection.type = type;
}

export const scanState = $state({
	loading: false,
	progress: 0,
	total: 0,
	status: "",
	enriching: false,
	enrichDone: 0,
	enrichTotal: 0,
	enrichErrors: 0,
})