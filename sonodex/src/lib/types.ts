export type Track = {
	id: number;
	uid: string;
	path: string;
	last_modified: number;
	title: string | null;
	artists: string | null;
	album_artist: string | null;
	albums: string | null;
	genres: string | null;
	year: string | null;
	rating: number | null;
	tags: string | null;
	duration_ms: number | null;
	bpm: number | null;
	key: string | null;
	credits: string | null;
	label: string | null;
};

export type Album = {
	id: number;
	uid: string;
	format: string | null;
	title: string;
	rating: number | null;
	artists: string | null;
	album_artist: string | null;
	release_date: string | null;
	tags: string | null;
	genres: string | null;
	tracks: string | null;
	credits: string | null;
	label: string | null;
	artwork_path: string | null;
};

export type Artist = {
	id: number;
	uid: string;
	name: string;
	aka: string | null;
	about: string | null;
	tags: string | null;
	genres: string | null;
	websites: string | null;
	members: string | null;
	profile_art_path: string | null;
	banner_art_path: string | null;
};

export type Playlist = {
	id: number;
	uid: string;
	title: string;
	description: string | null;
	owner: string | null;
	tracks: string | null;
	artwork_path: string | null;
};

export type Lyrics = {
	id: number | null;
	track_id: number;
	source: string;
	plain: string | null;
	synced: string | null;
	instrumental: boolean;
};

export type AudioCatagories = "track" | "album" | "artist" | "playlist";