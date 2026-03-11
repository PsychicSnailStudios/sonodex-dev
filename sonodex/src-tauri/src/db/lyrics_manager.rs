use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lyrics {
	pub id: Option<i64>,
	pub track_id: i64,
	pub source: String,
	pub plain: Option<String>,
	pub synced: Option<String>,
	pub instrumental: bool,
}

pub fn upsert_lyrics(conn: &Connection, lyrics: &Lyrics) -> Result<()> {
	conn.execute(
		"INSERT INTO lyrics (track_id, source, plain, synced, instrumental)
		 VALUES (?1, ?2, ?3, ?4, ?5)
		 ON CONFLICT(track_id) DO UPDATE SET
			source = excluded.source,
			plain = excluded.plain,
			synced = excluded.synced,
			instrumental = excluded.instrumental",
		params![
			lyrics.track_id,
			lyrics.source,
			lyrics.plain,
			lyrics.synced,
			lyrics.instrumental,
		],
	)?;
	Ok(())
}

pub fn get_lyrics(conn: &Connection, track_id: i64) -> Result<Option<Lyrics>> {
	let mut stmt = conn.prepare(
		"SELECT id, track_id, source, plain, synced, instrumental
		 FROM lyrics WHERE track_id = ?1"
	)?;
	let mut rows = stmt.query(params![track_id])?;
	if let Some(row) = rows.next()? {
		Ok(Some(Lyrics {
			id: row.get(0)?,
			track_id: row.get(1)?,
			source: row.get(2)?,
			plain: row.get(3)?,
			synced: row.get(4)?,
			instrumental: row.get(5)?,
		}))
	} else {
		Ok(None)
	}
}

pub fn delete_lyrics(conn: &Connection, track_id: i64) -> Result<()> {
	conn.execute("DELETE FROM lyrics WHERE track_id = ?1", params![track_id])?;
	Ok(())
}
