use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
	pub id: Option<i64>,
	pub uid: String,
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
	pub credits: Option<String>,
	pub label: Option<String>,
	pub artwork_blob: Option<Vec<u8>>,
	pub artwork_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateGroup {
	pub tracks: Vec<Track>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataUpdate {
	pub title: Option<String>,
	pub artists: Option<String>,
	pub album_artist: Option<String>,
	pub albums: Option<String>,
	pub year: Option<String>,
	pub genres: Option<String>,
	pub bpm: Option<f32>,
	pub rating: Option<f32>,
	pub tags: Option<String>,
	pub key: Option<String>,
	pub credits: Option<String>,
	pub label: Option<String>,
	#[serde(skip)]
	pub artwork_blob: Option<Vec<u8>>,
	pub artwork_path: Option<String>,
}

pub fn upsert_track(conn: &Connection, track: &Track) -> Result<()> {
	conn.execute(
		"INSERT INTO tracks (uid, path, last_modified, title, artists, album_artist, albums, genres, year, rating, duration_ms, bpm, key, credits, label, artwork_blob, artwork_path)
		 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
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
			credits = excluded.credits,
			label = excluded.label,
			artwork_blob = excluded.artwork_blob,
			artwork_path = excluded.artwork_path",
		params![
			track.uid, track.path, track.last_modified, track.title, track.artists,
			track.album_artist, track.albums, track.genres, track.year,
			track.rating, track.duration_ms, track.bpm, track.key,
			track.credits, track.label,
			track.artwork_blob, track.artwork_path
		],
	)?;
	Ok(())
}

pub fn delete_track(conn: &Connection, path: &str) -> Result<()> {
	conn.execute("DELETE FROM tracks WHERE path = ?1", params![path])?;
	Ok(())
}

pub fn delete_track_by_id(conn: &Connection, id: i64) -> Result<()> {
	conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
	Ok(())
}

pub fn delete_track_by_uid(conn: &Connection, uid: &str) -> Result<()> {
	conn.execute("DELETE FROM tracks WHERE uid = ?1", params![uid])?;
	Ok(())
}

pub fn get_all_tracks(conn: &Connection) -> Result<Vec<Track>> {
	let mut stmt = conn.prepare(
		"SELECT id, uid, path, last_modified, title, artists, album_artist, albums, genres, year, rating, tags, duration_ms, bpm, key, credits, label, artwork_path
		 FROM tracks ORDER BY album_artist, albums, title"
	)?;
	let tracks = stmt
		.query_map([], |row| {
			Ok(Track {
				id: row.get(0)?,
				uid: row.get(1)?,
				path: row.get(2)?,
				last_modified: row.get(3)?,
				title: row.get(4)?,
				artists: row.get(5)?,
				album_artist: row.get(6)?,
				albums: row.get(7)?,
				genres: row.get(8)?,
				year: row.get(9)?,
				rating: row.get(10)?,
				tags: row.get(11)?,
				duration_ms: row.get(12)?,
				bpm: row.get(13)?,
				key: row.get(14)?,
				credits: row.get(15)?,
				label: row.get(16)?,
				artwork_blob: None,
				artwork_path: row.get(17)?,
			})
		})?
		.collect::<Result<Vec<_>>>()?;
	Ok(tracks)
}

pub fn get_track_by_uid(conn: &Connection, uid: &str) -> Result<Option<Track>> {
	let mut stmt = conn.prepare(
		"SELECT id, uid, path, last_modified, title, artists, album_artist, albums, genres, year, rating, tags, duration_ms, bpm, key, credits, label, artwork_path
		 FROM tracks WHERE uid = ?1"
	)?;
	let mut rows = stmt.query(params![uid])?;
	if let Some(row) = rows.next()? {
		Ok(Some(Track {
			id: row.get(0)?,
			uid: row.get(1)?,
			path: row.get(2)?,
			last_modified: row.get(3)?,
			title: row.get(4)?,
			artists: row.get(5)?,
			album_artist: row.get(6)?,
			albums: row.get(7)?,
			genres: row.get(8)?,
			year: row.get(9)?,
			rating: row.get(10)?,
			tags: row.get(11)?,
			duration_ms: row.get(12)?,
			bpm: row.get(13)?,
			key: row.get(14)?,
			credits: row.get(15)?,
			label: row.get(16)?,
			artwork_blob: None,
			artwork_path: row.get(17)?,
		}))
	} else {
		Ok(None)
	}
}

pub fn get_track_artwork(conn: &Connection, id: i64) -> Result<Option<Vec<u8>>> {
	let mut stmt = conn.prepare("SELECT artwork_blob FROM tracks WHERE id = ?1")?;
	let mut rows = stmt.query(params![id])?;
	if let Some(row) = rows.next()? {
		Ok(row.get(0)?)
	} else {
		Ok(None)
	}
}

pub fn get_track_artwork_by_uid(conn: &Connection, uid: &str) -> Result<Option<Vec<u8>>> {
	let mut stmt = conn.prepare("SELECT artwork_blob FROM tracks WHERE uid = ?1")?;
	let mut rows = stmt.query(params![uid])?;
	if let Some(row) = rows.next()? {
		Ok(row.get(0)?)
	} else {
		Ok(None)
	}
}

pub fn find_duplicates(conn: &Connection) -> Result<Vec<DuplicateGroup>> {
	let mut stmt = conn.prepare(
		"SELECT id, uid, path, last_modified, title, artists, album_artist, albums, genres, year, rating, tags, duration_ms, bpm, key, credits, label, artwork_path
		 FROM (
			 SELECT a.id, a.uid, a.path, a.last_modified, a.title, a.artists, a.album_artist, a.albums, a.genres, a.year, a.rating, a.tags, a.duration_ms, a.bpm, a.key, a.credits, a.label, a.artwork_path
			 FROM tracks a
			 INNER JOIN tracks b ON (
				 a.id < b.id
				 AND LOWER(TRIM(a.title)) = LOWER(TRIM(b.title))
				 AND LOWER(TRIM(COALESCE(a.album_artist, JSON_EXTRACT(a.artists, '$[0]')))) = LOWER(TRIM(COALESCE(b.album_artist, JSON_EXTRACT(b.artists, '$[0]'))))
				 AND ABS(COALESCE(a.duration_ms, 0) - COALESCE(b.duration_ms, 0)) <= 1000
			 )
			 UNION
			 SELECT b.id, b.uid, b.path, b.last_modified, b.title, b.artists, b.album_artist, b.albums, b.genres, b.year, b.rating, b.tags, b.duration_ms, b.bpm, b.key, b.credits, b.label, b.artwork_path
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

	let all_tracks = stmt
		.query_map([], |row| {
			Ok(Track {
				id: row.get(0)?,
				uid: row.get(1)?,
				path: row.get(2)?,
				last_modified: row.get(3)?,
				title: row.get(4)?,
				artists: row.get(5)?,
				album_artist: row.get(6)?,
				albums: row.get(7)?,
				genres: row.get(8)?,
				year: row.get(9)?,
				rating: row.get(10)?,
				tags: row.get(11)?,
				duration_ms: row.get(12)?,
				bpm: row.get(13)?,
				key: row.get(14)?,
				credits: row.get(15)?,
				label: row.get(16)?,
				artwork_blob: None,
				artwork_path: row.get(17)?,
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
			let same_title = a.title.as_deref().map(|s| s.to_lowercase())
				== b.title.as_deref().map(|s| s.to_lowercase());
			let same_artist = a.album_artist.as_deref().map(|s| s.to_lowercase())
				== b.album_artist.as_deref().map(|s| s.to_lowercase());
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

pub fn update_track_metadata(conn: &Connection, id: i64, update: &MetadataUpdate) -> Result<()> {
	if let Some(ref title) = update.title {
		conn.execute("UPDATE tracks SET title = ?1 WHERE id = ?2", params![title, id])?;
	}
	if let Some(ref artists) = update.artists {
		conn.execute("UPDATE tracks SET artists = ?1 WHERE id = ?2", params![artists, id])?;
	}
	if let Some(ref album_artist) = update.album_artist {
		conn.execute("UPDATE tracks SET album_artist = ?1 WHERE id = ?2", params![album_artist, id])?;
	}
	if let Some(ref albums) = update.albums {
		conn.execute("UPDATE tracks SET albums = ?1 WHERE id = ?2", params![albums, id])?;
	}
	if let Some(ref year) = update.year {
		conn.execute("UPDATE tracks SET year = ?1 WHERE id = ?2", params![year, id])?;
	}
	if let Some(ref genres) = update.genres {
		conn.execute("UPDATE tracks SET genres = ?1 WHERE id = ?2", params![genres, id])?;
	}
	if let Some(bpm) = update.bpm {
		conn.execute("UPDATE tracks SET bpm = ?1 WHERE id = ?2", params![bpm, id])?;
	}
	if let Some(rating) = update.rating {
		conn.execute("UPDATE tracks SET rating = ?1 WHERE id = ?2", params![rating, id])?;
	}
	if let Some(ref tags) = update.tags {
		conn.execute("UPDATE tracks SET tags = ?1 WHERE id = ?2", params![tags, id])?;
	}
	if let Some(ref key) = update.key {
		conn.execute("UPDATE tracks SET key = ?1 WHERE id = ?2", params![key, id])?;
	}
	if let Some(ref credits) = update.credits {
		conn.execute("UPDATE tracks SET credits = ?1 WHERE id = ?2", params![credits, id])?;
	}
	if let Some(ref label) = update.label {
		conn.execute("UPDATE tracks SET label = ?1 WHERE id = ?2", params![label, id])?;
	}
	if let Some(ref blob) = update.artwork_blob {
		conn.execute("UPDATE tracks SET artwork_blob = ?1 WHERE id = ?2", params![blob, id])?;
	}
	if let Some(ref path) = update.artwork_path {
		conn.execute("UPDATE tracks SET artwork_path = ?1 WHERE id = ?2", params![path, id])?;
	}
	Ok(())
}

pub fn update_track_metadata_by_uid(conn: &Connection, uid: &str, update: &MetadataUpdate) -> Result<()> {
	if let Some(ref title) = update.title {
		conn.execute("UPDATE tracks SET title = ?1 WHERE uid = ?2", params![title, uid])?;
	}
	if let Some(ref artists) = update.artists {
		conn.execute("UPDATE tracks SET artists = ?1 WHERE uid = ?2", params![artists, uid])?;
	}
	if let Some(ref album_artist) = update.album_artist {
		conn.execute("UPDATE tracks SET album_artist = ?1 WHERE uid = ?2", params![album_artist, uid])?;
	}
	if let Some(ref albums) = update.albums {
		conn.execute("UPDATE tracks SET albums = ?1 WHERE uid = ?2", params![albums, uid])?;
	}
	if let Some(ref year) = update.year {
		conn.execute("UPDATE tracks SET year = ?1 WHERE uid = ?2", params![year, uid])?;
	}
	if let Some(ref genres) = update.genres {
		conn.execute("UPDATE tracks SET genres = ?1 WHERE uid = ?2", params![genres, uid])?;
	}
	if let Some(bpm) = update.bpm {
		conn.execute("UPDATE tracks SET bpm = ?1 WHERE uid = ?2", params![bpm, uid])?;
	}
	if let Some(rating) = update.rating {
		conn.execute("UPDATE tracks SET rating = ?1 WHERE uid = ?2", params![rating, uid])?;
	}
	if let Some(ref tags) = update.tags {
		conn.execute("UPDATE tracks SET tags = ?1 WHERE uid = ?2", params![tags, uid])?;
	}
	if let Some(ref key) = update.key {
		conn.execute("UPDATE tracks SET key = ?1 WHERE uid = ?2", params![key, uid])?;
	}
	if let Some(ref credits) = update.credits {
		conn.execute("UPDATE tracks SET credits = ?1 WHERE uid = ?2", params![credits, uid])?;
	}
	if let Some(ref label) = update.label {
		conn.execute("UPDATE tracks SET label = ?1 WHERE uid = ?2", params![label, uid])?;
	}
	if let Some(ref blob) = update.artwork_blob {
		conn.execute("UPDATE tracks SET artwork_blob = ?1 WHERE uid = ?2", params![blob, uid])?;
	}
	if let Some(ref path) = update.artwork_path {
		conn.execute("UPDATE tracks SET artwork_path = ?1 WHERE uid = ?2", params![path, uid])?;
	}
	Ok(())
}