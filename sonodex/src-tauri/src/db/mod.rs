pub mod album_manager;
pub mod artist_manager;
pub mod playlist_manager;
pub mod settings_manager;
pub mod track_manager;

pub use album_manager::*;
pub use artist_manager::*;
pub use playlist_manager::*;
pub use settings_manager::*;
pub use track_manager::*;

use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub fn get_db_path() -> PathBuf {
	let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
	path.push("sonodex");
	std::fs::create_dir_all(&path).ok();
	path.push("library.db");
	path
}

pub fn init_db(conn: &Connection) -> Result<()> {
	conn.execute_batch("
		CREATE TABLE IF NOT EXISTS library_paths (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			path TEXT NOT NULL UNIQUE
		);

		CREATE TABLE IF NOT EXISTS tracks (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			uid TEXT NOT NULL UNIQUE,
			path TEXT NOT NULL UNIQUE,
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
			artwork_path TEXT
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
			artwork_blob BLOB,
			artwork_path TEXT
		);

		CREATE TABLE IF NOT EXISTS settings (
			key TEXT PRIMARY KEY,
			value TEXT NOT NULL
		);

		CREATE INDEX IF NOT EXISTS idx_tracks_album_artist ON tracks(album_artist);
		CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);
		CREATE INDEX IF NOT EXISTS idx_tracks_uid ON tracks(uid);
		CREATE INDEX IF NOT EXISTS idx_albums_uid ON albums(uid);
		CREATE INDEX IF NOT EXISTS idx_artists_uid ON artists(uid);
		CREATE INDEX IF NOT EXISTS idx_playlists_uid ON playlists(uid);

		INSERT OR IGNORE INTO settings (key, value) VALUES
			('dark_mode', 'false'),
			('filename_priority_title', 'tag'),
			('filename_priority_artist', 'tag'),
			('filename_priority_album', 'tag'),
			('filename_priority_year', 'tag'),
			('filename_custom_pattern', ''),
			('folder_fallback_artist', 'false'),
			('folder_fallback_album', 'false'),
			('folder_fallback_year', 'false');
		INSERT OR IGNORE INTO settings (key, value) VALUES
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
			('api_audiodb_key', '');
		INSERT OR IGNORE INTO settings (key, value) VALUES
			('artist_tag_delimiters', ' / | ; '),
			('artist_filename_delimiters', ' / | ; | feat. | ft. | featuring '),
			('genre_delimiters', ' / | ; | , '),
			('scan_create_artists', 'true'),
			('scan_create_albums', 'true');
	")
}
