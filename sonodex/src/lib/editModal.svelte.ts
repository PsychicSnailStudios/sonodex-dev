export type EditModalTarget =
	| { type: "track"; id: number }
	| { type: "album"; id: number }
	| { type: "artist"; id: number }
	| { type: "playlist"; id: number };

export let editModal = $state({
	open: false,
	target: null as EditModalTarget | null,
	onsave: null as (() => void) | null,
});

export function openEditModal(target: EditModalTarget, onsave?: () => void) {
	editModal.target = target;
	editModal.open = true;
}

export function closeEditModal() {
	editModal.open = false;
	editModal.target = null;
	editModal.onsave = null;
}