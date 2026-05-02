import { invoke } from "@tauri-apps/api/core";
import { library, getArtistUidFromName } from "$lib/ts/library.svelte";
import type { Track } from "$lib/ts/util/types";
import { player } from "$lib/ts/audio/audioManager.svelte";
import { offlineMode } from "$lib/ts/app-states/state_session.svelte";

let activeScrobbleUid: string | null = null;
let activeTrackUid: string | null = null;
let activeTrackStartTime: number = 0;
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

function firstArtistName(track: Track): string {
	try {
		const artists: string[] = JSON.parse(track.artists ?? "[]");
		return artists[0] ?? "";
	} catch {
		return "";
	}
}

function firstAlbumName(track: Track): string {
	try {
		const albums: { name: string }[] = JSON.parse(track.albums as unknown as string ?? "[]");
		return albums[0]?.name ?? "";
	} catch {
		return "";
	}
}

export async function scrobbleStart(
	track: Track,
	reason: string = "trackdone",
	playingLocal: boolean = true,
): Promise<void> {
	didSeek = false;
	didPause = false;

	if (activeScrobbleUid) {
		await scrobbleEnd(0, "trackdone", false);
	}

	activeTrackUid = track.uid;
	activeTrackStartTime = Math.floor(Date.now() / 1000);

	try {
		const artistUid = resolveArtistUid(track);
		activeScrobbleUid = await invoke<string>("log_scrobble", {
			trackUid: track.uid,
			artistUid,
			reasonStart: reason,
			shuffle: player.shuffleType > 0,
			offline: offlineMode.offline,
			playingLocal,
			trackName: track.title ?? null,
			trackArtist: firstArtistName(track) || null,
			trackAlbum: firstAlbumName(track) || null,
		});
	} catch (e) {
		console.error("scrobbleStart failed", e);
		activeScrobbleUid = null;
	}

	try {
		await invoke("update_now_playing", { uid: track.uid });
	} catch (e) {
		console.error("update_now_playing failed", e);
	}
}

export async function scrobbleEnd(
	durationPlayed: number,
	reason: string = "trackdone",
	skipped: boolean = false,
): Promise<void> {
	const uid = activeScrobbleUid;
	const trackUid = activeTrackUid;

	activeScrobbleUid = null;
	activeTrackUid = null;

	if (!uid) return;

	try {
		await invoke("update_scrobble", {
			uid,
			durationPlayed: Math.round(durationPlayed * 1000),
			didSeek,
			didPause,
			reasonEnd: reason,
			skipped,
		});
	} catch (e) {
		console.error("scrobbleEnd failed", e);
	}

	if (trackUid && durationPlayed > 30) {
		try {
			await invoke("scrobble_track", { uid: trackUid });
		} catch (e) {
			console.error("scrobble_track failed", e);
		}
	}
}

export function scrobbleMarkSeeked(): void {
	didSeek = true;
}

export function scrobbleMarkPaused(): void {
	didPause = true;
}