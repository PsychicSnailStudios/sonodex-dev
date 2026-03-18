import type { Track } from "$lib/types";

export function parseArtists(artists: string | null): string {
		if (!artists) return "Unknown Artist";
		try {
			const parsed = JSON.parse(artists);
			return Array.isArray(parsed) ? parsed.join(", ") : "Unknown Artist";
		} catch {
			return "Unknown Artist";
		}
	}

	export function parseAlbum(albums: string | null): string {
		if (!albums) return "—";
		try {
			const parsed = JSON.parse(albums);
			return Array.isArray(parsed) && parsed.length > 0 ? parsed[0].name : "—";
		} catch {
			return "—";
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