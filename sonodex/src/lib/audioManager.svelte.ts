import { convertFileSrc } from "@tauri-apps/api/core";
import { flushSync } from "svelte";
import { library } from "$lib/library.svelte";
import type { Track, AudioCatagories } from "$lib/types";

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

function startAudio(track: Track) {
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
	el.play();
}

function playTrack(track: Track) {
	if (currentlyPlaying.track) {
		playedTracks.push(currentlyPlaying.track);
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

export function queueTracksFromUid(uid: string, type: AudioCatagories, play: boolean = false, shuffle: boolean = false) {
	if (play) clearQueue();

	let tracks: Track[] = [];

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

	const prev = playedTracks.pop();
	if (!prev) {
		el.currentTime = 0;
		return;
	}

	startAudio(prev);

	if (queueIndex > 0) queueIndex--;
}

export function skipNext() {
	const nextIndex = queueIndex + 1;
	if (nextIndex >= queuedTracks.length) return;

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