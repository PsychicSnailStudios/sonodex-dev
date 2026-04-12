use uuid::Uuid;

pub mod album_manager;
pub mod artist_manager;
pub mod lyrics_manager;
pub mod playlist_manager;
pub mod settings_manager;
pub mod track_manager;
pub mod analytics_manager;

pub use album_manager::*;
pub use artist_manager::*;
pub use lyrics_manager::*;
pub use playlist_manager::*;
pub use settings_manager::*;
pub use track_manager::*;

use rusqlite::{Connection, Result};

pub fn generate_uid(prefix: &str) -> String {
	format!("{}-{}", prefix, Uuid::new_v4())
}

pub fn init_settings_db(conn: &Connection) -> Result<()> {
	conn.execute_batch(
		"
		CREATE TABLE IF NOT EXISTS library_paths (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			path TEXT NOT NULL UNIQUE
		);

		CREATE TABLE IF NOT EXISTS settings (
			key TEXT PRIMARY KEY,
			value TEXT NOT NULL
		);

		CREATE TABLE IF NOT EXISTS shared_connections (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			uid TEXT NOT NULL UNIQUE,
			source_uid TEXT NOT NULL,
			source_name TEXT NOT NULL,
			source_type TEXT NOT NULL DEFAULT 'file',
			source_url TEXT,
			last_synced INTEGER
		);

		CREATE TABLE IF NOT EXISTS uuid_remaps (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			shared_uid TEXT NOT NULL,
			local_uid TEXT NOT NULL,
			entity_type TEXT NOT NULL,
			UNIQUE(shared_uid, entity_type)
		);

		INSERT OR IGNORE INTO settings (key, value) VALUES
			('dark_mode', 'false'),
			('filename_priority_title', 'tag'),
			('filename_priority_artist', 'tag'),
			('filename_priority_album', 'tag'),
			('filename_priority_year', 'tag'),
			('filename_custom_pattern', ''),
			('enrich_primary_api', 'musicbrainz'),
			('enrich_priority_title', 'local'),
			('enrich_priority_artists', 'local'),
			('enrich_priority_album_artist', 'local'),
			('enrich_priority_album', 'local'),
			('enrich_priority_year', 'local'),
			('enrich_priority_genres', 'local'),
			('enrich_priority_bpm', 'local'),
			('enrich_priority_key', 'local'),
			('enrich_priority_artwork', 'local'),
			('api_audiodb_key', ''),
			('auto_enrich_tracks', 'false'),
			('auto_enrich_albums', 'false'),
			('auto_enrich_artists', 'true'),
			('scan_try_parse_ampersand', 'true'),
			('auto_fetch_lyrics', 'false'),
			('api_lastfm_key', ''),
			('api_discogs_key', ''),
			('api_lastfm_secret',     ''),
			('lastfm_session_key',    ''),
			('spotify_client_id',     '3d32b50657484297a6e1069a085c5d83'),
			('spotify_access_token',  ''),
			('spotify_refresh_token', ''),
			('spotify_token_expiry',  '0'),
			('artist_tag_delimiters', ' / |; |, |,'),
			('artist_filename_delimiters', ' / |; | feat. | ft. | featuring '),
			('genre_delimiters', ' / |; |, '),
			('scan_create_artists', 'true'),
			('scan_create_albums', 'true');
	",
	)
}

pub fn init_lib_db(conn: &Connection) -> Result<()> {
	conn.execute_batch(
		"
		CREATE TABLE IF NOT EXISTS tracks (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			uid TEXT NOT NULL UNIQUE,
			path TEXT NOT NULL,
			last_modified INTEGER NOT NULL,
			title TEXT,
			artists TEXT,
			album_artist TEXT,
			albums TEXT,
			genres TEXT,
			year TEXT,
			rating REAL,
			tags TEXT DEFAULT '[]',
			duration_ms INTEGER,
			bpm REAL,
			key TEXT,
			credits TEXT,
			label TEXT,
			artwork_blob BLOB,
			artwork_path TEXT,
			user_options TEXT
		);

		CREATE TABLE IF NOT EXISTS albums (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			uid TEXT NOT NULL UNIQUE,
			format TEXT,
			title TEXT NOT NULL,
			rating REAL,
			artists TEXT,
			album_artist TEXT,
			release_date TEXT,
			tags TEXT DEFAULT '[]',
			genres TEXT DEFAULT '[]',
			tracks TEXT DEFAULT '[]',
			credits TEXT,
			label TEXT,
			artwork_blob BLOB,
			artwork_path TEXT
		);

		CREATE TABLE IF NOT EXISTS artists (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			uid TEXT NOT NULL UNIQUE,
			name TEXT NOT NULL,
			aka TEXT DEFAULT '[]',
			about TEXT,
			tags TEXT DEFAULT '[]',
			genres TEXT DEFAULT '[]',
			websites TEXT DEFAULT '[]',
			members TEXT DEFAULT '[]',
			profile_art_blob BLOB,
			profile_art_path TEXT,
			banner_art_blob BLOB,
			banner_art_path TEXT
		);

		CREATE TABLE IF NOT EXISTS playlists (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			uid TEXT NOT NULL UNIQUE,
			title TEXT NOT NULL,
			description TEXT,
			owner TEXT,
			tracks TEXT DEFAULT '[]',
			folder TEXT,
			artwork_blob BLOB,
			artwork_path TEXT
		);

		CREATE TABLE IF NOT EXISTS lyrics (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			track_id INTEGER NOT NULL UNIQUE,
			source TEXT NOT NULL,
			plain TEXT,
			synced TEXT,
			instrumental INTEGER NOT NULL DEFAULT 0,
			FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
		);

		CREATE INDEX IF NOT EXISTS idx_tracks_album_artist ON tracks(album_artist);
		CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);
		CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_uid ON tracks(uid);
		CREATE UNIQUE INDEX IF NOT EXISTS idx_albums_uid ON albums(uid);
		CREATE UNIQUE INDEX IF NOT EXISTS idx_artists_uid ON artists(uid);
		CREATE UNIQUE INDEX IF NOT EXISTS idx_playlists_uid ON playlists(uid);
		CREATE INDEX IF NOT EXISTS idx_playlists_folder ON playlists(folder);
	",
	)
}