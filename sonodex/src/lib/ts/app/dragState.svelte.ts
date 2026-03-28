export type DragPayload = {
	type: "tracks";
	uids: string[];
	sourcePlaylistUid: string | null;
};

let active = $state(false);
let payload = $state<DragPayload | null>(null);
let hoveredPlaylistUid = $state<string | null>(null);

export const dragState = {
	get active() { return active; },
	get payload() { return payload; },
	get hoveredPlaylistUid() { return hoveredPlaylistUid; },
};

export function startDrag(p: DragPayload) {
	active = true;
	payload = p;
}

export function endDrag() {
	active = false;
	payload = null;
	hoveredPlaylistUid = null;
}

export function setHoveredPlaylist(uid: string | null) {
	hoveredPlaylistUid = uid;
}
