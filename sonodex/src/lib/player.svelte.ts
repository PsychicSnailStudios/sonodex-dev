import { playing } from "$lib/session.svelte";
import type { Track } from "$lib/types";

let audio: HTMLAudioElement | null = null;

export const player = $state({
	track: null as Track | null,
	isPlaying: false,
	currentTime: 0,
	duration: 0,
	volume: 1,
});

function bindEvents() {
	if (!audio) return;
	audio.ontimeupdate = () => { player.currentTime = audio!.currentTime; };
	audio.ondurationchange = () => { player.duration = audio!.duration; };
	audio.onended = () => { player.isPlaying = false; };
	audio.onplay = () => { player.isPlaying = true; };
	audio.onpause = () => { player.isPlaying = false; };
}

export function playTrack(track: Track) {
	if (audio) {
		audio.pause();
		audio = null;
	}
	player.track = track;
	playing.id = track.id;
	playing.type = "track";
	audio = new Audio(`file://${track.path}`);
	audio.volume = player.volume;
	bindEvents();
	audio.play();
}

export function togglePlay() {
	if (!audio) return;
	if (player.isPlaying) {
		audio.pause();
	} else {
		audio.play();
	}
}

export function seek(seconds: number) {
	if (!audio) return;
	audio.currentTime = seconds;
}

export function setVolume(vol: number) {
	player.volume = vol;
	if (audio) audio.volume = vol;
}

export function skipBack() {
	if (!audio) return;
	if (audio.currentTime > 3) {
		audio.currentTime = 0;
	}
}