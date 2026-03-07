use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: Option<i64>,
    pub path: String,
    pub last_modified: i64,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub duration_ms: Option<i64>,
    pub bpm: Option<f32>,
    pub key: Option<String>,
    pub track_number: Option<i32>,
    pub artwork: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibraryPath {
    pub id: Option<i64>,
    pub path: String,
}

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
            path TEXT NOT NULL UNIQUE,
            last_modified INTEGER NOT NULL,
            title TEXT,
            artist TEXT,
            album TEXT,
            album_artist TEXT,
            genre TEXT,
            year INTEGER,
            duration_ms INTEGER,
            bpm REAL,
            key TEXT,
            track_number INTEGER,
            artwork BLOB
        );

        CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
        CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
        CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);
    ")
}

pub fn upsert_track(conn: &Connection, track: &Track) -> Result<()> {
    conn.execute(
        "INSERT INTO tracks (path, last_modified, title, artist, album, album_artist, genre, year, duration_ms, bpm, key, track_number, artwork)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(path) DO UPDATE SET
            last_modified = excluded.last_modified,
            title = excluded.title,
            artist = excluded.artist,
            album = excluded.album,
            album_artist = excluded.album_artist,
            genre = excluded.genre,
            year = excluded.year,
            duration_ms = excluded.duration_ms,
            bpm = excluded.bpm,
            key = excluded.key,
            track_number = excluded.track_number,
            artwork = excluded.artwork",
        params![
            track.path, track.last_modified, track.title, track.artist,
            track.album, track.album_artist, track.genre, track.year,
            track.duration_ms, track.bpm, track.key, track.track_number,
            track.artwork
        ],
    )?;
    Ok(())
}

pub fn delete_track(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("DELETE FROM tracks WHERE path = ?1", params![path])?;
    Ok(())
}

pub fn get_all_tracks(conn: &Connection) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, last_modified, title, artist, album, album_artist, genre, year, duration_ms, bpm, key, track_number, artwork FROM tracks ORDER BY artist, album, track_number"
    )?;
    let tracks = stmt.query_map([], |row| {
        Ok(Track {
            id: row.get(0)?,
            path: row.get(1)?,
            last_modified: row.get(2)?,
            title: row.get(3)?,
            artist: row.get(4)?,
            album: row.get(5)?,
            album_artist: row.get(6)?,
            genre: row.get(7)?,
            year: row.get(8)?,
            duration_ms: row.get(9)?,
            bpm: row.get(10)?,
            key: row.get(11)?,
            track_number: row.get(12)?,
            artwork: row.get(13)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;
    Ok(tracks)
}

pub fn add_library_path(conn: &Connection, path: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO library_paths (path) VALUES (?1)",
        params![path],
    )?;
    Ok(())
}

pub fn remove_library_path(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("DELETE FROM library_paths WHERE path = ?1", params![path])?;
    conn.execute("DELETE FROM tracks WHERE path LIKE ?1", params![format!("{}%", path)])?;
    Ok(())
}

pub fn get_library_paths(conn: &Connection) -> Result<Vec<LibraryPath>> {
    let mut stmt = conn.prepare("SELECT id, path FROM library_paths")?;
    let paths = stmt.query_map([], |row| {
        Ok(LibraryPath {
            id: row.get(0)?,
            path: row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;
    Ok(paths)
}