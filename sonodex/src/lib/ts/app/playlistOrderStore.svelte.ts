const STORAGE_KEY = "sonodex:playlist-order";

type OrderStore = {
	playlists: Record<string, string[]>;
	folders: Record<string, string[]>;
};

function load(): OrderStore {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) return JSON.parse(raw);
	} catch {}
	return { playlists: {}, folders: {} };
}

function save(store: OrderStore) {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
	} catch {}
}

let _store = $state<OrderStore>(load());

export const playlistOrder = {
	getPlaylists(folderPath: string | null): string[] {
		return _store.playlists[folderPath ?? "__root__"] ?? [];
	},
	setPlaylists(folderPath: string | null, uids: string[]) {
		_store = {
			..._store,
			playlists: { ..._store.playlists, [folderPath ?? "__root__"]: uids },
		};
		save(_store);
	},
	getFolders(parentPath: string | null): string[] {
		return _store.folders[parentPath ?? "__root__"] ?? [];
	},
	setFolders(parentPath: string | null, paths: string[]) {
		_store = {
			..._store,
			folders: { ..._store.folders, [parentPath ?? "__root__"]: paths },
		};
		save(_store);
	},
};

export function applyCustomOrder<T>(
	items: T[],
	savedOrder: string[],
	getKey: (item: T) => string
): T[] {
	if (savedOrder.length === 0) return items;
	const indexMap = new Map(savedOrder.map((k, i) => [k, i]));
	const known: T[] = [];
	const unknown: T[] = [];
	for (const item of items) {
		if (indexMap.has(getKey(item))) known.push(item);
		else unknown.push(item);
	}
	known.sort((a, b) => (indexMap.get(getKey(a)) ?? 0) - (indexMap.get(getKey(b)) ?? 0));
	return [...known, ...unknown];
}

export function reorder<T>(items: T[], fromIndex: number, toIndex: number): T[] {
	const result = [...items];
	const [moved] = result.splice(fromIndex, 1);
	result.splice(toIndex, 0, moved);
	return result;
}
