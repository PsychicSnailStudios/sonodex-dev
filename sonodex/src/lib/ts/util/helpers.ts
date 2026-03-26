import type { Track, TrackAlbumEntry, PlaylistTrackEntry } from "$lib/ts/util/types";

export type UidType = "track" | "album" | "artist" | "playlist" | "unknown";

export function parseUidType(uid: string): UidType {
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

export function parseAlbum(albums: string | null): string {
	const entries = parseAlbumEntries(albums);
	return entries.length > 0 ? entries[0].name : "—";
}

export function parseTrackNumber(albums: string | null | undefined): number | null {
	if (!albums) return null;
	const entries = parseAlbumEntries(albums);
	if (entries.length > 0) {
		const n = entries[0].track_number;
		return typeof n === "number" ? n : null;
	}
	return null;
}

export function parsePlaylistTracks(tracks: string | null): PlaylistTrackEntry[] {
	if (!tracks) return [];
	try {
		const parsed = JSON.parse(tracks);
		if (!Array.isArray(parsed)) return [];
		return parsed.slice().sort((a, b) => a.order - b.order);
	} catch {
		return [];
	}
}

export function formatRating(rating: number | null): string {
	if (rating === null) return "—";
	return rating.toFixed(1);
}

export function formatDuration(ms: number | null): string {
	if (ms === null) return "—";
	const totalSeconds = Math.floor(ms / 1000);
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function totalDuration(tracks: Track[]): string {
	const total = tracks.reduce((sum, t) => sum + (t.duration_ms ?? 0), 0);
	return formatDuration(total);
}

export async function getArtworkColor(bytes: number[], opacity = 1): Promise<string> {
	return new Promise((resolve) => {
		const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" })
		const url = URL.createObjectURL(blob)
		const img = new Image()
		img.crossOrigin = "anonymous"
		img.src = url

		img.onload = () => {
			const canvas = document.createElement("canvas")
			canvas.width = 10
			canvas.height = 10
			const ctx = canvas.getContext("2d")!
			ctx.drawImage(img, 0, 0, 10, 10)

			const data = ctx.getImageData(0, 0, 10, 10).data
			let r = 0, g = 0, b = 0, count = 0

			for (let i = 0; i < data.length; i += 4) {
				r += data[i]
				g += data[i + 1]
				b += data[i + 2]
				count++
			}

			r = Math.floor(r / count)
			g = Math.floor(g / count)
			b = Math.floor(b / count)

			URL.revokeObjectURL(url)
			resolve(`rgba(${r}, ${g}, ${b}, ${opacity})`)
		}

		img.onerror = () => {
			URL.revokeObjectURL(url)
			resolve(`rgba(30, 30, 30, ${opacity})`)
		}
	})
}

export function clamp(num: number, min: number, max: number): number {
  return num <= min ? min : num >= max ? max : num;
}