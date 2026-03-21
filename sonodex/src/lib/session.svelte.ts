import { invoke } from "@tauri-apps/api/core";

export let selection = $state({
	uid: "" as string,
	type: "none" as "track" | "album" | "artist" | "playlist" | "none",
});

export function setSelection(uid: string, type: "track" | "album" | "artist" | "playlist" | "none") {
	selection.uid = uid;
	selection.type = type;
}