import { convertFileSrc } from "@tauri-apps/api/core";
import { flushSync } from "svelte";
import { library } from "$lib/library.svelte";
import { Queue } from "$lib/queue.svelte";
import type { Track, AudioCatagories } from "$lib/types";

let audio: HTMLAudioElement | null = null;

const trackQueue = new Queue<Track>();
const playedTrackQueue = new Queue<Track>();

export const currentlyPlaying = $state({
	uid: "" as string,
	track: null as Track | null,
});

export const player = $state({
	track: null as Track | null,
	isPlaying: false,
	currentTime: 0,
	duration: 0,
	volume: 1,
});

function bindEvents(el: HTMLAudioElement) {
	el.addEventListener("timeupdate", () => {
		player.currentTime = el.currentTime;
	});
	el.addEventListener("durationchange", () => {
		player.duration = el.duration;
	});
	el.addEventListener("ended", () => {
		if (trackQueue.size() > 0) {
			let nextTrack = trackQueue.dequeue();
			if (nextTrack) {
				playedTrackQueue.enqueue(currentlyPlaying.track!);
				playTrackByObject(nextTrack);
			}
		} else {
			player.isPlaying = false;
			player.currentTime = 0;
		}
	});
	el.addEventListener("play", () => {
		player.isPlaying = true;
	});
	el.addEventListener("pause", () => {
		player.isPlaying = false;
	});
}

export function playTrackByUid(uid: string) {
	if (audio) {
		audio.pause();
		audio = null;
	}

	const track = library.tracks.find((t) => t.uid === uid);
	if (!track) return;

	player.track = track;
	currentlyPlaying.uid = track.uid;
	currentlyPlaying.track = track;

	const el = new Audio(convertFileSrc(track.path));
	el.volume = player.volume;
	bindEvents(el);
	audio = el;
	el.play();
}

export function playTrackByObject(track: Track) {
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

export function clearQueue() {
	trackQueue.clear();
}

export function getQueuedTracks(): Track[] {
	return trackQueue.getQueue();
}

export function getPlayedTracks(): Track[] {
	return playedTrackQueue.getQueue();
}

export function queueTracksByObject(tracks: Track[], play: boolean = false) {
	if (play) {
		clearQueue();
	}

	flushSync(() => {
		tracks.forEach((t) => {
			trackQueue.enqueue(t);
		});
	});

	if (play) {
		const first = trackQueue.dequeue();
		if (first) playTrackByObject(first);
	}
}

export function queueTracksByUid(uids: string[], play: boolean = false) {
	if (play) {
		clearQueue();
	}

	flushSync(() => {
		uids.forEach((uid) => {
			let track = library.tracks.find((t) => t.uid === uid);
			if (track) trackQueue.enqueue(track);
		});
	});

	if (play) {
		const first = trackQueue.dequeue();
		if (first) playTrackByObject(first);
	}
}

export function queueTracksFromUid(uid: string, type: AudioCatagories, play: boolean = false) {
	if (play) {
		clearQueue();
	}

	if (type === "album") {
		const album = library.albums.find((a) => a.uid === uid);
		if (!album || !album.tracks) return;

		const trackRefs: { uid: string; name: string }[] = JSON.parse(album.tracks);
		trackRefs.forEach((ref) => {
			const track = library.tracks.find((t) => t.uid === ref.uid);
			if (track) trackQueue.enqueue(track);
		});
	} else if (type === "playlist") {
		const playlist = library.playlists.find((p) => p.uid === uid);
		if (!playlist || !playlist.tracks) return;

		const trackRefs: { uid: string; name: string }[] = JSON.parse(playlist.tracks);
		trackRefs.forEach((ref) => {
			const track = library.tracks.find((t) => t.uid === ref.uid);
			if (track) trackQueue.enqueue(track);
		});
	}

	if (play) {
		const first = trackQueue.dequeue();
		if (first) playTrackByObject(first);
	}
}

export function togglePlay() {
	const el = audio;
	if (!el) return;
	if (player.isPlaying) {
		el.pause();
	} else {
		el.play();
	}
}

export function seek(seconds: number) {
	const el = audio;
	if (!el || !isFinite(seconds)) return;
	el.currentTime = seconds;
}

export function setVolume(vol: number) {
	player.volume = vol;
	if (audio) audio.volume = vol;
}

export function skipBack() {
	const el = audio;
	if (!el) return;

	if (el.currentTime < 2) {
		const prevTrack = playedTrackQueue.dequeue();
		if (!prevTrack) { el.currentTime = 0; return; }
		if (currentlyPlaying.track) playedTrackQueue.enqueue(currentlyPlaying.track);
		playTrackByObject(prevTrack);
	} else {
		el.currentTime = 0;
	}
}

export function skipNext() {
	if (trackQueue.size() === 0) return;

	const nextTrack = trackQueue.dequeue();
	if (!nextTrack) return;
	if (currentlyPlaying.track) playedTrackQueue.enqueue(currentlyPlaying.track);
	playTrackByObject(nextTrack);
}