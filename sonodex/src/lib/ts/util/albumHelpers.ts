import type { Track } from "$ts/util/types";

export function getTracksFirstAlbumName(track: Track): string {
	try {
		const albums: { name: string }[] = JSON.parse(track.albums as unknown as string ?? "[]");
		return albums[0]?.name ?? "";
	} catch {
		return "";
	}
}