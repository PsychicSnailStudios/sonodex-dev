use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use crate::db::generate_uid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scrobble {
	pub uid: String,
	pub timestamp: i64,
	pub track_uid: String,
	pub artist_uid: String,
	pub duration_played: i64,
	pub did_seek: bool,
	pub did_pause: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrobbleUpdate {
	pub duration_played: i64,
	pub did_seek: bool,
	pub did_pause: bool,
}

pub fn init_analytics_db(conn: &Connection) -> Result<()> {
	conn.execute_batch("
		CREATE TABLE IF NOT EXISTS scrobbles (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			uid TEXT NOT NULL UNIQUE,
			timestamp INTEGER NOT NULL,
			track_uid TEXT NOT NULL,
			artist_uid TEXT NOT NULL,
			duration_played INTEGER NOT NULL DEFAULT 0,
			did_seek INTEGER NOT NULL DEFAULT 0,
			did_pause INTEGER NOT NULL DEFAULT 0
		);
		CREATE INDEX IF NOT EXISTS idx_scrobbles_uid ON scrobbles (uid);
		CREATE INDEX IF NOT EXISTS idx_scrobbles_track_uid ON scrobbles (track_uid);
		CREATE INDEX IF NOT EXISTS idx_scrobbles_artist_uid ON scrobbles (artist_uid);
		CREATE INDEX IF NOT EXISTS idx_scrobbles_timestamp ON scrobbles (timestamp);
	")
}

pub fn log_scrobble(conn: &Connection, scrobble: &Scrobble) -> Result<()> {
	conn.execute(
		"INSERT INTO scrobbles (uid, timestamp, track_uid, artist_uid, duration_played, did_seek, did_pause)
		VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
		params![
			scrobble.uid,
			scrobble.timestamp,
			scrobble.track_uid,
			scrobble.artist_uid,
			scrobble.duration_played,
			scrobble.did_seek as i64,
			scrobble.did_pause as i64,
		],
	)?;
	Ok(())
}

pub fn get_all_scrobbles(conn: &Connection) -> Result<Vec<Scrobble>> {
	let mut stmt = conn.prepare(
		"SELECT uid, timestamp, track_uid, artist_uid, duration_played, did_seek, did_pause
		FROM scrobbles
		ORDER BY timestamp DESC"
	)?;
	let rows = stmt.query_map([], |row| {
		Ok(Scrobble {
			uid: row.get(0)?,
			timestamp: row.get(1)?,
			track_uid: row.get(2)?,
			artist_uid: row.get(3)?,
			duration_played: row.get(4)?,
			did_seek: row.get::<_, i64>(5)? != 0,
			did_pause: row.get::<_, i64>(6)? != 0,
		})
	})?;
	rows.collect()
}

pub fn get_scrobbles_for_track(conn: &Connection, track_uid: &str) -> Result<Vec<Scrobble>> {
	let mut stmt = conn.prepare(
		"SELECT uid, timestamp, track_uid, artist_uid, duration_played, did_seek, did_pause
		FROM scrobbles
		WHERE track_uid = ?1
		ORDER BY timestamp DESC"
	)?;
	let rows = stmt.query_map(params![track_uid], |row| {
		Ok(Scrobble {
			uid: row.get(0)?,
			timestamp: row.get(1)?,
			track_uid: row.get(2)?,
			artist_uid: row.get(3)?,
			duration_played: row.get(4)?,
			did_seek: row.get::<_, i64>(5)? != 0,
			did_pause: row.get::<_, i64>(6)? != 0,
		})
	})?;
	rows.collect()
}

pub fn update_scrobble(conn: &Connection, uid: &str, update: &ScrobbleUpdate) -> Result<()> {
	conn.execute(
		"UPDATE scrobbles
		SET duration_played = ?1, did_seek = ?2, did_pause = ?3
		WHERE uid = ?4",
		params![
			update.duration_played,
			update.did_seek as i64,
			update.did_pause as i64,
			uid,
		],
	)?;
	Ok(())
}

pub fn delete_scrobble(conn: &Connection, uid: &str) -> Result<()> {
	conn.execute("DELETE FROM scrobbles WHERE uid = ?1", params![uid])?;
	Ok(())
}

pub fn new_scrobble_uid() -> String {
	generate_uid("sc-")
}
