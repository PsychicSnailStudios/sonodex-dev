import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { toast } from "svelte-sonner";
import { library } from "$ts/store/library.svelte";
import { offlineMode } from "$ts/store/session.svelte";
import type { Track } from "$ts/util/types";

// ─── Path classification ──────────────────────────────────────────────────────

export function isRemotePath(path: string): boolean {
	return (
		path.startsWith("http://") ||
		path.startsWith("https://") ||
		path.startsWith("\\\\") ||
		path.startsWith("//")
	);
}

export function isLocalPath(path: string): boolean {
	return path.length > 0 && !isRemotePath(path);
}

export function pathToSrc(path: string): string {
	if (path.startsWith("http://") || path.startsWith("https://")) {
		return path;
	}
	return convertFileSrc(path);
}

// ─── Track data helpers ───────────────────────────────────────────────────────

export interface TrackData {
	bitrate?: number | null;
	format?: string | null;
	is_ghost?: boolean;
}

export interface RemoteData {
	bitrate?: number | null;
	format?: string | null;
	is_ghost?: boolean;
}

export function parseTrackData(track: Track): TrackData {
	try {
		if (track.track_data) return JSON.parse(track.track_data as string);
	} catch {}
	return {};
}

export function parseRemoteData(track: Track): RemoteData {
	try {
		if (track.remote_data) return JSON.parse(track.remote_data as string);
	} catch {}
	return {};
}

export function isGhostTrack(track: Track): boolean {
	const td = parseTrackData(track);
	if (td.is_ghost === true) return true;
	if (track.remote_path && track.remote_path.length > 0) return false;
	if (!track.path || track.path === "" || track.path === track.uid) return true;
	return false;
}

export function localBitrate(track: Track): number {
	return parseTrackData(track).bitrate ?? track.bitrate ?? 0;
}

export function remoteBitrate(track: Track): number {
	return parseRemoteData(track).bitrate ?? 0;
}

export async function markGhost(uid: string) {
	try {
		const track = library.trackMap.get(uid);
		if (!track) return;
		const td = parseTrackData(track);
		td.is_ghost = true;
		await invoke("update_track_metadata", {
			uid,
			update: { track_data: JSON.stringify(td) },
		});
	} catch {}
}

export async function clearGhost(uid: string) {
	try {
		const track = library.trackMap.get(uid);
		if (!track) return;
		const td = parseTrackData(track);
		td.is_ghost = false;
		await invoke("update_track_metadata", {
			uid,
			update: { track_data: JSON.stringify(td) },
		});
	} catch {}
}

// ─── Offline mode ─────────────────────────────────────────────────────────────

let offlineModeToastShown = false;

export function triggerOfflineMode() {
	offlineMode.offline = true;
	if (offlineModeToastShown) return;
	offlineModeToastShown = true;

	toast.warning("No internet connection — going into offline mode.", {
		duration: 10000,
		action: {
			label: "Stay online",
			onClick: () => {
				offlineMode.offline = false;
				offlineModeToastShown = false;
			},
		},
		onDismiss: () => {
			offlineModeToastShown = false;
		},
	});
}

export async function checkOnline(): Promise<boolean> {
	try {
		const res = await fetch("https://www.gstatic.com/generate_204", {
			method: "HEAD",
			cache: "no-store",
			signal: AbortSignal.timeout(3000),
		});
		return res.ok || res.status === 204;
	} catch {
		return false;
	}
}

// ─── Settings cache ───────────────────────────────────────────────────────────

let cachedAutoOffline: boolean | null = null;

export async function getAutoOfflineSetting(): Promise<boolean> {
	if (cachedAutoOffline !== null) return cachedAutoOffline;
	try {
		const settings = await invoke<Array<{ key: string; value: string }>>("get_settings");
		cachedAutoOffline = settings.find(s => s.key === "offline_mode_auto")?.value === "true";
		return cachedAutoOffline;
	} catch {
		return false;
	}
}
