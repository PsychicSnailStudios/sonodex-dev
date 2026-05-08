use crate::db::{self, Lyrics};
use crate::library_manager;
use crate::state::AppState;
use crate::{open_lib_conn, open_merged_conn};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_track_lyrics(state: State<AppState>, uid: String) -> Result<Option<Lyrics>, String> {
	let profile_uid = state.get_uid();
	let conn = open_merged_conn(&profile_uid);
	let track = db::get_track_by_uid(&conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or("Track not found")?;
	let track_id = track.id.ok_or("Track has no id")?;
	db::lyrics_manager::get_lyrics(&conn, track_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_track_lyrics(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();

	let (title, artist, album, duration_secs) = {
		let conn = open_merged_conn(&profile_uid);
		let track = db::get_track_by_uid(&conn, &uid)
			.map_err(|e| e.to_string())?
			.ok_or("Track not found")?;
		let title = track.title.clone().unwrap_or_default();
		let artist = track
			.album_artist
			.clone()
			.or_else(|| {
				track.artists.as_deref().and_then(|a| {
					serde_json::from_str::<Vec<String>>(a)
						.ok()
						.and_then(|v| v.into_iter().next())
				})
			})
			.unwrap_or_default();
		let album = track.albums.as_deref().and_then(|a| {
			serde_json::from_str::<Vec<serde_json::Value>>(a)
				.ok()
				.and_then(|v| {
					v.into_iter()
						.next()
						.and_then(|e| e["name"].as_str().map(|s| s.to_string()))
				})
		});
		let duration_secs = track.duration_ms.map(|ms| (ms / 1000) as u64);
		(title, artist, album, duration_secs)
	};

	let client = crate::enrichment::make_client()?;
	let result = crate::enrichment::lyrics::fetch_lyrics(
		&client,
		&title,
		&artist,
		album.as_deref(),
		duration_secs,
	)
	.await;

	if let Some(lyrics) = result {
		// Write to the source library
		match library_manager::open_source_conn_for_entity(&profile_uid, &uid, "tracks") {
			Ok((source_conn, lib)) => {
				let track = db::get_track_by_uid(&source_conn, &uid)
					.map_err(|e| e.to_string())?
					.ok_or("Track not found in source")?;
				let track_id = track.id.ok_or("Track has no id")?;
				db::lyrics_manager::upsert_lyrics(
					&source_conn,
					&Lyrics {
						id: None,
						track_id,
						source: lyrics.source,
						plain: lyrics.plain,
						synced: lyrics.synced,
						instrumental: lyrics.instrumental,
					},
				)
				.map_err(|e| e.to_string())?;
				library_manager::incremental_update(&profile_uid, &[lib.uid])
					.map_err(|e| e.to_string())?;
			}
			Err(_) => {
				let conn = open_lib_conn(&profile_uid);
				let track = db::get_track_by_uid(&conn, &uid)
					.map_err(|e| e.to_string())?
					.ok_or("Track not found")?;
				let track_id = track.id.ok_or("Track has no id")?;
				db::lyrics_manager::upsert_lyrics(
					&conn,
					&Lyrics {
						id: None,
						track_id,
						source: lyrics.source,
						plain: lyrics.plain,
						synced: lyrics.synced,
						instrumental: lyrics.instrumental,
					},
				)
				.map_err(|e| e.to_string())?;
			}
		}
		app.emit("lyrics:updated", uid).ok();
	}

	Ok(())
}

#[tauri::command]
pub fn delete_track_lyrics(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	match library_manager::open_source_conn_for_entity(&profile_uid, &uid, "tracks") {
		Ok((source_conn, lib)) => {
			let track = db::get_track_by_uid(&source_conn, &uid)
				.map_err(|e| e.to_string())?
				.ok_or("Track not found")?;
			let track_id = track.id.ok_or("Track has no id")?;
			db::lyrics_manager::delete_lyrics(&source_conn, track_id)
				.map_err(|e| e.to_string())?;
			library_manager::incremental_update(&profile_uid, &[lib.uid])
				.map_err(|e| e.to_string())?;
		}
		Err(_) => {
			let conn = open_lib_conn(&profile_uid);
			let track = db::get_track_by_uid(&conn, &uid)
				.map_err(|e| e.to_string())?
				.ok_or("Track not found")?;
			let track_id = track.id.ok_or("Track has no id")?;
			db::lyrics_manager::delete_lyrics(&conn, track_id).map_err(|e| e.to_string())?;
		}
	}
	Ok(())
}
