import { invoke } from "@tauri-apps/api/core";
import { library, getArtistUidFromName } from "$lib/ts/library.svelte";
import type { Track } from "$lib/ts/util/types";

let activeScrobbleUid: string | null = null;
let didSeek = false;
let didPause = false;

function resolveArtistUid(track: Track): string {
	try {
		const artists: string[] = JSON.parse(track.artists ?? "[]");
		const name = artists[0];
		if (!name) return "";
		return getArtistUidFromName(name) ?? "";
	} catch {
		return "";
	}
}

export async function scrobbleStart(track: Track): Promise<void> {
	didSeek = false;
	didPause = false;

	if (activeScrobbleUid) {
		await scrobbleEnd(0);
	}

	try {
		const artistUid = resolveArtistUid(track);
		activeScrobbleUid = await invoke<string>("log_scrobble", {
			trackUid: track.uid,
			artistUid,
		});
	} catch (e) {
		console.error("scrobbleStart failed", e);
		activeScrobbleUid = null;
	}
}

export async function scrobbleEnd(durationPlayed: number): Promise<void> {
	if (!activeScrobbleUid) return;

	const uid = activeScrobbleUid;
	activeScrobbleUid = null;

	try {
		await invoke("update_scrobble", {
			uid,
			durationPlayed: Math.round(durationPlayed * 1000),
			didSeek,
			didPause,
		});
	} catch (e) {
		console.error("scrobbleEnd failed", e);
	}
}

export function scrobbleMarkSeeked(): void {
	didSeek = true;
}

export function scrobbleMarkPaused(): void {
	didPause = true;
}