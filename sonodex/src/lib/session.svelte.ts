import { invoke } from "@tauri-apps/api/core";

export let selection = $state({
	id: 0 as number,
	type: "none" as "track" | "album" | "artist" | "playlist" | "none",
});

export function setSelection(id: number, type: "track" | "album" | "artist" | "playlist" | "none") {
	selection.id = id;
	selection.type = type;
}