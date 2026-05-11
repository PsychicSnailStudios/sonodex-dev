import type { Track, TrackAlbumEntry, PlaylistTrackEntry, AudioCatagories, UserOptions, Artist, ArtistEntry, AlbumEntry } from "$ts/util/types";

export function parseUidType(uid: string): AudioCatagories {
	if (uid.startsWith("t-")) return "track";
	if (uid.startsWith("ar-")) return "artist";
	if (uid.startsWith("a-")) return "album";
	if (uid.startsWith("p-")) return "playlist";
	return "unknown";
}

export function parseArtistsToString(artists: ArtistEntry[] | null): string {
	if (!artists || artists.length === 0) return "Unknown Artist";

	return artists
		.map(artist => artist.name || "Unknown Artist")
		.join(", ");
}

export function parseAlbumsToString(albums: TrackAlbumEntry[] | null): string {
	if (!albums || albums.length === 0) return "Unknown Albums";

	return albums
		.map(album => album.name || "Unknown Albums")
		.join(", ");
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