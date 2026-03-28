import { library } from "$lib/ts/library.svelte";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { Track } from "../util/types";

type SelectionContext = "library" | "playlist";

let selectedUids = $state<Set<string>>(new Set());
let lastClickedUid = $state<string | null>(null);
let context = $state<SelectionContext>("library");
let contextUid = $state<string | null>(null);

export const trackSelection = {
	get selected() { return selectedUids; },
	get lastClicked() { return lastClickedUid; },
	get context() { return context; },
	get contextUid() { return contextUid; },
	get count() { return selectedUids.size; },
	isSelected(uid: string) { return selectedUids.has(uid); },
};

export function setTrackSelectionContext(ctx: SelectionContext, uid: string | null = null) {
	context = ctx;
	contextUid = uid;
	selectedUids = new Set();
	lastClickedUid = null;
}

export function clearTrackSelection() {
	selectedUids = new Set();
	lastClickedUid = null;
}

export function selectTrack(uid: string, orderedUids: string[], event: MouseEvent) {
	if (event.shiftKey && lastClickedUid) {
		const a = orderedUids.indexOf(lastClickedUid);
		const b = orderedUids.indexOf(uid);
		if (a !== -1 && b !== -1) {
			const [lo, hi] = a < b ? [a, b] : [b, a];
			const range = orderedUids.slice(lo, hi + 1);
			if (event.ctrlKey || event.metaKey) {
				selectedUids = new Set([...selectedUids, ...range]);
			} else {
				selectedUids = new Set(range);
			}
			return;
		}
	}

	if (event.ctrlKey || event.metaKey) {
		const next = new Set(selectedUids);
		if (next.has(uid)) {
			next.delete(uid);
		} else {
			next.add(uid);
		}
		selectedUids = next;
	} else {
		selectedUids = new Set([uid]);
	}

	lastClickedUid = uid;
}

export function getSelectedTracks() {
	return library.tracks.filter((t) => selectedUids.has(t.uid));
}

export function copySelectedToClipboard(orderedUids: string[]) {
	const tracks = orderedUids
		.filter((uid) => selectedUids.has(uid))
		.map((uid) => library.tracks.find((t) => t.uid === uid))
		.filter(Boolean);

	const trackNames = tracks.map((t) => (t!.title ?? "Unknown Title") + "; " + (t!.album_artist ?? "Unknown Artist")).join("\n");
	const uids = tracks.map((t) => t!.uid).join("\n");

	writeText(trackNames + "\n---\n" + uids).catch(() => {});
}

export function copySelectedNamesToClipboard(orderedUids: string[]) {
	const tracks = orderedUids
		.filter((uid) => selectedUids.has(uid))
		.map((uid) => library.tracks.find((t) => t.uid === uid))
		.filter(Boolean);

	const trackNames = tracks.map((t) => (t!.title ?? "Unknown Title") + "; " + (t!.album_artist ?? "Unknown Artist")).join("\n");

	writeText(trackNames).catch(() => {});
}

export function copySelectedUIDsToClipboard(orderedUids: string[]) {
	const tracks = orderedUids
		.filter((uid) => selectedUids.has(uid))
		.map((uid) => library.tracks.find((t) => t.uid === uid))
		.filter(Boolean);

	const uids = tracks.map((t) => t!.uid).join("\n");

	writeText(uids).catch(() => {});
}
export function copySelectedNameToClipboard(track: Track) {
	writeText(track.title ?? "Unknown Title" + "; " + (track.album_artist ?? "Unknown Artist")).catch(() => {});
}
