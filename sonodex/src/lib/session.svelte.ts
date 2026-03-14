import { invoke } from "@tauri-apps/api/core";

export let selection = $state({
	id: 0 as number,
	type: "none" as "track" | "album" | "artist" | "playlist" | "none",
});

export function setSelection(id: number, type: "track" | "album" | "artist" | "playlist" | "none") {
	selection.id = id;
	selection.type = type;
}

export const playing = $state({
	id: 0,
	type: "" as "track" | "album" | "artist" | "playlist" | "",
});

export function setPlaying(id: number) {
	playing.id = id;
}