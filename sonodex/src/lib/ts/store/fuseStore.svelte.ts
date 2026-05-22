import Fuse from "fuse.js";
import { library } from "$ts/store/library.svelte";
import type { Track, Album, Artist } from "$ts/util/types";
import { parseAlbumsToString, parseArtistsToString } from "$ts/util/parsers";

function makeFuse<T>(list: T[], keys: Fuse.FuseOptionKey<T>[]): Fuse<T> {
	return new Fuse(list, {
		keys,
		threshold: 0.35,
		ignoreLocation: true,
		includeScore: false,
		useExtendedSearch: false,
		minMatchCharLength: 2,
	});
}

let _trackFuse: Fuse<Track> | null = null;
let _trackRef: Track[] | null = null;

let _albumFuse: Fuse<Album> | null = null;
let _albumRef: Album[] | null = null;

let _artistFuse: Fuse<Artist> | null = null;
let _artistRef: Artist[] | null = null;

function getTrackFuse(): Fuse<Track> {
	if (_trackFuse && _trackRef === library.tracks) return _trackFuse;
	_trackRef = library.tracks;
	_trackFuse = makeFuse(library.tracks, [
		{ name: "title",        weight: 0.5,  getFn: (t: Track) => t.title ?? "" },
		{ name: "artists",      weight: 0.25, getFn: (t: Track) => parseArtistsToString(t.artists) },
		{ name: "album_artist", weight: 0.15, getFn: (t: Track) => t.album_artist ?? "" },
		{ name: "albums",       weight: 0.1,  getFn: (t: Track) => parseAlbumsToString(t.albums) },
		{ name: "tags",         weight: 0.05, getFn: (t: Track) => t.tags ?? "" },
		{ name: "genres",       weight: 0.05, getFn: (t: Track) => t.genres ?? "" },
	]);
	return _trackFuse;
}

function getAlbumFuse(): Fuse<Album> {
	if (_albumFuse && _albumRef === library.albums) return _albumFuse;
	_albumRef = library.albums;
	_albumFuse = makeFuse(library.albums, [
		{ name: "title",        weight: 0.5,  getFn: (a: Album) => a.title ?? "" },
		{ name: "artists",      weight: 0.25, getFn: (a: Album) => parseArtistsToString(a.artists) },
		{ name: "album_artist", weight: 0.15, getFn: (a: Album) => a.album_artist ?? "" },
		{ name: "year",         weight: 0.1,  getFn: (a: Album) => a.release_date ?? "" },
		{ name: "tags",         weight: 0.05, getFn: (a: Album) => a.tags ?? "" },
		{ name: "genres",       weight: 0.05, getFn: (a: Album) => a.genres ?? "" },
	]);
	return _albumFuse;
}

function getArtistFuse(): Fuse<Artist> {
	if (_artistFuse && _artistRef === library.artists) return _artistFuse;
	_artistRef = library.artists;
	_artistFuse = makeFuse(library.artists, [
		{ name: "name",   weight: 0.5,  getFn: (a: Artist) => a.name ?? "" },
		{ name: "akas",   weight: 0.35, getFn: (a: Artist) => a.aka ?? "" },
		{ name: "tags",   weight: 0.05, getFn: (a: Artist) => a.tags ?? "" },
		{ name: "genres", weight: 0.05, getFn: (a: Artist) => a.genres ?? "" },
	]);
	return _artistFuse;
}

export function searchTracks(query: string): Track[] {
	if (query.trim().length < 2) return library.tracks;
	return getTrackFuse().search(query).map((r) => r.item);
}

export function searchAlbums(query: string): Album[] {
	if (query.trim().length < 2) return library.albums;
	return getAlbumFuse().search(query).map((r) => r.item);
}

export function searchArtists(query: string): Artist[] {
	if (query.trim().length < 2) return library.artists;
	return getArtistFuse().search(query).map((r) => r.item);
}

// Call this after library load, deferred with setTimeout(warmupFuseIndexes, 0),
// so index construction happens after the UI has painted and feels responsive.
export function warmupFuseIndexes(): void {
	getTrackFuse();
	getAlbumFuse();
	getArtistFuse();
}