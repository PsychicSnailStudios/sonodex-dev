import { convertFileSrc } from "@tauri-apps/api/core";
import { flushSync } from "svelte";
import { library } from "$lib/ts/library.svelte";
import type { Track, AudioCatagories } from "$lib/ts/util/types";
import { parseTrackNumber, parseUidType } from "../util/helpers";

let audio: HTMLAudioElement | null = null;

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

export function loadPlayerState() {
	try {
		const raw = localStorage.getItem(PLAYER_STORAGE_KEY);
		if (!raw) return;
		const saved = JSON.parse(raw);
		player.volume = saved.volume ?? 1;
		player.muted = saved.muted ?? false;
		player.loopType = saved.loopType ?? 0;
		player.shuffleType = saved.shuffleType ?? 0;
		player.currentTime = saved.currentTime ?? 0;
		if (saved.track) player.track = saved.track;
		
		startAudio(saved.track, false);
		player.duration = saved.duration ?? 0;
		player.isPlaying = false;

	} catch {}
	try {
		const raw = localStorage.getItem(QUEUE_STORAGE_KEY);
		if (!raw) return;
		const saved = JSON.parse(raw);
		queuedTracks = saved.tracks ?? [];
	} catch {}
}

export function savePlayerState() {
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
	});
	el.addEventListener("pause", () => {
		player.isPlaying = false;
	});
}

function startAudio(track: Track, play: boolean = true) {
	if (audio) {
		audio.pause();
		audio = null;
	}

	player.track = track;
	currentlyPlaying.uid = track.uid;
	currentlyPlaying.track = track;

	const el = new Audio(convertFileSrc(track.path));
	el.volume = player.volume;
	bindEvents(el);
	audio = el;

	if (!play) return;
	el.play();
}

function playTrack(track: Track) {
	if (currentlyPlaying.track) {
		playedTracks.unshift(currentlyPlaying.track);
	}
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
	let nextIndex = queueIndex + 1;
	if (nextIndex >= queuedTracks.length) {
		if (player.loopType === 2) {
			nextIndex = 0;
		}
		else {
			return;
		}
	};

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
	if (player.isPlaying) {
		audio.pause();
	} else {
		audio.play();
	}
}

function onTrackEnd() {
	if (player.loopType === 1) {
		if (audio) {
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