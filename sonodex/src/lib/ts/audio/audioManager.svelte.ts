import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { flushSync } from "svelte";
import { toast } from "svelte-sonner";
import { library } from "$lib/ts/library.svelte";
import type { Track, AudioCatagories } from "$lib/ts/util/types";
import { parseTrackNumber, parseUidType } from "../util/helpers";
import { eq, EQ_BANDS } from "$lib/ts/app/eqStore.svelte";
import { scrobbleStart, scrobbleEnd, scrobbleMarkPaused, scrobbleMarkSeeked } from "$lib/ts/audio/scrobbler.svelte";
import { profileState } from "$lib/ts/profiles.svelte";

let audio: HTMLAudioElement | null = null;

let audioCtx: AudioContext | null = null;
let sourceNode: MediaElementAudioSourceNode | null = null;
let filterNodes: BiquadFilterNode[] = [];
let gainNode: GainNode | null = null;

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

	el.addEventListener("error", () => {
		const err = el.error;
		if (err && err.code === 4) {
			console.error(err);
			handleMissingPath();
		}
	});
}

async function handleMissingPath() {
	toast.warning("Cannot Play Track, No local file found.");

	// set track as ghost
	let uid = player.track!.uid;
	await invoke("update_track_metadata", {
		uid,
		update: { path: "" },
	});

	// skip track
	onTrackEnd(true);
}

function startAudio(track: Track, play: boolean = true) {
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
	el.src = convertFileSrc(track.path);
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

function playTrack(track: Track) {
	if (currentlyPlaying.track) {
		playedTracks.unshift(currentlyPlaying.track);
	}

	let isGhost = $derived(/^[a-z]+-[0-9a-f-]{36}$/.test(track.path));
	if (isGhost) {
		
		toast.warning("Cannot Play Track, No local file found.");
		
		scrobbleStart(track);
		onTrackEnd(true);
	}
	else {
		scrobbleStart(track);
		startAudio(track);
	}
}

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
	let ordered
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
	let ordered
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
	let ordered
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

function spacedShuffle(tracks: Track[]): Track[] {
    // Start with a Fisher-Yates shuffle
    const shuffled = [...tracks];
    for (let i = shuffled.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }

    if (shuffled.length < 4) return shuffled;

    // Helper to get first album uid from a track
    const albumUid = (t: Track): string | null =>
        t.albums?.[0]?.uid ?? null;

    // Helper to get first album's track_number
    const trackNum = (t: Track): number | null =>
        t.albums?.[0]?.track_number ?? null;

    // Try to relocate a track at index i by swapping it with the best candidate
    // further ahead in the array. Returns true if a beneficial swap was made.
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

            // Would swapping i and candidate improve both positions?
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

    // Score function: penalty for adjacency violations (higher = worse)
    function adjacencyScore(a: Track, b: Track): number {
        const aAlbum = albumUid(a);
        const bAlbum = albumUid(b);
        const aArtist = a.album_artist ?? null;
        const bArtist = b.album_artist ?? null;
        const aNum = trackNum(a);
        const bNum = trackNum(b);

        // Priority 1: consecutive track numbers on same album
        if (
            aAlbum && bAlbum && aAlbum === bAlbum &&
            aNum !== null && bNum !== null &&
            Math.abs(aNum - bNum) === 1
        ) return 3;

        // Priority 2: same album
        if (aAlbum && bAlbum && aAlbum === bAlbum) return 2;

        // Priority 3: same album artist
        if (aArtist && bArtist && aArtist === bArtist) return 1;

        return 0;
    }

    // Run several passes, each time trying to fix the worst adjacency violations
    const passes = Math.min(3, Math.floor(shuffled.length / 2));
    for (let pass = 0; pass < passes; pass++) {
        for (let i = 1; i < shuffled.length; i++) {
            const penalty = adjacencyScore(shuffled[i - 1], shuffled[i]);
            if (penalty > 0) {
                // Look ahead up to 20 positions for a swap candidate
                const lookAhead = Math.min(i + 20, shuffled.length - 1);
                tryRelocate(shuffled, shuffled[i], adjacencyScore, i, lookAhead);
            }
        }
    }

    return shuffled;
}

function smartShuffle(tracks: Track[]): Track[] {
    if (tracks.length < 4) return [...tracks].sort(() => Math.random() - 0.5);

    // --- Helpers ---

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

    // --- Scoring ---

    // Separation penalty: higher = these two should NOT be adjacent
    function separationPenalty(a: Track, b: Track): number {
        let score = 0;

        const aAlbum = albumUid(a);
        const bAlbum = albumUid(b);
        const sameAlbum = aAlbum !== null && aAlbum === bAlbum;

        // Consecutive tracks on the same album (highest priority)
        if (sameAlbum) {
            const aNum = trackNum(a);
            const bNum = trackNum(b);
            if (aNum !== null && bNum !== null && Math.abs(aNum - bNum) === 1)
                score += 40;
        }

        // Same album
        if (sameAlbum) score += 25;

        // Same album artist
        const aArtist = albumArtist(a);
        const bArtist = albumArtist(b);
        if (aArtist !== null && aArtist === bArtist) score += 15;

        // Both are favorites (very low priority)
        if (isFavorite(a) && isFavorite(b)) score += 3;

        return score;
    }

    // Cohesion bonus: higher = these two SHOULD be adjacent
    function cohesionBonus(a: Track, b: Track): number {
        let score = 0;

        // Matching BPM within 10
        const aBpm = bpm(a);
        const bBpm = bpm(b);
        if (aBpm !== null && bBpm !== null && Math.abs(aBpm - bBpm) <= 10)
            score += 20;

        // Overlapping genres
        if (setsOverlap(genres(a), genres(b))) score += 15;

        // Overlapping non-favorite tags
        if (setsOverlap(nonFavTags(a), nonFavTags(b))) score += 12;

        // Same key
        const aKey = key(a);
        const bKey = key(b);
        if (aKey !== null && aKey === bKey) score += 10;

        return score;
    }

    // Combined edge score: what we want to MINIMISE
    // Separation penalty pushes score up, cohesion bonus pushes it down
    function edgeScore(a: Track, b: Track): number {
        return separationPenalty(a, b) - cohesionBonus(a, b);
    }

    // Total score of the full arrangement (lower = better)
    function totalScore(arr: Track[]): number {
        let s = 0;
        for (let i = 1; i < arr.length; i++) s += edgeScore(arr[i - 1], arr[i]);
        return s;
    }

    // --- Initial shuffle ---
    const shuffled = [...tracks];
    for (let i = shuffled.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }

    // --- Optimisation passes ---
    // For each track that has a bad edge, search a window ahead/behind for a
    // swap that improves the total score at all four affected edges.
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

                // Simulate swap
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

        if (!improved) break; // Converged early
    }

    return shuffled;
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
	scrobbleEnd(audio!.currentTime)
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
	scrobbleEnd(audio!.currentTime)
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
	if (!skipGhost) scrobbleEnd(audio!.currentTime)

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
