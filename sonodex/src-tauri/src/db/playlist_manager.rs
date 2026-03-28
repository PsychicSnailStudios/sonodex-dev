use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistTrackEntry {
	pub uid: String,
	pub name: String,
	pub order: u32,
}

pub fn normalize_playlist_track_order(tracks: &mut Vec<PlaylistTrackEntry>) {
	let any_nonzero = tracks.iter().any(|t| t.order != 0);
	if !any_nonzero {
		for (i, t) in tracks.iter_mut().enumerate() {
			t.order = (i + 1) as u32;
		}
	} else {
		tracks.sort_by_key(|t| t.order);
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
	pub id: Option<i64>,
	pub uid: String,
	pub title: String,
	pub description: Option<String>,
	pub owner: Option<String>,
	pub tracks: Option<String>,
	pub folder: Option<String>,
	pub artwork_blob: Option<Vec<u8>>,
	pub artwork_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistUpdate {
	pub title: Option<String>,
	pub description: Option<String>,
	pub owner: Option<String>,
	pub tracks: Option<String>,
	pub folder: Option<String>,
	pub artwork_blob: Option<Vec<u8>>,
	pub artwork_path: Option<String>,
}

pub fn create_playlist(conn: &Connection, playlist: &Playlist) -> Result<()> {
	conn.execute(
		"INSERT INTO playlists (uid, title, description, owner, tracks, folder, artwork_blob, artwork_path)
		 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
		params![
			playlist.uid,
			playlist.title,
			playlist.description,
			playlist.owner,
			playlist.tracks,
			playlist.folder,
			playlist.artwork_blob,
			playlist.artwork_path
		],
	)?;
	Ok(())
}

pub fn get_all_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
	let mut stmt = conn.prepare(
		"SELECT id, uid, title, description, owner, tracks, folder, artwork_path
		 FROM playlists ORDER BY folder, title",
	)?;
	let playlists = stmt
		.query_map([], |row| {
			Ok(Playlist {
				id: row.get(0)?,
				uid: row.get(1)?,
				title: row.get(2)?,
				description: row.get(3)?,
				owner: row.get(4)?,
				tracks: row.get(5)?,
				folder: row.get(6)?,
				artwork_blob: None,
				artwork_path: row.get(7)?,
			})
		})?
		.collect::<Result<Vec<_>>>()?;
	Ok(playlists)
}

pub fn get_playlist_by_id(conn: &Connection, id: i64) -> Result<Option<Playlist>> {
	let mut stmt = conn.prepare(
		"SELECT id, uid, title, description, owner, tracks, folder, artwork_blob, artwork_path
		 FROM playlists WHERE id = ?1",
	)?;
	let mut rows = stmt.query(params![id])?;
	if let Some(row) = rows.next()? {
		Ok(Some(Playlist {
			id: row.get(0)?,
			uid: row.get(1)?,
			title: row.get(2)?,
			description: row.get(3)?,
			owner: row.get(4)?,
			tracks: row.get(5)?,
			folder: row.get(6)?,
			artwork_blob: row.get(7)?,
			artwork_path: row.get(8)?,
		}))
	} else {
		Ok(None)
	}
}

pub fn get_playlist_by_uid(conn: &Connection, uid: &str) -> Result<Option<Playlist>> {
	let mut stmt = conn.prepare(
		"SELECT id, uid, title, description, owner, tracks, folder, artwork_blob, artwork_path
		 FROM playlists WHERE uid = ?1",
	)?;
	let mut rows = stmt.query(params![uid])?;
	if let Some(row) = rows.next()? {
		Ok(Some(Playlist {
			id: row.get(0)?,
			uid: row.get(1)?,
			title: row.get(2)?,
			description: row.get(3)?,
			owner: row.get(4)?,
			tracks: row.get(5)?,
			folder: row.get(6)?,
			artwork_blob: row.get(7)?,
			artwork_path: row.get(8)?,
		}))
	} else {
		Ok(None)
	}
}

pub fn get_playlist_artwork(conn: &Connection, id: i64) -> Result<Option<Vec<u8>>> {
	let mut stmt = conn.prepare("SELECT artwork_blob FROM playlists WHERE id = ?1")?;
	let mut rows = stmt.query(params![id])?;
	if let Some(row) = rows.next()? {
		Ok(row.get(0)?)
	} else {
		Ok(None)
	}
}

pub fn get_playlist_artwork_by_uid(conn: &Connection, uid: &str) -> Result<Option<Vec<u8>>> {
	let mut stmt = conn.prepare("SELECT artwork_blob FROM playlists WHERE uid = ?1")?;
	let mut rows = stmt.query(params![uid])?;
	if let Some(row) = rows.next()? {
		Ok(row.get(0)?)
	} else {
		Ok(None)
	}
}

pub fn update_playlist(conn: &Connection, id: i64, update: &PlaylistUpdate) -> Result<()> {
	if let Some(ref title) = update.title {
		conn.execute("UPDATE playlists SET title = ?1 WHERE id = ?2", params![title, id])?;
	}
	if let Some(ref description) = update.description {
		conn.execute("UPDATE playlists SET description = ?1 WHERE id = ?2", params![description, id])?;
	}
	if let Some(ref owner) = update.owner {
		conn.execute("UPDATE playlists SET owner = ?1 WHERE id = ?2", params![owner, id])?;
	}
	if let Some(ref tracks) = update.tracks {
		conn.execute("UPDATE playlists SET tracks = ?1 WHERE id = ?2", params![tracks, id])?;
	}
	if let Some(ref folder) = update.folder {
		conn.execute("UPDATE playlists SET folder = ?1 WHERE id = ?2", params![folder, id])?;
	}
	if let Some(ref blob) = update.artwork_blob {
		conn.execute("UPDATE playlists SET artwork_blob = ?1 WHERE id = ?2", params![blob, id])?;
	}
	if let Some(ref path) = update.artwork_path {
		conn.execute("UPDATE playlists SET artwork_path = ?1 WHERE id = ?2", params![path, id])?;
	}
	Ok(())
}

pub fn update_playlist_by_uid(conn: &Connection, uid: &str, update: &PlaylistUpdate) -> Result<()> {
	if let Some(ref title) = update.title {
		conn.execute("UPDATE playlists SET title = ?1 WHERE uid = ?2", params![title, uid])?;
	}
	if let Some(ref description) = update.description {
		conn.execute("UPDATE playlists SET description = ?1 WHERE uid = ?2", params![description, uid])?;
	}
	if let Some(ref owner) = update.owner {
		conn.execute("UPDATE playlists SET owner = ?1 WHERE uid = ?2", params![owner, uid])?;
	}
	if let Some(ref tracks) = update.tracks {
		conn.execute("UPDATE playlists SET tracks = ?1 WHERE uid = ?2", params![tracks, uid])?;
	}
	if let Some(ref folder) = update.folder {
		conn.execute("UPDATE playlists SET folder = ?1 WHERE uid = ?2", params![folder, uid])?;
	}
	if let Some(ref blob) = update.artwork_blob {
		conn.execute("UPDATE playlists SET artwork_blob = ?1 WHERE uid = ?2", params![blob, uid])?;
	}
	if let Some(ref path) = update.artwork_path {
		conn.execute("UPDATE playlists SET artwork_path = ?1 WHERE uid = ?2", params![path, uid])?;
	}
	Ok(())
}

pub fn rename_folder(conn: &Connection, old_path: &str, new_path: &str) -> Result<()> {
	let like_prefix = format!("{}/%", old_path);
	conn.execute(
		"UPDATE playlists SET folder = ?1 WHERE folder = ?2",
		params![new_path, old_path],
	)?;
	conn.execute(
		"UPDATE playlists SET folder = ?1 || SUBSTR(folder, ?2) WHERE folder LIKE ?3",
		params![new_path, old_path.len() as i64 + 1, like_prefix],
	)?;
	Ok(())
}

pub fn move_playlists_to_folder(conn: &Connection, old_folder: &str, new_folder: &str) -> Result<()> {
	let like_prefix = format!("{}/%", old_folder);
	let new_folder_val: Option<&str> = if new_folder.is_empty() { None } else { Some(new_folder) };
	conn.execute(
		"UPDATE playlists SET folder = ?1 WHERE folder = ?2",
		params![new_folder_val, old_folder],
	)?;
	conn.execute(
		"UPDATE playlists SET folder = ?1 || SUBSTR(folder, ?2) WHERE folder LIKE ?3",
		params![new_folder, old_folder.len() as i64 + 1, like_prefix],
	)?;
	Ok(())
}

pub fn delete_playlist(conn: &Connection, id: i64) -> Result<()> {
	conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
	Ok(())
}

pub fn delete_playlist_by_uid(conn: &Connection, uid: &str) -> Result<()> {
	conn.execute("DELETE FROM playlists WHERE uid = ?1", params![uid])?;
	Ok(())
}