import { convertFileSrc } from "@tauri-apps/api/core";
import { flushSync } from "svelte";
import { library } from "$lib/ts/library.svelte";
import type { Track, AudioCatagories } from "$lib/ts/util/types";
import { parseTrackNumber, parseUidType } from "../util/helpers";
import { eq, EQ_BANDS } from "$lib/ts/app/eqStore.svelte";
import { scrobbleStart, scrobbleEnd, scrobbleMarkPaused, scrobbleMarkSeeked } from "$lib/ts/audio/scrobbler.svelte";

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

const PLAYER_STORAGE_KEY = "sonodex:player";
const QUEUE_STORAGE_KEY = "sonodex:queue";

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
		const raw = localStorage.getItem(PLAYER_STORAGE_KEY);
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
		const raw = localStorage.getItem(QUEUE_STORAGE_KEY);
		if (!raw) return;
		const saved = JSON.parse(raw);
		queuedTracks = saved.tracks ?? [];
	} catch {}
}

export function savePlayerState() {
	scrobbleEnd(audio!.currentTime);

	try {
		localStorage.setItem(PLAYER_STORAGE_KEY, JSON.stringify({
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
		localStorage.setItem(QUEUE_STORAGE_KEY, JSON.stringify({
			tracks: queuedTracks,
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

	//const el = new Audio(convertFileSrc(track.path));
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
	scrobbleStart(track);
	startAudio(track);
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

	const ordered = (player.shuffleType === 1 || shuffle)
		? [...tracks].sort(() => Math.random() - 0.5)
		: [...tracks];

	lastQueuedSet = ordered;
	ordered.forEach(t => queuedTracks.push(t));

	if (play) startPlayingQueue();
}

export function queueTracksByUid(uids: string[], play: boolean = false, shuffle: boolean = false) {
	if (play) clearQueue();

	let tracks: Track[] = [];
	flushSync(() => {
		uids.forEach((uid) => {
			const track = library.tracks.find((t) => t.uid === uid);
			if (track) tracks.push(track);
		});
	});

	const ordered = (player.shuffleType === 1 || shuffle)
		? tracks.sort(() => Math.random() - 0.5)
		: tracks;

	lastQueuedSet = ordered;
	ordered.forEach(t => queuedTracks.push(t));

	if (play) startPlayingQueue();
}

export function queueTracksFromUid(uid: string, play: boolean = false, shuffle: boolean = false) {
	if (play) clearQueue();

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

	const ordered = (player.shuffleType === 1 || shuffle)
		? tracks.sort(() => Math.random() - 0.5)
		: tracks;

	lastQueuedSet = ordered;
	ordered.forEach(t => queuedTracks.push(t));

	if (play) startPlayingQueue();
}

function startPlayingQueue() {
	if (queuedTracks.length === 0) return;
	queueIndex = 0;
	playTrack(queuedTracks[0]);
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
	player.shuffleType = (player.shuffleType + 1) % 2;
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

function onTrackEnd() {
	scrobbleEnd(audio!.currentTime)

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
		const requeued = (player.shuffleType === 1)
			? [...lastQueuedSet].sort(() => Math.random() - 0.5)
			: [...lastQueuedSet];

		queuedTracks.splice(0, queuedTracks.length);
		requeued.forEach(t => queuedTracks.push(t));
		queueIndex = 0;
		playTrack(queuedTracks[0]);
		return;
	}

	player.isPlaying = false;
	player.currentTime = 0;
}