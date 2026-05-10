import { invoke } from "@tauri-apps/api/core";
import type { AudioCatagories } from "$ts/util/types";

export const artworkCache = new Map<string, string | null>();
export const artworkInflight = new Map<string, Promise<string | null>>();

const commandMap: Record<string, string> = {
	track: "get_track_artwork",
	album: "get_album_artwork",
	artist: "get_artist_profile_art",
	playlist: "get_playlist_artwork",
};

export async function prefetchArtwork(uids: string[], type: AudioCatagories): Promise<void> {
	const command = commandMap[type];
	if (!command) return;

	const uncached = uids.filter(uid => !artworkCache.has(`${type}:${uid}`));
	if (uncached.length === 0) return;

	await Promise.all(
		uncached.map(async (uid) => {
			const cacheKey = `${type}:${uid}`;

			if (artworkInflight.has(cacheKey)) return artworkInflight.get(cacheKey);

			const promise = invoke<number[] | null>(command, { uid }).then((bytes) => {
				let url: string | null = null;
				if (bytes) {
					const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
					url = URL.createObjectURL(blob);
				}
				artworkCache.set(cacheKey, url);
				artworkInflight.delete(cacheKey);
				return url;
			}).catch(() => {
				artworkCache.set(cacheKey, null);
				artworkInflight.delete(cacheKey);
				return null;
			});

			artworkInflight.set(cacheKey, promise);
			return promise;
		})
	);
}