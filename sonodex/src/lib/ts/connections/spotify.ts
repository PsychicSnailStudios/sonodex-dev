// Spotify connection helpers

import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { listen } from "@tauri-apps/api/event";

// ── Types ─────────────────────────────────────────────────────────────────────

export type SpotifyPlaylistSummary = {
	id: string;
	name: string;
	description: string | null;
	track_count: number;
	owner: string;
};

// ── Connection state ──────────────────────────────────────────────────────────

export async function spotifyIsConnected(): Promise<boolean> {
	return invoke<boolean>("spotify_connection_status");
}

// ── Auth ──────────────────────────────────────────────────────────────────────

/**
 * Begin Spotify PKCE OAuth. Opens the Spotify auth page in the system browser.
 * After the user grants access, Spotify redirects to sonodex://spotify-callback?code=...
 * which the Rust deep-link handler picks up and exchanges automatically.
 *
 * The code verifier is stored in AppState on the Rust side — no frontend storage needed.
 * Listen for "spotify:connected" to know when auth completes.
 */
export async function connectSpotify(): Promise<void> {
	// Returns [url, verifier] — verifier is stored in AppState by the command
	const [url] = await invoke<[string, string]>("spotify_get_auth_url");
	await open(url);
}

export async function disconnectSpotify(): Promise<void> {
	await invoke("spotify_disconnect_cmd");
}

/**
 * Listen for the connected event emitted after a successful deep-link callback.
 * Returns an unlisten function.
 */
export async function onSpotifyConnected(cb: () => void) {
	return listen("spotify:connected", cb);
}

// ── Playlists ─────────────────────────────────────────────────────────────────

/** Fetch the user's Spotify playlists. */
export async function getSpotifyPlaylists(): Promise<SpotifyPlaylistSummary[]> {
	return invoke<SpotifyPlaylistSummary[]>("spotify_get_playlists_cmd");
}

/**
 * Import a Spotify playlist into the local library as a Sonodex playlist.
 * Tracks already in the library are matched; unrecognised tracks become stubs.
 * Returns the new playlist's uid.
 */
export async function importSpotifyPlaylist(
	spotifyPlaylistId: string,
	playlistName: string,
	owner?: string,
): Promise<string> {
	return invoke<string>("spotify_import_playlist_cmd", {
		spotifyPlaylistId,
		playlistName,
		owner: owner ?? null,
	});
}

// ── Enrichment ────────────────────────────────────────────────────────────────

export async function spotifyEnrichTrack(uid: string): Promise<void> {
	await invoke("spotify_enrich_track_cmd", { uid });
}

export async function spotifyEnrichAlbum(uid: string): Promise<void> {
	await invoke("spotify_enrich_album_cmd", { uid });
}

export async function spotifyEnrichArtist(uid: string): Promise<void> {
	await invoke("spotify_enrich_artist_cmd", { uid });
}
