export type TrackAlbumEntry = {
	uid: string;
	name: string;
	track_number: number | null;
};

export type PlaylistTrackEntry = {
	uid: string;
	name: string;
	order: number;
};

export type Track = {
	id: number;
	uid: string;
	path: string;
	last_modified: number;
	title: string | null;
	artists: string | null;
	album_artist: string | null;
	albums: TrackAlbumEntry[] | null;
	genres: string | null;
	year: string | null;
	rating: number | null;
	tags: string | null;
	duration_ms: number | null;
	bpm: number | null;
	key: string | null;
	user_options: string | null;
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
	tracks: PlaylistTrackEntry[] | null;
	artwork_path: string | null;
	folder: string | null;
};

export type Lyrics = {
	track_uid: string;
	track_id: number;
	source: string;
	plain: string | null;
	synced: string | null;
	instrumental: boolean;
};

export type AudioCatagories = "track" | "album" | "artist" | "playlist" | "unknown";

export type UserOptions = {
	linkedShuffle: string | null;
	trimStart:     number | null;
	trimEnd:       number | null;
};

export type ParsedTrack = {
	title: string;
	artist: string;
	album: string;
	duration_ms: number | null;
	year: string | null;
	track_number: number | null;
};

export type ImportState = "idle" | "parsed" | "importing" | "done" | "error";

export type DuplicateGroup = {
	tracks: Track[];
};
 
export function parseUserOptions(raw: string | null | undefined): UserOptions {
	const defaults: UserOptions = { linkedShuffle: null, trimStart: null, trimEnd: null };
	if (!raw) return defaults;
	try {
		return { ...defaults, ...JSON.parse(raw) };
	} catch {
		return defaults;
	}
}
 
export function serializeUserOptions(opts: UserOptions): string {
	return JSON.stringify(opts);
}
