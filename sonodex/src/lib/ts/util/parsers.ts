import type { Track, TrackAlbumEntry, PlaylistTrackEntry, AudioCatagories, UserOptions } from "$ts/util/types";

export function parseUidType(uid: string): AudioCatagories {
	if (uid.startsWith("t-")) return "track";
	if (uid.startsWith("ar-")) return "artist";
	if (uid.startsWith("a-")) return "album";
	if (uid.startsWith("p-")) return "playlist";
	return "unknown";
}

export function parseArtists(artists: string | null): string {
	if (!artists) return "Unknown Artist";
	try {
		const parsed = JSON.parse(artists);
		return Array.isArray(parsed) ? parsed.join(", ") : "Unknown Artist";
	} catch {
		return "Unknown Artist";
	}
}

export function parseAlbumEntries(albums: string | null): TrackAlbumEntry[] {
	if (!albums) return [];
	try {
		const parsed = JSON.parse(albums);
		return Array.isArray(parsed) ? parsed : [];
	} catch {
		return [];
	}
}

export function parseAlbum(albums: string): string {
	const entries = parseAlbumEntries(albums);
	return entries.length > 0 ? entries[0].name : "—";
}

export function parseTrackNumber(albums: string): number | null {
	if (!albums) return null;
	const entries = parseAlbumEntries(albums);
	if (entries.length > 0) {
		const n = entries[0].track_number;
		return typeof n === "number" ? n : null;
	}
	return null;
}

export function parseTags(tags: string | null): string[] {
	if (!tags) return [];
	try {
		const parsed = JSON.parse(tags);
		if (!Array.isArray(parsed)) return [];
		return parsed;
	} catch {
		return [];
	}
}

export function parseUserOptions(raw: string | null | undefined): UserOptions {
	const defaults: UserOptions = { linkedShuffle: null, trimStart: null, trimEnd: null };
	if (!raw) return defaults;
	try {
		return { ...defaults, ...JSON.parse(raw) };
	} catch {
		return defaults;
	}
}