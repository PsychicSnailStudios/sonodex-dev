import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { flushSync } from "svelte";
import { toast } from "svelte-sonner";
import { library } from "$lib/ts/library.svelte";
import type { Track, AudioCatagories } from "$lib/ts/util/types";
import { parseTrackNumber, parseUidType } from "../util/helpers";
import { eq, EQ_BANDS } from "$lib/ts/app/eqStore.svelte";
import { scrobbleStart, scrobbleEnd, scrobbleMarkPaused, scrobbleMarkSeeked } from "$lib/ts/audio/scrobbler.svelte";
import { profileState } from "$lib/ts/profiles.svelte";
import { offlineMode } from "$lib/ts/app-states/state_session.svelte";

let audio: HTMLAudioElement | null = null;

let audioCtx: AudioContext | null = null;
let sourceNode: MediaElementAudioSourceNode | null = null;
let filterNodes: BiquadFilterNode[] = [];
let gainNode: GainNode | null = null;
let cachedAutoOffline: boolean | null = null;

let queuedTracks = $state<Track[]>([]);
let queueIndex = $state(-1);
let lastQueuedSet: Track[] = [];

let playedTracks = $state<Track[]>([]);

export const currentlyPlaying = $state({
	uid: "" as string,
	track: null as Track | null,
});

export const player = $state({
	track: null as Track | null,
	isPlaying: false,
	loopType: 0,
	shuffleType: 0,
	currentTime: 0,
	duration: 0,
	volume: 1,
	muted: false,
});

function playerKey() { return `sonodex:player:${profileState.active?.uid ?? "default"}`; }
function queueKey() { return `sonodex:queue:${profileState.active?.uid ?? "default"}`; }

// ─── Path classification ──────────────────────────────────────────────────────

function isRemotePath(path: string): boolean {
	return (
		path.startsWith("http://") ||
		path.startsWith("https://") ||
		path.startsWith("\\\\") ||
      path.startsWith("//")
	);
}

function isLocalPath(path: string): boolean {
	return path.length > 0 && !isRemotePath(path);
}

function pathToSrc(path: string): string {
	if (path.startsWith("http://") || path.startsWith("https://")) {
		return path;
	}
	return convertFileSrc(path);
}

// ─── Track data helpers ───────────────────────────────────────────────────────

interface TrackData {
	bitrate?: number | null;
	format?: string | null;
	is_ghost?: boolean;
}

interface RemoteData {
	bitrate?: number | null;
	format?: string | null;
	is_ghost?: boolean;
}

function parseTrackData(track: Track): TrackData {
	try {
		if (track.track_data) return JSON.parse(track.track_data as string);
	} catch {}
	return {};
}

function parseRemoteData(track: Track): RemoteData {
	try {
		if (track.remote_data) return JSON.parse(track.remote_data as string);
	} catch {}
	return {};
}

function isGhostTrack(track: Track): boolean {
	const td = parseTrackData(track);
	if (td.is_ghost === true) return true;
	if (track.remote_path && track.remote_path.length > 0) return false;
	if (!track.path || track.path === "" || track.path === track.uid) return true;
	return false;
}

function localBitrate(track: Track): number {
	return parseTrackData(track).bitrate ?? track.bitrate ?? 0;
}

function remoteBitrate(track: Track): number {
	return parseRemoteData(track).bitrate ?? 0;
}

async function markGhost(uid: string) {
	try {
		const track = library.tracks.find(t => t.uid === uid);
		if (!track) return;
		const td = parseTrackData(track);
		td.is_ghost = true;
		await invoke("update_track_metadata", {
			uid,
			update: { track_data: JSON.stringify(td) },
		});
	} catch {}
}

async function clearGhost(uid: string) {
	try {
		const track = library.tracks.find(t => t.uid === uid);
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

function triggerOfflineMode() {
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

async function checkOnline(): Promise<boolean> {
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

// ─── Audio graph ──────────────────────────────────────────────────────────────

function buildAudioGraph(el: HTMLAudioElement) {
	if (!audioCtx) {
		audioCtx = new AudioContext();
	}

	if (sourceNode) {
		sourceNode.disconnect();
		sourceNode = null;
	}
	filterNodes = [];

	sourceNode = audioCtx.createMediaElementSource(el);

	const filters = EQ_BANDS.map((freq, i) => {
		const filter = audioCtx!.createBiquadFilter();
		filter.type = i === 0 ? "lowshelf" : i === EQ_BANDS.length - 1 ? "highshelf" : "peaking";
		filter.frequency.value = freq;
		filter.gain.value = eq.enabled ? eq.gains[i] : 0;
		filter.Q.value = 1.0;
		return filter;
	});

	gainNode = audioCtx.createGain();
	gainNode.gain.value = 1;

	sourceNode.connect(filters[0]);
	for (let i = 0; i < filters.length - 1; i++) {
		filters[i].connect(filters[i + 1]);
	}
	filters[filters.length - 1].connect(gainNode);
	gainNode.connect(audioCtx.destination);

	filterNodes = filters;
}

export function applyEqToGraph() {
	filterNodes.forEach((filter, i) => {
		filter.gain.value = eq.enabled ? eq.gains[i] : 0;
	});
}

// ─── State persistence ────────────────────────────────────────────────────────

export function loadPlayerState() {
	try {
		const raw = localStorage.getItem(playerKey());
		if (!raw) return;
		const saved = JSON.parse(raw);
		player.volume = saved.volume ?? 1;
		player.muted = saved.muted ?? false;
		player.loopType = saved.loopType ?? 0;
		player.shuffleType = saved.shuffleType ?? 0;
		player.duration = saved.duration ?? 0;
		player.isPlaying = false;

		if (saved.track) {
			player.track = saved.track;
			startAudio(saved.track, false);

			if (audio && saved.currentTime) {
				const seek = () => {
					audio!.currentTime = saved.currentTime;
					player.currentTime = saved.currentTime;
					audio!.removeEventListener("canplay", seek);
				};
				audio.addEventListener("canplay", seek);
			}
		}
	} catch {}
	try {
		const raw = localStorage.getItem(queueKey());
		if (!raw) return;
		const saved = JSON.parse(raw);
		queuedTracks = saved.tracks ?? [];
		queueIndex = saved.index ?? -1;
		lastQueuedSet = saved.lastSet ?? [];
	} catch {}
}

export function savePlayerState() {
	if (audio) scrobbleEnd(audio.currentTime);

	try {
		localStorage.setItem(playerKey(), JSON.stringify({
			track: player.track,
			isPlaying: false,
			loopType: player.loopType,
			shuffleType: player.shuffleType,
			currentTime: player.currentTime,
			duration: player.duration,
			volume: player.volume,
			muted: player.muted,
		}));
	} catch {}
	try {
		localStorage.setItem(queueKey(), JSON.stringify({
			tracks: queuedTracks,
			index: queueIndex,
			lastSet: lastQueuedSet
		}));
	} catch {}
}

// ─── Audio element ────────────────────────────────────────────────────────────

function bindEvents(el: HTMLAudioElement) {
	el.addEventListener("timeupdate", () => {
		player.currentTime = el.currentTime;
	});
	el.addEventListener("durationchange", () => {
		player.duration = el.duration;
	});
	el.addEventListener("ended", () => {
		onTrackEnd();
	});
	el.addEventListener("play", () => {
		player.isPlaying = true;
		if (audioCtx && audioCtx.state === "suspended") {
			audioCtx.resume();
		}
	});
	el.addEventListener("pause", () => {
		player.isPlaying = false;
	});
}

function startAudio(track: Track, play: boolean = true, srcOverride?: string) {
	if (audio) {
		audio.pause();
		audio = null;
		sourceNode = null;
		filterNodes = [];
		gainNode = null;
	}

	player.track = track;
	currentlyPlaying.uid = track.uid;
	currentlyPlaying.track = track;

	const el = new Audio();
	el.crossOrigin = "anonymous";
	el.src = srcOverride ?? pathToSrc(track.path);
	el.volume = player.volume;
	el.muted = player.muted;
	bindEvents(el);
	audio = el;

	buildAudioGraph(el);

	if (!play) return;

	if (audioCtx && audioCtx.state === "suspended") {
		audioCtx.resume();
	}
	el.play();
}

// ─── Core playback logic ──────────────────────────────────────────────────────

async function tryPlayWithSrc(track: Track, src: string): Promise<boolean> {
	return new Promise((resolve) => {
		if (audio) {
			audio.pause();
			audio = null;
			sourceNode = null;
			filterNodes = [];
			gainNode = null;
		}

		player.track = track;
		currentlyPlaying.uid = track.uid;
		currentlyPlaying.track = track;

		const el = new Audio();
		el.crossOrigin = "anonymous";
		el.src = src;
		el.volume = player.volume;
		el.muted = player.muted;
		bindEvents(el);
		audio = el;
		buildAudioGraph(el);

		if (audioCtx && audioCtx.state === "suspended") {
			audioCtx.resume();
		}

		const onCanPlay = () => {
			cleanup();
			el.play().catch(() => resolve(false));
			resolve(true);
		};

		const onError = () => {
			cleanup();
			resolve(false);
		};

		const cleanup = () => {
			el.removeEventListener("canplay", onCanPlay);
			el.removeEventListener("error", onError);
		};

		el.addEventListener("canplay", onCanPlay);
		el.addEventListener("error", onError);

		el.load();
	});
}

async function playTrack(track: Track) {
	if (currentlyPlaying.track) {
		playedTracks.unshift(currentlyPlaying.track);
	}

	if (isGhostTrack(track)) {
		toast.warning("Cannot play track — no file available.");
		scrobbleStart(track);
		onTrackEnd(true);
		return;
	}

	const hasLocal = isLocalPath(track.path);
	const hasRemote = !!(track.remote_path && track.remote_path.length > 0);

	const localBr = localBitrate(track);
	const remoteBr = remoteBitrate(track);

	// Prefer local if it exists and is higher or equal quality (or remote bitrate unknown)
	const preferLocal = hasLocal && (!hasRemote || localBr >= remoteBr);

	if (preferLocal) {
		const ok = await tryPlayWithSrc(track, pathToSrc(track.path));
		if (ok) {
			scrobbleStart(track);
			return;
		}

		// Local failed — mark ghost, try remote
		await markGhost(track.uid);

		if (!hasRemote) {
			toast.warning("Cannot play track — local file missing.");
			onTrackEnd(true);
			return;
		}
	}

	// Try remote (either preferred or fallback)
	if (hasRemote) {
		if (offlineMode.offline) {
			// In offline mode: attempt anyway — if it works, exit offline mode
			const ok = await tryPlayWithSrc(track, pathToSrc(track.remote_path!));
			if (ok) {
				offlineMode.offline = false;
				offlineModeToastShown = false;
				scrobbleStart(track);
				return;
			} else {
				onTrackEnd(true);
				return;
			}
		}

		const ok = await tryPlayWithSrc(track, pathToSrc(track.remote_path!));
		if (ok) {
			scrobbleStart(track);
			return;
		}

		// Remote failed — check connectivity
		const online = await checkOnline();
		if (!online) {
			const autoOffline = await getAutoOfflineSetting();
			if (autoOffline) {
				triggerOfflineMode();
			} else {
				triggerOfflineMode();
			}
			// Fallback to local if available
			if (hasLocal) {
				const localOk = await tryPlayWithSrc(track, pathToSrc(track.path));
				if (localOk) {
					scrobbleStart(track);
					return;
				}
			}
		} else {
			// Online but couldn't play — ghost it
			await markGhost(track.uid);
			toast.warning("Cannot play track — stream unavailable.");
		}

		onTrackEnd(true);
		return;
	}

	// No remote, no working local
	toast.warning("Cannot play track — no file available.");
	onTrackEnd(true);
}

async function getAutoOfflineSetting(): Promise<boolean> {
	if (cachedAutoOffline !== null) return cachedAutoOffline;
	try {
		const settings = await invoke<Array<{ key: string; value: string }>>("get_settings");
		cachedAutoOffline = settings.find(s => s.key === "offline_mode_auto")?.value === "true";
		return cachedAutoOffline;
	} catch {
		return false;
	}
}

// ─── Public API ───────────────────────────────────────────────────────────────

export function playTrackByUid(uid: string) {
	const track = library.tracks.find((t) => t.uid === uid);
	if (!track) return;
	playTrack(track);
}

export function playTrackByObject(track: Track) {
	playTrack(track);
}

export function clearQueue() {
	queuedTracks.splice(0, queuedTracks.length);
	queueIndex = -1;
	lastQueuedSet = [];
}

export function getQueuedTracks(): Track[] {
	return queuedTracks.slice(queueIndex + 1);
}

export function getPlayedTracks(): Track[] {
	return playedTracks;
}

export function addTrackToQueue(track: Track) {
	queuedTracks.push(track);
}

export function queueTracksByObject(tracks: Track[], play: boolean = false, shuffle: boolean = false) {
	if (play) clearQueue();
	if (shuffle && player.shuffleType === 0) player.shuffleType = 1;

	lastQueuedSet = tracks;
	let ordered;
	switch (player.shuffleType) {
		case 1:
			ordered = spacedShuffle(tracks);
			break;
		case 2:
			ordered = smartShuffle(tracks);
			break;
		default:
			ordered = tracks;
	}

	ordered.forEach(t => queuedTracks.push(t));

	if (play) startPlayingQueue();
}

export function queueTracksByUid(uids: string[], play: boolean = false, shuffle: boolean = false) {
	if (play) clearQueue();
	if (shuffle && player.shuffleType === 0) player.shuffleType = 1;

	let tracks: Track[] = [];
	flushSync(() => {
		uids.forEach((uid) => {
			const track = library.tracks.find((t) => t.uid === uid);
			if (track) tracks.push(track);
		});
	});

	lastQueuedSet = tracks;
	let ordered;
	switch (player.shuffleType) {
		case 1:
			ordered = spacedShuffle(tracks);
			break;
		case 2:
			ordered = smartShuffle(tracks);
			break;
		default:
			ordered = tracks;
	}

	ordered.forEach(t => queuedTracks.push(t));

	if (play) startPlayingQueue();
}

export function queueTracksFromUid(uid: string, play: boolean = false, shuffle: boolean = false) {
	if (play) clearQueue();
	if (shuffle && player.shuffleType === 0) player.shuffleType = 1;

	let tracks: Track[] = [];
	let type = parseUidType(uid);

	if (type === "album") {
		const album = library.albums.find((a) => a.uid === uid);
		if (!album || !album.tracks) return;
		const trackRefs: { uid: string; name: string }[] = JSON.parse(album.tracks);
		trackRefs.forEach((ref) => {
			const track = library.tracks.find((t) => t.uid === ref.uid);
			if (track) tracks.push(track);
		});
	} else if (type === "playlist") {
		const playlist = library.playlists.find((p) => p.uid === uid);
		if (!playlist || !playlist.tracks) return;
		const trackRefs: { uid: string; name: string }[] = JSON.parse(playlist.tracks);
		trackRefs.forEach((ref) => {
			const track = library.tracks.find((t) => t.uid === ref.uid);
			if (track) tracks.push(track);
		});
	}

	lastQueuedSet = tracks;
	let ordered;
	switch (player.shuffleType) {
		case 1:
			ordered = spacedShuffle(tracks);
			break;
		case 2:
			ordered = smartShuffle(tracks);
			break;
		default:
			ordered = tracks;
	}

	ordered.forEach(t => queuedTracks.push(t));

	if (play) startPlayingQueue();
}

function startPlayingQueue() {
	if (queuedTracks.length === 0) return;
	queueIndex = 0;
	playTrack(queuedTracks[0]);
}

export function getQueuedTracksAll(): Track[] {
	return queuedTracks;
}

export function getQueueIndex(): number {
	return queueIndex;
}

export function reorderQueue(fromDisplayIndices: number[], toDisplayIndex: number) {
	const offset = queueIndex + 1;
	const absIndices = fromDisplayIndices.map(i => i + offset);
	const absTo = toDisplayIndex + offset;

	if (
		absIndices.some(i => i < offset || i >= queuedTracks.length) ||
		absTo < offset || absTo >= queuedTracks.length
	) return;

	const movingSet = new Set(absIndices);
	const removed = absIndices.map(i => queuedTracks[i]);
	const without = queuedTracks.filter((_, i) => !movingSet.has(i));

	const anchorTrack = queuedTracks[absTo];
	const anchorIndexInWithout = without.indexOf(anchorTrack);

	without.splice(anchorIndexInWithout, 0, ...removed);

	queuedTracks.splice(0, queuedTracks.length, ...without);
}

export function removeFromQueue(displayIndex: number) {
	const abs = displayIndex + queueIndex + 1;
	if (abs < queueIndex + 1 || abs >= queuedTracks.length) return;
	queuedTracks.splice(abs, 1);
}

export function insertIntoQueue(tracks: Track[], afterDisplayIndex?: number) {
	const offset = queueIndex + 1;
	if (afterDisplayIndex === undefined) {
		tracks.forEach(t => queuedTracks.push(t));
	} else {
		const absInsert = afterDisplayIndex + offset + 1;
		const clampedInsert = Math.min(absInsert, queuedTracks.length);
		queuedTracks.splice(clampedInsert, 0, ...tracks);
	}
}

export function seek(seconds: number) {
	scrobbleMarkSeeked();
	if (!audio || !isFinite(seconds)) return;
	audio.currentTime = seconds;
}

export function setVolume(vol: number) {
	player.volume = vol;
	if (audio) audio.volume = vol;
}

export function toggleMute() {
	player.muted = !player.muted;
	if (audio) audio.muted = player.muted;
}

export function skipBack() {
	scrobbleEnd(audio!.currentTime);
	const el = audio;
	if (!el) return;

	if (el.currentTime >= 2) {
		el.currentTime = 0;
		return;
	}

	if (queueIndex < 1) return;

	queueIndex--;
	playTrack(queuedTracks[queueIndex]);
}

export function skipNext() {
	scrobbleEnd(audio!.currentTime);
	let nextIndex = queueIndex + 1;
	if (nextIndex >= queuedTracks.length) {
		if (player.loopType === 2) {
			nextIndex = 0;
		} else {
			return;
		}
	}

	queueIndex = nextIndex;
	playTrack(queuedTracks[queueIndex]);
}

export function toggleLoop() {
	player.loopType = (player.loopType + 1) % 3;
}

export function toggleShuffle() {
	player.shuffleType = (player.shuffleType + 1) % 3;
}

export function togglePlay() {
	if (!audio) return;
	if (audioCtx && audioCtx.state === "suspended") {
		audioCtx.resume();
	}
	if (player.isPlaying) {
		scrobbleMarkPaused();
		audio.pause();
	} else {
		audio.play();
	}
}

function onTrackEnd(skipGhost = false) {
	if (!skipGhost) scrobbleEnd(audio!.currentTime);

	if (player.loopType === 1) {
		if (audio) {
			scrobbleStart(player.track!);
			audio.currentTime = 0;
			audio.play();
		}
		return;
	}

	const nextIndex = queueIndex + 1;

	if (nextIndex < queuedTracks.length) {
		queueIndex = nextIndex;
		playTrack(queuedTracks[queueIndex]);
		return;
	}

	if (player.loopType === 2 && lastQueuedSet.length > 0) {
		let requeued;
		switch (player.shuffleType) {
			case 1:
				requeued = spacedShuffle(lastQueuedSet);
				break;
			case 2:
				requeued = smartShuffle(lastQueuedSet);
				break;
			default:
				requeued = lastQueuedSet;
		}

		queuedTracks.splice(0, queuedTracks.length);
		requeued.forEach(t => queuedTracks.push(t));
		queueIndex = 0;
		playTrack(queuedTracks[0]);
		return;
	}

	player.isPlaying = false;
	player.currentTime = 0;
}

// ─── Shuffle algorithms (unchanged) ──────────────────────────────────────────

function spacedShuffle(tracks: Track[]): Track[] {
	const shuffled = [...tracks];
	for (let i = shuffled.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1));
		[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
	}

	if (shuffled.length < 4) return shuffled;

	const albumUid = (t: Track): string | null =>
		t.albums?.[0]?.uid ?? null;

	const trackNum = (t: Track): number | null =>
		t.albums?.[0]?.track_number ?? null;

	function tryRelocate(
		arr: Track[],
		i: Track,
		score: (a: Track, b: Track) => number,
		startAt: number,
		endAt: number
	): boolean {
		const current = score(arr[startAt - 1] ?? arr[0], i);
		if (current === 0) return false;

		let bestIdx = -1;
		let bestScore = current;

		for (let j = startAt + 1; j <= endAt; j++) {
			const candidate = arr[j];
			const prevTrack = arr[j - 1];
			const candidateScore = score(prevTrack, candidate);

			const newScoreAtI = score(arr[startAt - 1] ?? arr[0], candidate);
			const newScoreAtJ = score(prevTrack, i);

			if (newScoreAtI + newScoreAtJ < current + candidateScore) {
				if (newScoreAtI + newScoreAtJ < bestScore) {
					bestScore = newScoreAtI + newScoreAtJ;
					bestIdx = j;
				}
			}
		}

		if (bestIdx !== -1) {
			[arr[startAt], arr[bestIdx]] = [arr[bestIdx], arr[startAt]];
			return true;
		}
		return false;
	}

	function adjacencyScore(a: Track, b: Track): number {
		const aAlbum = albumUid(a);
		const bAlbum = albumUid(b);
		const aArtist = a.album_artist ?? null;
		const bArtist = b.album_artist ?? null;
		const aNum = trackNum(a);
		const bNum = trackNum(b);

		if (
			aAlbum && bAlbum && aAlbum === bAlbum &&
			aNum !== null && bNum !== null &&
			Math.abs(aNum - bNum) === 1
		) return 3;

		if (aAlbum && bAlbum && aAlbum === bAlbum) return 2;

		if (aArtist && bArtist && aArtist === bArtist) return 1;

		return 0;
	}

	const passes = Math.min(3, Math.floor(shuffled.length / 2));
	for (let pass = 0; pass < passes; pass++) {
		for (let i = 1; i < shuffled.length; i++) {
			const penalty = adjacencyScore(shuffled[i - 1], shuffled[i]);
			if (penalty > 0) {
				const lookAhead = Math.min(i + 20, shuffled.length - 1);
				tryRelocate(shuffled, shuffled[i], adjacencyScore, i, lookAhead);
			}
		}
	}

	return shuffled;
}

function smartShuffle(tracks: Track[]): Track[] {
	if (tracks.length < 4) return [...tracks].sort(() => Math.random() - 0.5);

	const albumUid = (t: Track): string | null =>
		t.albums?.[0]?.uid ?? null;

	const trackNum = (t: Track): number | null =>
		t.albums?.[0]?.track_number ?? null;

	const albumArtist = (t: Track): string | null =>
		t.album_artist?.trim() || null;

	const bpm = (t: Track): number | null =>
		(t.bpm != null && t.bpm > 0) ? t.bpm : null;

	const key = (t: Track): string | null =>
		t.key?.trim() || null;

	const genres = (t: Track): Set<string> => {
		try {
			const parsed = typeof t.genres === "string"
				? JSON.parse(t.genres)
				: t.genres;
			if (Array.isArray(parsed))
				return new Set(parsed.map((g: string) => g?.toLowerCase?.().trim()).filter(Boolean));
		} catch {}
		return new Set();
	};

	const tags = (t: Track): Set<string> => {
		try {
			const parsed = typeof t.tags === "string"
				? JSON.parse(t.tags)
				: t.tags;
			if (Array.isArray(parsed))
				return new Set(parsed.map((g: string) => g?.toLowerCase?.().trim()).filter(Boolean));
		} catch {}
		return new Set();
	};

	const isFavorite = (t: Track): boolean =>
		tags(t).has("favorite");

	const nonFavTags = (t: Track): Set<string> => {
		const t2 = tags(t);
		t2.delete("favorite");
		return t2;
	};

	const setsOverlap = (a: Set<string>, b: Set<string>): boolean => {
		for (const v of a) if (b.has(v)) return true;
		return false;
	};

	function separationPenalty(a: Track, b: Track): number {
		let score = 0;

		const aAlbum = albumUid(a);
		const bAlbum = albumUid(b);
		const sameAlbum = aAlbum !== null && aAlbum === bAlbum;

		if (sameAlbum) {
			const aNum = trackNum(a);
			const bNum = trackNum(b);
			if (aNum !== null && bNum !== null && Math.abs(aNum - bNum) === 1)
				score += 40;
		}

		if (sameAlbum) score += 25;

		const aArtist = albumArtist(a);
		const bArtist = albumArtist(b);
		if (aArtist !== null && aArtist === bArtist) score += 15;

		if (isFavorite(a) && isFavorite(b)) score += 3;

		return score;
	}

	function cohesionBonus(a: Track, b: Track): number {
		let score = 0;

		const aBpm = bpm(a);
		const bBpm = bpm(b);
		if (aBpm !== null && bBpm !== null && Math.abs(aBpm - bBpm) <= 10)
			score += 20;

		if (setsOverlap(genres(a), genres(b))) score += 15;

		if (setsOverlap(nonFavTags(a), nonFavTags(b))) score += 12;

		const aKey = key(a);
		const bKey = key(b);
		if (aKey !== null && aKey === bKey) score += 10;

		return score;
	}

	function edgeScore(a: Track, b: Track): number {
		return separationPenalty(a, b) - cohesionBonus(a, b);
	}

	const shuffled = [...tracks];
	for (let i = shuffled.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1));
		[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
	}

	const WINDOW = 30;
	const PASSES = 4;

	for (let pass = 0; pass < PASSES; pass++) {
		let improved = false;

		for (let i = 1; i < shuffled.length - 1; i++) {
			const currentEdgeScore =
				edgeScore(shuffled[i - 1], shuffled[i]) +
				edgeScore(shuffled[i], shuffled[i + 1]);

			const lo = Math.max(i + 1, 0);
			const hi = Math.min(i + WINDOW, shuffled.length - 1);

			for (let j = lo; j <= hi; j++) {
				const jPrev = shuffled[j - 1];
				const jNext = j + 1 < shuffled.length ? shuffled[j + 1] : null;

				const beforeSwap =
					currentEdgeScore +
					edgeScore(jPrev, shuffled[j]) +
					(jNext ? edgeScore(shuffled[j], jNext) : 0);

				const ti = shuffled[i];
				const tj = shuffled[j];

				const afterSwap =
					edgeScore(shuffled[i - 1], tj) +
					edgeScore(tj, shuffled[i + 1]) +
					edgeScore(jPrev === ti ? tj : jPrev, ti) +
					(jNext ? edgeScore(ti, jNext === ti ? tj : jNext) : 0);

				if (afterSwap < beforeSwap) {
					[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
					improved = true;
					break;
				}
			}
		}

		if (!improved) break;
	}

	return shuffled;
}