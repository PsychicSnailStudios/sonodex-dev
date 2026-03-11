mod db;
mod scanner;
mod watcher;
mod enrichment;

use db::{
	add_library_path, get_all_tracks, get_db_path, get_library_paths, init_db, remove_library_path,
	LibraryPath, Track,
	Album, AlbumUpdate, create_album, get_all_albums, get_album_by_id, update_album, delete_album,
	Artist, ArtistUpdate, create_artist, get_all_artists, get_artist_by_id, update_artist, delete_artist,
	Playlist, PlaylistUpdate, create_playlist, get_all_playlists, get_playlist_by_id, update_playlist, delete_playlist,
};
use rusqlite::Connection;
use tauri::{AppHandle, Emitter};

fn open_conn() -> Connection {
	Connection::open(get_db_path()).expect("Failed to open database")
}


// ─────────────────────────────────────────────
// LIBRARY PATHS
// ─────────────────────────────────────────────

#[tauri::command]
fn add_path(app: AppHandle, path: String) -> Result<(), String> {
	let conn = open_conn();
	add_library_path(&conn, &path).map_err(|e| e.to_string())?;

	let app_clone = app.clone();
	let path_clone = path.clone();
	std::thread::spawn(move || {
		let conn = open_conn();
		scanner::scan_directory_with_progress(&conn, &path_clone, &app_clone);
		let paths = get_library_paths(&conn)
			.unwrap_or_default()
			.into_iter()
			.map(|p| p.path)
			.collect();
		watcher::start_watcher(app_clone, paths);
	});

	Ok(())
}

#[tauri::command]
fn remove_path(path: String) -> Result<(), String> {
	let conn = open_conn();
	remove_library_path(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_paths() -> Result<Vec<LibraryPath>, String> {
	let conn = open_conn();
	get_library_paths(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn rescan(app: AppHandle) -> Result<(), String> {
	let conn = open_conn();
	let paths = get_library_paths(&conn).map_err(|e| e.to_string())?;

	let app_clone = app.clone();
	let path_strings: Vec<String> = paths.into_iter().map(|p| p.path).collect();
	std::thread::spawn(move || {
		let conn = open_conn();
		for p in &path_strings {
			scanner::scan_directory_with_progress(&conn, p, &app_clone);
		}
		watcher::start_watcher(app_clone, path_strings);
	});

	Ok(())
}


// ─────────────────────────────────────────────
// TRACKS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_tracks() -> Result<Vec<Track>, String> {
	let conn = open_conn();
	get_all_tracks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_track_artwork(id: i64) -> Result<Option<Vec<u8>>, String> {
	let conn = open_conn();
	db::get_track_artwork(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_duplicates() -> Result<Vec<db::DuplicateGroup>, String> {
	let conn = open_conn();
	db::find_duplicates(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_track_from_library(id: i64) -> Result<(), String> {
	let conn = open_conn();
	db::delete_track_by_id(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_track_file(app: AppHandle, id: i64, path: String) -> Result<(), String> {
	let conn = open_conn();
	std::fs::remove_file(&path).map_err(|e| e.to_string())?;
	db::delete_track_by_id(&conn, id).map_err(|e| e.to_string())?;
	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
fn update_track_metadata(id: i64, update: db::MetadataUpdate) -> Result<(), String> {
	let conn = open_conn();
	db::update_track_metadata(&conn, id, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_track_tags(app: AppHandle, id: i64, path: String, update: db::MetadataUpdate) -> Result<(), String> {
	use lofty::prelude::*;
	use lofty::probe::Probe;

	let conn = open_conn();

	let mut tagged_file = Probe::open(&path)
		.map_err(|e| e.to_string())?
		.guess_file_type()
		.map_err(|e| e.to_string())?
		.read()
		.map_err(|e| e.to_string())?;

	let has_primary = tagged_file.primary_tag().is_some();
	let tag = if has_primary {
		tagged_file.primary_tag_mut()
	} else {
		tagged_file.first_tag_mut()
	}.ok_or("No tag found in file")?;

	if let Some(title) = &update.title {
		tag.set_title(title.clone());
	}
	if let Some(artists) = &update.artists {
		let parsed: Vec<String> = serde_json::from_str(artists).unwrap_or_default();
		tag.set_artist(parsed.join(", "));
	}
	if let Some(album_artist) = &update.album_artist {
		tag.insert(lofty::tag::TagItem::new(
			lofty::tag::ItemKey::AlbumArtist,
			lofty::tag::ItemValue::Text(album_artist.clone()),
		));
	}
	if let Some(albums) = &update.albums {
		let parsed: Vec<serde_json::Value> = serde_json::from_str(albums).unwrap_or_default();
		if let Some(name) = parsed.first().and_then(|a| a.get("name")).and_then(|n| n.as_str()) {
			tag.set_album(name.to_string());
		}
	}
	if let Some(year) = &update.year {
		if let Ok(y) = year.parse::<u32>() {
			tag.set_year(y);
		}
	}
	if let Some(genres) = &update.genres {
		let parsed: Vec<String> = serde_json::from_str(genres).unwrap_or_default();
		tag.set_genre(parsed.join("/"));
	}
	if let Some(bpm) = update.bpm {
		tag.insert(lofty::tag::TagItem::new(
			lofty::tag::ItemKey::Bpm,
			lofty::tag::ItemValue::Text(bpm.to_string()),
		));
	}

	tagged_file.save_to_path(&path, lofty::config::WriteOptions::default())
		.map_err(|e| e.to_string())?;

	db::update_track_metadata(&conn, id, &update).map_err(|e| e.to_string())?;
	app.emit("library:updated", ()).ok();
	Ok(())
}


// ─────────────────────────────────────────────
// ALBUMS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_albums() -> Result<Vec<Album>, String> {
	let conn = open_conn();
	get_all_albums(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_album(id: i64) -> Result<Option<Album>, String> {
	let conn = open_conn();
	get_album_by_id(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_album_artwork(id: i64) -> Result<Option<Vec<u8>>, String> {
	let conn = open_conn();
	db::get_album_artwork(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_album_entry(album: Album) -> Result<(), String> {
	let conn = open_conn();
	create_album(&conn, &album).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_album_entry(id: i64, update: AlbumUpdate) -> Result<(), String> {
	let conn = open_conn();
	update_album(&conn, id, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_album_entry(id: i64) -> Result<(), String> {
	let conn = open_conn();
	delete_album(&conn, id).map_err(|e| e.to_string())
}


// ─────────────────────────────────────────────
// ARTISTS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_artists() -> Result<Vec<Artist>, String> {
	let conn = open_conn();
	get_all_artists(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_artist(id: i64) -> Result<Option<Artist>, String> {
	let conn = open_conn();
	get_artist_by_id(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_artist_profile_art(id: i64) -> Result<Option<Vec<u8>>, String> {
	let conn = open_conn();
	db::get_artist_profile_art(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_artist_banner_art(id: i64) -> Result<Option<Vec<u8>>, String> {
	let conn = open_conn();
	db::get_artist_banner_art(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_artist_entry(artist: Artist) -> Result<(), String> {
	let conn = open_conn();
	create_artist(&conn, &artist).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_artist_entry(id: i64, update: ArtistUpdate) -> Result<(), String> {
	let conn = open_conn();
	update_artist(&conn, id, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_artist_entry(id: i64) -> Result<(), String> {
	let conn = open_conn();
	delete_artist(&conn, id).map_err(|e| e.to_string())
}


// ─────────────────────────────────────────────
// PLAYLISTS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_playlists() -> Result<Vec<Playlist>, String> {
	let conn = open_conn();
	get_all_playlists(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlist(id: i64) -> Result<Option<Playlist>, String> {
	let conn = open_conn();
	get_playlist_by_id(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlist_artwork(id: i64) -> Result<Option<Vec<u8>>, String> {
	let conn = open_conn();
	db::get_playlist_artwork(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_playlist_entry(playlist: Playlist) -> Result<(), String> {
	let conn = open_conn();
	create_playlist(&conn, &playlist).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_playlist_entry(id: i64, update: PlaylistUpdate) -> Result<(), String> {
	let conn = open_conn();
	update_playlist(&conn, id, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_playlist_entry(id: i64) -> Result<(), String> {
	let conn = open_conn();
	delete_playlist(&conn, id).map_err(|e| e.to_string())
}


// ─────────────────────────────────────────────
// SETTINGS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_settings() -> Result<Vec<db::Setting>, String> {
	let conn = open_conn();
	db::get_all_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_setting(key: String, value: String) -> Result<(), String> {
	let conn = open_conn();
	db::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}


// ─────────────────────────────────────────────
// ENRICHMENT
// ─────────────────────────────────────────────

fn load_enrich_settings(conn: &Connection) -> enrichment::EnrichSettings {
	let g = |key: &str, default: &str| {
		db::get_setting(conn, key).ok().flatten().unwrap_or_else(|| default.to_string())
	};
	enrichment::EnrichSettings {
		primary_api: g("enrich_primary_api", "musicbrainz"),
		audiodb_key: g("api_audiodb_key", ""),
		priority_title: g("enrich_priority_title", "local"),
		priority_artists: g("enrich_priority_artists", "local"),
		priority_album_artist: g("enrich_priority_album_artist", "local"),
		priority_album: g("enrich_priority_album", "local"),
		priority_year: g("enrich_priority_year", "local"),
		priority_genres: g("enrich_priority_genres", "local"),
		priority_bpm: g("enrich_priority_bpm", "local"),
		priority_key: g("enrich_priority_key", "local"),
		priority_artwork: g("enrich_priority_artwork", "local"),
	}
}

#[tauri::command]
async fn enrich_track(app: AppHandle, id: i64) -> Result<(), String> {
	let (track_input, settings) = {
		let conn = open_conn();
		let tracks = db::get_all_tracks(&conn).map_err(|e| e.to_string())?;
		let track = tracks.into_iter().find(|t| t.id == Some(id))
			.ok_or("Track not found")?;
		let artwork = db::get_track_artwork(&conn, id).ok().flatten();
		let settings = load_enrich_settings(&conn);
		let input = enrichment::TrackInput {
			id,
			title: track.title,
			artists: track.artists,
			album_artist: track.album_artist,
			albums: track.albums,
			year: track.year,
			genres: track.genres,
			bpm: track.bpm,
			key: track.key,
			existing_artwork: artwork,
		};
		(input, settings)
	};

	let client = enrichment::make_client()?;
	let result = enrichment::enrich_track_async(&client, &track_input, &settings).await?;

	{
		let conn = open_conn();
		let update = db::MetadataUpdate {
			title: result.title,
			artists: result.artists,
			album_artist: result.album_artist,
			albums: result.albums,
			year: result.year,
			genres: result.genres,
			bpm: result.bpm,
			rating: None,
			tags: None,
			key: result.key,
			credits: None,
			label: None,
			artwork_blob: result.artwork,
			artwork_path: None,
		};
		db::update_track_metadata(&conn, id, &update).map_err(|e| e.to_string())?;
	}

	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn enrich_all(app: AppHandle) -> Result<(), String> {
	let (track_inputs, settings) = {
		let conn = open_conn();
		let tracks = db::get_all_tracks(&conn).map_err(|e| e.to_string())?;
		let settings = load_enrich_settings(&conn);
		let inputs: Vec<enrichment::TrackInput> = tracks.into_iter().filter_map(|t| {
			let id = t.id?;
			let artwork = db::get_track_artwork(&conn, id).ok().flatten();
			Some(enrichment::TrackInput {
				id,
				title: t.title,
				artists: t.artists,
				album_artist: t.album_artist,
				albums: t.albums,
				year: t.year,
				genres: t.genres,
				bpm: t.bpm,
				key: t.key,
				existing_artwork: artwork,
			})
		}).collect();
		(inputs, settings)
	};

	let total = track_inputs.len();
	let mut done = 0usize;
	let mut errors = 0usize;

	app.emit("enrich:progress", serde_json::json!({ "done": 0, "total": total, "errors": 0 })).ok();

	let client = enrichment::make_client()?;

	for track_input in &track_inputs {
		let id = track_input.id;
		match enrichment::enrich_track_async(&client, track_input, &settings).await {
			Ok(result) => {
				let conn = open_conn();
				let update = db::MetadataUpdate {
					title: result.title,
					artists: result.artists,
					album_artist: result.album_artist,
					albums: result.albums,
					year: result.year,
					genres: result.genres,
					bpm: result.bpm,
					rating: None,
					tags: None,
					key: result.key,
					credits: None,
					label: None,
					artwork_blob: result.artwork,
					artwork_path: None,
				};
				db::update_track_metadata(&conn, id, &update).ok();
			}
			Err(_) => errors += 1,
		}
		done += 1;
		if done % 5 == 0 || done == total {
			app.emit("enrich:progress", serde_json::json!({ "done": done, "total": total, "errors": errors })).ok();
		}
	}

	app.emit("enrich:done", serde_json::json!({ "total": total, "errors": errors })).ok();
	app.emit("library:updated", ()).ok();
	Ok(())
}


// ─────────────────────────────────────────────
// ENTRY POINT
// ─────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let conn = open_conn();
	init_db(&conn).expect("Failed to initialize database");

	tauri::Builder::default()
		.plugin(tauri_plugin_log::Builder::new().build())
		.plugin(tauri_plugin_dialog::init())
		.setup(|app| {
			let handle = app.handle().clone();
			let conn = open_conn();
			let paths = db::get_library_paths(&conn)
				.unwrap_or_default()
				.into_iter()
				.map(|p| p.path)
				.collect();
			watcher::start_watcher(handle, paths);
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			add_path,
			remove_path,
			get_paths,
			rescan,
			get_tracks,
			get_track_artwork,
			get_duplicates,
			remove_track_from_library,
			delete_track_file,
			update_track_metadata,
			write_track_tags,
			get_albums,
			get_album,
			get_album_artwork,
			create_album_entry,
			update_album_entry,
			delete_album_entry,
			get_artists,
			get_artist,
			get_artist_profile_art,
			get_artist_banner_art,
			create_artist_entry,
			update_artist_entry,
			delete_artist_entry,
			get_playlists,
			get_playlist,
			get_playlist_artwork,
			create_playlist_entry,
			update_playlist_entry,
			delete_playlist_entry,
			get_settings,
			save_setting,
			enrich_track,
			enrich_all,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}