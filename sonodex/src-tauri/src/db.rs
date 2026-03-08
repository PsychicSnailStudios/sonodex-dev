use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: Option<i64>,
    pub path: String,
    pub last_modified: i64,
    pub title: Option<String>,
    pub artists: Option<String>,
    pub album_artist: Option<String>,
    pub albums: Option<String>,
    pub genres: Option<String>,
    pub year: Option<String>,
    pub rating: Option<f32>,
    pub tags: Option<String>,
    pub duration_ms: Option<i64>,
    pub bpm: Option<f32>,
    pub key: Option<String>,
    pub artwork: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibraryPath {
    pub id: Option<i64>,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setting {
    pub key: String,
    pub value: String,
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
            artwork BLOB
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tracks_album_artist ON tracks(album_artist);
        CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);

        INSERT OR IGNORE INTO settings (key, value) VALUES
            ('filename_priority_title', 'tag'),
            ('filename_priority_artist', 'tag'),
            ('filename_priority_album', 'tag'),
            ('filename_priority_year', 'tag'),
            ('filename_custom_pattern', '');
    ")
}

pub fn upsert_track(conn: &Connection, track: &Track) -> Result<()> {
    conn.execute(
        "INSERT INTO tracks (path, last_modified, title, artists, album_artist, albums, genres, year, rating, duration_ms, bpm, key, artwork)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(path) DO UPDATE SET
            last_modified = excluded.last_modified,
            title = excluded.title,
            artists = excluded.artists,
            album_artist = excluded.album_artist,
            albums = excluded.albums,
            genres = excluded.genres,
            year = excluded.year,
            rating = excluded.rating,
            duration_ms = excluded.duration_ms,
            bpm = excluded.bpm,
            key = excluded.key,
            artwork = excluded.artwork",
        params![
            track.path, track.last_modified, track.title, track.artists,
            track.album_artist, track.albums, track.genres, track.year,
            track.rating, track.duration_ms, track.bpm, track.key,
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
        "SELECT id, path, last_modified, title, artists, album_artist, albums, genres, year, rating, tags, duration_ms, bpm, key
         FROM tracks ORDER BY album_artist, albums, title"
    )?;
    let tracks = stmt.query_map([], |row| {
        Ok(Track {
            id: row.get(0)?,
            path: row.get(1)?,
            last_modified: row.get(2)?,
            title: row.get(3)?,
            artists: row.get(4)?,
            album_artist: row.get(5)?,
            albums: row.get(6)?,
            genres: row.get(7)?,
            year: row.get(8)?,
            rating: row.get(9)?,
            tags: row.get(10)?,
            duration_ms: row.get(11)?,
            bpm: row.get(12)?,
            key: row.get(13)?,
            artwork: None,
        })
    })?
    .collect::<Result<Vec<_>>>()?;
    Ok(tracks)
}

pub fn get_track_artwork(conn: &Connection, id: i64) -> Result<Option<Vec<u8>>> {
    let mut stmt = conn.prepare("SELECT artwork FROM tracks WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateGroup {
    pub tracks: Vec<Track>,
}

pub fn find_duplicates(conn: &Connection) -> Result<Vec<DuplicateGroup>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, last_modified, title, artists, album_artist, albums, genres, year, rating, tags, duration_ms, bpm, key
         FROM (
             SELECT a.id, a.path, a.last_modified, a.title, a.artists, a.album_artist, a.albums, a.genres, a.year, a.rating, a.tags, a.duration_ms, a.bpm, a.key
             FROM tracks a
             INNER JOIN tracks b ON (
                 a.id < b.id
                 AND LOWER(TRIM(a.title)) = LOWER(TRIM(b.title))
                 AND LOWER(TRIM(COALESCE(a.album_artist, JSON_EXTRACT(a.artists, '$[0]')))) = LOWER(TRIM(COALESCE(b.album_artist, JSON_EXTRACT(b.artists, '$[0]'))))
                 AND ABS(COALESCE(a.duration_ms, 0) - COALESCE(b.duration_ms, 0)) <= 1000
             )
             UNION
             SELECT b.id, b.path, b.last_modified, b.title, b.artists, b.album_artist, b.albums, b.genres, b.year, b.rating, b.tags, b.duration_ms, b.bpm, b.key
             FROM tracks a
             INNER JOIN tracks b ON (
                 a.id < b.id
                 AND LOWER(TRIM(a.title)) = LOWER(TRIM(b.title))
                 AND LOWER(TRIM(COALESCE(a.album_artist, JSON_EXTRACT(a.artists, '$[0]')))) = LOWER(TRIM(COALESCE(b.album_artist, JSON_EXTRACT(b.artists, '$[0]'))))
                 AND ABS(COALESCE(a.duration_ms, 0) - COALESCE(b.duration_ms, 0)) <= 1000
             )
         )
         ORDER BY title, album_artist, duration_ms"
    )?;

    let all_tracks = stmt.query_map([], |row| {
        Ok(Track {
            id: row.get(0)?,
            path: row.get(1)?,
            last_modified: row.get(2)?,
            title: row.get(3)?,
            artists: row.get(4)?,
            album_artist: row.get(5)?,
            albums: row.get(6)?,
            genres: row.get(7)?,
            year: row.get(8)?,
            rating: row.get(9)?,
            tags: row.get(10)?,
            duration_ms: row.get(11)?,
            bpm: row.get(12)?,
            key: row.get(13)?,
            artwork: None,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    let mut groups: Vec<DuplicateGroup> = Vec::new();
    let mut i = 0;
    while i < all_tracks.len() {
        let mut group = vec![all_tracks[i].clone()];
        let mut j = i + 1;
        while j < all_tracks.len() {
            let a = &all_tracks[i];
            let b = &all_tracks[j];
            let same_title = a.title.as_deref().map(|s| s.to_lowercase()) == b.title.as_deref().map(|s| s.to_lowercase());
            let same_artist = a.album_artist.as_deref().map(|s| s.to_lowercase()) == b.album_artist.as_deref().map(|s| s.to_lowercase());
            let duration_close = match (a.duration_ms, b.duration_ms) {
                (Some(da), Some(db)) => (da - db).abs() <= 1000,
                _ => false,
            };
            if same_title && same_artist && duration_close {
                group.push(all_tracks[j].clone());
                j += 1;
            } else {
                break;
            }
        }
        groups.push(DuplicateGroup { tracks: group });
        i = j;
    }

    Ok(groups)
}

pub fn delete_track_by_id(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
    Ok(())
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

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<Vec<Setting>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let settings = stmt.query_map([], |row| {
        Ok(Setting {
            key: row.get(0)?,
            value: row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;
    Ok(settings)
}