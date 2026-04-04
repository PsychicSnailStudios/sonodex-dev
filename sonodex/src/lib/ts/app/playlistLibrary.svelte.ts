import { folderSelection } from "$lib/ts/app/folderSelection.svelte";
import { playlistOrder, applyCustomOrder } from "$lib/ts/app/playlistOrderStore.svelte";
import { library } from "$lib/ts/library.svelte";
import type { Playlist } from "$lib/ts/util/types";

export type PlaylistSortField = "title" | "date_created" | "custom";

export function folderOf(p: Playlist): string | null {
	const f = (p as any).folder;
	return f && f !== "" ? f : null;
}

export function folderLabel(fullPath: string): string {
	return fullPath.split("/").at(-1) ?? fullPath;
}

function immediateChild(parentPath: string | null, folderPath: string): string | null {
	if (parentPath === null) return folderPath.split("/")[0];
	if (!folderPath.startsWith(parentPath + "/")) return null;
	return folderPath.slice(parentPath.length + 1).split("/")[0];
}

export function getChildFoldersRaw(
	playlists: Playlist[],
	parentPath: string | null
): string[] {
	const seen = new Set<string>();
	for (const p of playlists) {
		const pf = folderOf(p);
		if (!pf) continue;
		const child = immediateChild(parentPath, pf);
		if (child) seen.add(parentPath === null ? child : `${parentPath}/${child}`);
	}
	for (const known of folderSelection.knownFolders) {
		const child = immediateChild(parentPath, known);
		if (child) seen.add(parentPath === null ? child : `${parentPath}/${child}`);
	}
	return [...seen];
}

export function getSortedFolders(
	playlists: Playlist[],
	parentPath: string | null,
	sortField: PlaylistSortField,
	sortDir: "asc" | "desc"
): string[] {
	const raw = getChildFoldersRaw(playlists, parentPath);
	if (sortField === "custom") {
		return applyCustomOrder(raw, playlistOrder.getFolders(parentPath), (f) => f);
	}
	return [...raw].sort((a, b) => {
		const la = a.split("/").at(-1) ?? a;
		const lb = b.split("/").at(-1) ?? b;
		return sortDir === "asc" ? la.localeCompare(lb) : lb.localeCompare(la);
	});
}

export function getDirectPlaylists(
	playlists: Playlist[],
	folderPath: string | null,
	sortField: PlaylistSortField,
	sortDir: "asc" | "desc"
): Playlist[] {
	const direct = playlists.filter((p) => folderOf(p) === folderPath);
	if (sortField === "custom") {
		return applyCustomOrder(direct, playlistOrder.getPlaylists(folderPath), (p) => p.uid);
	}
	return [...direct].sort((a, b) => {
		if (sortField === "title") {
			return sortDir === "asc"
				? a.title.localeCompare(b.title)
				: b.title.localeCompare(a.title);
		}
		if (sortField === "date_created") {
			const ai = (a as any).id ?? 0;
			const bi = (b as any).id ?? 0;
			return sortDir === "asc" ? ai - bi : bi - ai;
		}
		return 0;
	});
}

export function getFolderArtworkUids(playlists: Playlist[], folderPath: string): string[] {
	return playlists
		.filter((p) => {
			const pf = folderOf(p);
			return pf === folderPath || (pf?.startsWith(folderPath + "/") ?? false);
		})
		.map((p) => p.uid)
		.slice(0, 4);
}
