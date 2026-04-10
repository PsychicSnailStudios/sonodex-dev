import { invoke } from "@tauri-apps/api/core";

export type AlbumEntry = {
	uid: string;
	name: string;
	track_number: number | null;
};

// ─── Internal helpers ─────────────────────────────────────────────────────────

function parseJsonArray(val: string | null | undefined): any[] {
	if (!val) return [];
	try {
		const parsed = JSON.parse(val);
		return Array.isArray(parsed) ? parsed : [];
	} catch {
		return [];
	}
}

// ─── Artist sync ──────────────────────────────────────────────────────────────

export async function syncArtists(artistNames: string[]): Promise<void> {
	if (artistNames.length === 0) return;

	const allArtists = await invoke<any[]>("get_artists");

	for (const name of artistNames) {
		const nameLower = name.toLowerCase();
		const exists = allArtists.some((a) => (a.name ?? "").toLowerCase() === nameLower);
		if (!exists) {
			await invoke("create_artist_entry", {
				artist: {
					uid: `ar-${crypto.randomUUID()}`,
					name,
					aka: null,
					about: null,
					tags: JSON.stringify([]),
					genres: JSON.stringify([]),
					websites: null,
					members: null,
					profile_art_blob: null,
					profile_art_path: null,
					banner_art_blob: null,
					banner_art_path: null,
				},
			});
		}
	}
}

/**
 * Delete artist records for any name in `removedNames` that is no longer
 * referenced by any track (artists, album_artist) or album (artists, album_artist).
 */
export async function pruneArtists(removedNames: string[]): Promise<void> {
	if (removedNames.length === 0) return;

	const [allArtists, allTracks, allAlbums] = await Promise.all([
		invoke<any[]>("get_artists"),
		invoke<any[]>("get_tracks"),
		invoke<any[]>("get_albums"),
	]);

	const usedNames = new Set<string>();
	for (const t of allTracks) {
		parseJsonArray(t.artists).forEach((n: string) => usedNames.add(n.toLowerCase()));
		if (t.album_artist) usedNames.add((t.album_artist as string).toLowerCase());
	}
	for (const a of allAlbums) {
		parseJsonArray(a.artists).forEach((n: string) => usedNames.add(n.toLowerCase()));
		if (a.album_artist) usedNames.add((a.album_artist as string).toLowerCase());
	}

	const nameToUid = new Map(allArtists.map((a) => [a.name.toLowerCase(), a.uid as string]));

	for (const name of removedNames) {
		const lower = name.toLowerCase();
		if (!usedNames.has(lower)) {
			const uid = nameToUid.get(lower);
			if (uid) await invoke("delete_artist_entry", { uid });
		}
	}
}

// ─── Album sync ───────────────────────────────────────────────────────────────

export async function syncAlbums(
	cleanedAlbums: AlbumEntry[],
	trackUid: string,
	trackTitle: string,
	trackAlbumArtist: string
): Promise<AlbumEntry[]> {
	const allAlbums = await invoke<any[]>("get_albums");
	const finalAlbumEntries: AlbumEntry[] = [];

	for (const entry of cleanedAlbums) {
		const nameLower = entry.name.toLowerCase();
		const artistLower = trackAlbumArtist.toLowerCase();

		const match = allAlbums.find((a) => {
			const titleMatch = (a.title ?? "").toLowerCase() === nameLower;
			const artistMatch = (a.album_artist ?? "").toLowerCase() === artistLower;
			return titleMatch && artistMatch;
		});

		if (match) {
			const existingTracks = parseJsonArray(match.tracks);
			const alreadyIn = existingTracks.some((t: any) => t.uid === trackUid);

			if (alreadyIn) {
				const updated = existingTracks.map((t: any) =>
					t.uid === trackUid ? { ...t, track_number: entry.track_number ?? null } : t
				);
				await invoke("update_album_entry", { uid: match.uid, update: { tracks: JSON.stringify(updated) } });
			} else {
				existingTracks.push({ uid: trackUid, name: trackTitle, track_number: entry.track_number ?? null });
				await invoke("update_album_entry", { uid: match.uid, update: { tracks: JSON.stringify(existingTracks) } });
			}

			finalAlbumEntries.push({ uid: match.uid, name: entry.name, track_number: entry.track_number });
		} else {
			const newAlbum = {
				uid: `a-${crypto.randomUUID()}`,
				title: entry.name,
				album_artist: trackAlbumArtist || null,
				tracks: JSON.stringify([{ uid: trackUid, name: trackTitle, track_number: entry.track_number ?? null }]),
				artists: null,
				format: null,
				rating: null,
				release_date: null,
				tags: JSON.stringify([]),
				genres: JSON.stringify([]),
				credits: null,
				label: null,
				artwork_blob: null,
				artwork_path: null,
			};

			await invoke("create_album_entry", { album: newAlbum });

			const refreshed = await invoke<any[]>("get_albums");
			const created = refreshed.find((a) =>
				(a.title ?? "").toLowerCase() === entry.name.toLowerCase() &&
				(a.album_artist ?? "").toLowerCase() === trackAlbumArtist.toLowerCase()
			);

			finalAlbumEntries.push({
				uid: created?.uid ?? newAlbum.uid,
				name: entry.name,
				track_number: entry.track_number,
			});
		}
	}

	return finalAlbumEntries;
}

/**
 * Remove this track from albums it was previously on.
 * If an album ends up with zero tracks after removal, delete the album record entirely.
 */
export async function removeTrackFromOldAlbums(
	trackUid: string,
	removedAlbumUids: string[]
): Promise<void> {
	for (const albumUid of removedAlbumUids) {
		const album = await invoke<any | null>("get_album", { uid: albumUid });
		if (!album) continue;

		const remaining = parseJsonArray(album.tracks).filter((t: any) => t.uid !== trackUid);

		if (remaining.length === 0) {
			await invoke("delete_album_entry", { uid: albumUid });
		} else {
			await invoke("update_album_entry", { uid: albumUid, update: { tracks: JSON.stringify(remaining) } });
		}
	}
}

// ─── Stub track creation ──────────────────────────────────────────────────────

export async function createStubTrack(
	title: string,
	artist: string,
	album: string,
	trackNumber: number | null,
	durationMs: number | null,
	year: string | null
): Promise<string> {
	const uid = `t-${crypto.randomUUID()}`;

	const albumArtist = artist;
	const albumEntry = album
		? JSON.stringify([{ uid: "", name: album, track_number: trackNumber ?? null }])
		: null;

	await invoke("add_track", {
		track: {
			uid,
			path: "",
			last_modified: 0,
			title,
			artists: JSON.stringify([artist]),
			album_artist: albumArtist,
			albums: albumEntry,
			genres: null,
			year,
			rating: null,
			tags: "[]",
			duration_ms: durationMs,
			bpm: null,
			key: null,
			credits: null,
			label: null,
			artwork_blob: null,
			artwork_path: null,
		},
	});

	if (album) {
		const albumEntries: AlbumEntry[] = [{ uid: "", name: album, track_number: trackNumber ?? null }];
		const synced = await syncAlbums(albumEntries, uid, title, albumArtist);
		await invoke("update_track_metadata", {
			uid,
			update: { albums: JSON.stringify(synced) },
		});
	}

	await syncArtists([artist]);

	return uid;
}
