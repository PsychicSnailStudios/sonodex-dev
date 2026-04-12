mod db;
mod enrichment;
mod profiles;
mod scanner;
mod state;
mod watcher;
mod connections;

use db::{
	add_library_path, create_album, create_artist, create_playlist, delete_album_by_uid,
	delete_artist_by_uid, delete_lyrics, delete_playlist_by_uid, get_album_by_uid, get_all_albums,
	get_all_artists, get_all_playlists, get_all_tracks, get_artist_by_uid, get_library_paths,
	get_lyrics, get_playlist_by_uid, init_lib_db, init_settings_db, remove_library_path,
	update_album_by_uid, update_artist_by_uid, update_playlist_by_uid, upsert_track, upsert_lyrics, Album,
	AlbumUpdate, Artist, ArtistUpdate, LibraryPath, Lyrics, Playlist, PlaylistUpdate, Track,
};

use crate::connections::lastfm_auth;
use crate::connections::spotify_auth;
use crate::db::{MetadataUpdate, Setting};

use profiles::{
	create_profile as new_profile, get_lib_db_path, get_settings_db_path, read_registry,
	write_registry, Profile,
};
use rusqlite::Connection;
use state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;

pub fn open_settings_conn(uid: &str) -> Connection {
	Connection::open(get_settings_db_path(uid)).expect("Failed to open settings database")
}

pub fn open_analytics_conn(uid: &str) -> Result<Connection, rusqlite::Error> {
	let path = crate::profiles::get_profile_dir(uid).join("analytics.db");
	let conn = Connection::open(path)?;
	db::analytics_manager::init_analytics_db(&conn)?;
	Ok(conn)
}

pub fn open_lib_conn(uid: &str) -> Connection {
	let conn = Connection::open(get_lib_db_path(uid)).expect("Failed to open lib database");
	let settings_path = get_settings_db_path(uid);
	conn.execute_batch(&format!(
		"ATTACH DATABASE '{}' AS settings;",
		settings_path.to_string_lossy().replace('\'', "''")
	))
	.ok();
	conn
}

// ─────────────────────────────────────────────
// PROFILES
// ─────────────────────────────────────────────

#[tauri::command]
fn needs_profile_setup() -> bool {
	read_registry().profiles.is_empty()
}

#[tauri::command]
fn get_profiles() -> Result<Vec<Profile>, String> {
	let mut profiles = read_registry().profiles;
	for p in &mut profiles {
		p.avatar_blob = None;
	}
	Ok(profiles)
}

#[tauri::command]
fn get_active_profile(state: State<AppState>) -> Result<Profile, String> {
	let uid = state.get_uid();
	read_registry()
		.profiles
		.into_iter()
		.find(|p| p.uid == uid)
		.map(|mut p| {
			p.avatar_blob = None;
			p
		})
		.ok_or("Active profile not found".to_string())
}

#[tauri::command]
fn get_profile_avatar(uid: String) -> Result<Option<Vec<u8>>, String> {
	Ok(read_registry()
		.profiles
		.into_iter()
		.find(|p| p.uid == uid)
		.and_then(|p| p.avatar_blob))
}

#[tauri::command]
fn create_profile_cmd(
	app: AppHandle,
	state: State<AppState>,
	name: String,
	avatar_blob: Option<Vec<u8>>,
	copy_paths_from: Option<String>,
) -> Result<Profile, String> {
	let profile = new_profile(&name, avatar_blob);

	let settings_conn = open_settings_conn(&profile.uid);
	init_settings_db(&settings_conn).map_err(|e| e.to_string())?;

	let lib_conn = Connection::open(get_lib_db_path(&profile.uid)).map_err(|e| e.to_string())?;
	init_lib_db(&lib_conn).map_err(|e| e.to_string())?;

	if let Some(source_uid) = copy_paths_from {
		let source_conn = open_settings_conn(&source_uid);
		let paths = get_library_paths(&source_conn).unwrap_or_default();
		for p in paths {
			add_library_path(&settings_conn, &p.path).ok();
		}
	}

	let mut registry = read_registry();
	let is_first = registry.profiles.is_empty();
	let is_only = registry.profiles.iter().all(|p| p.uid == state.get_uid()) && registry.profiles.len() <= 1;
	registry.profiles.push(profile.clone());
	if is_first || is_only {
		registry.active = profile.uid.clone();
		state.set_uid(profile.uid.clone());
	}
	write_registry(&registry);

	if is_first {
		app.emit("profile:ready", ()).ok();
	}

	Ok(profile)
}

#[tauri::command]
fn update_profile_cmd(
	uid: String,
	name: Option<String>,
	avatar_blob: Option<Vec<u8>>,
) -> Result<(), String> {
	let mut registry = read_registry();
	if let Some(profile) = registry.profiles.iter_mut().find(|p| p.uid == uid) {
		if let Some(n) = name {
			profile.name = n;
		}
		if avatar_blob.is_some() {
			profile.avatar_blob = avatar_blob;
		}
	}
	write_registry(&registry);
	Ok(())
}

#[tauri::command]
fn delete_profile_cmd(uid: String, state: State<AppState>) -> Result<(), String> {
	if state.get_uid() == uid {
		return Err("Cannot delete the active profile".to_string());
	}
	let mut registry = read_registry();
	registry.profiles.retain(|p| p.uid != uid);
	write_registry(&registry);
	let profile_dir = crate::profiles::get_profile_dir(&uid);
	std::fs::remove_dir_all(profile_dir).ok();
	Ok(())
}

#[tauri::command]
fn switch_profile(uid: String, state: State<AppState>) -> Result<(), String> {
	let registry = read_registry();
	if !registry.profiles.iter().any(|p| p.uid == uid) {
		return Err("Profile not found".to_string());
	}
	let mut registry = registry;
	registry.active = uid.clone();
	write_registry(&registry);
	state.set_uid(uid);
	Ok(())
}

// ─────────────────────────────────────────────
// LIBRARY PATHS
// ─────────────────────────────────────────────

fn normalize_path(path: &str) -> String {
	path.replace('\\', "/")
}

#[tauri::command]
fn add_path(app: AppHandle, state: State<AppState>, path: String) -> Result<(), String> {
	let path = normalize_path(&path);
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);
	add_library_path(&settings_conn, &path).map_err(|e| e.to_string())?;

	let app_clone = app.clone();
	let path_clone = path.clone();
	let uid_clone = uid.clone();
	std::thread::spawn(move || {
		let lib_conn = open_lib_conn(&uid_clone);
		scanner::scan_directory_with_progress(&lib_conn, &path_clone, &app_clone);

		let settings_conn = open_settings_conn(&uid_clone);
		let auto_tracks = db::get_setting(&settings_conn, "auto_enrich_tracks")
			.ok().flatten().map(|v| v == "true").unwrap_or(false);
		let auto_albums = db::get_setting(&settings_conn, "auto_enrich_albums")
			.ok().flatten().map(|v| v == "true").unwrap_or(false);
		let auto_artists = db::get_setting(&settings_conn, "auto_enrich_artists")
			.ok().flatten().map(|v| v == "true").unwrap_or(false);

		if auto_tracks || auto_albums || auto_artists {
			let app_enrich = app_clone.clone();
			let uid_enrich = uid_clone.clone();
			tauri::async_runtime::spawn(async move {
				let state = app_enrich.state::<AppState>();
				if auto_tracks {
					let _ = enrich_all(app_enrich.clone(), state.clone()).await;
				}
				if auto_albums {
					let _ = enrich_all_albums(app_enrich.clone(), state.clone()).await;
				}
				if auto_artists {
					let _ = enrich_all_artists(app_enrich.clone(), state.clone()).await;
				}
				let _ = uid_enrich;
			});
		}

		let settings_conn2 = open_settings_conn(&uid_clone);
		let paths = get_library_paths(&settings_conn2)
			.unwrap_or_default()
			.into_iter()
			.map(|p| p.path)
			.collect();
		watcher::start_watcher(app_clone, uid_clone, paths);
	});

	Ok(())
}

#[tauri::command]
fn remove_path(state: State<AppState>, path: String) -> Result<(), String> {
	let path = normalize_path(&path);
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);
	remove_library_path(&settings_conn, &path).map_err(|e| e.to_string())?;
	let lib_conn = Connection::open(get_lib_db_path(&uid)).map_err(|e| e.to_string())?;
	lib_conn
		.execute(
			"DELETE FROM tracks WHERE path LIKE ?1",
			rusqlite::params![format!("{}%", path)],
		)
		.map_err(|e| e.to_string())?;
	lib_conn
		.execute(
			"DELETE FROM albums WHERE uid NOT IN (
				SELECT DISTINCT json_each.value
				FROM tracks, json_each(tracks.albums, '$[*].uid')
				WHERE json_each.value != ''
			)",
			[],
		)
		.map_err(|e| e.to_string())?;
	lib_conn
		.execute(
			"DELETE FROM artists WHERE name NOT IN (
				SELECT DISTINCT json_each.value
				FROM tracks, json_each(tracks.artists)
			) AND name NOT IN (
				SELECT DISTINCT album_artist FROM tracks WHERE album_artist IS NOT NULL
			)",
			[],
		)
		.map_err(|e| e.to_string())?;
	Ok(())
}

#[tauri::command]
fn get_paths(state: State<AppState>) -> Result<Vec<LibraryPath>, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	get_library_paths(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn rescan(app: AppHandle, state: State<AppState>) -> Result<(), String> {
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);
	let paths = get_library_paths(&settings_conn).map_err(|e| e.to_string())?;

	let app_clone = app.clone();
	let uid_clone = uid.clone();
	let path_strings: Vec<String> = paths.into_iter().map(|p| p.path).collect();
	std::thread::spawn(move || {
		let lib_conn = open_lib_conn(&uid_clone);
		for p in &path_strings {
			scanner::scan_directory_with_progress(&lib_conn, p, &app_clone);
		}
		watcher::start_watcher(app_clone, uid_clone, path_strings);
	});

	Ok(())
}

// ─────────────────────────────────────────────
// TRACKS
// ─────────────────────────────────────────────

#[tauri::command]
fn add_track(state: State<AppState>, track: Track) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	upsert_track(&conn, &track).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_tracks(state: State<AppState>) -> Result<Vec<Track>, String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	get_all_tracks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_track(state: State<AppState>, uid: String) -> Result<Track, String> {
	let tracks_uid = state.get_uid();
	let conn = open_lib_conn(&tracks_uid);
	let track = db::get_track_by_uid(&conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or("Track not found")?;
	Ok(track)
}

#[tauri::command]
fn get_track_artwork(state: State<AppState>, uid: String) -> Result<Option<Vec<u8>>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	db::get_track_artwork_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_duplicates(state: State<AppState>) -> Result<Vec<db::DuplicateGroup>, String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	db::find_duplicates(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_track_from_library(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	db::delete_track_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_track_file(
	app: AppHandle,
	state: State<AppState>,
	uid: String,
	path: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	std::fs::remove_file(&path).map_err(|e| e.to_string())?;
	db::delete_track_by_uid(&conn, &uid).map_err(|e| e.to_string())?;
	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
fn update_track_metadata(
	state: State<AppState>,
	uid: String,
	update: db::MetadataUpdate,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	db::update_track_metadata_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_track_tags(
	app: AppHandle,
	state: State<AppState>,
	uid: String,
	path: String,
	update: db::MetadataUpdate,
) -> Result<(), String> {
	use lofty::prelude::*;
	use lofty::probe::Probe;

	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);

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
	}
	.ok_or("No tag found in file")?;

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
		if let Some(name) = parsed
			.first()
			.and_then(|a| a.get("name"))
			.and_then(|n| n.as_str())
		{
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

	tagged_file
		.save_to_path(&path, lofty::config::WriteOptions::default())
		.map_err(|e| e.to_string())?;

	db::update_track_metadata_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())?;
	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn replace_track_path(
	app_handle: tauri::AppHandle,
	state: State<'_, AppState>,
	uid: String,
	new_path: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let lib_conn = open_lib_conn(&profile_uid);
	let settings_conn = open_settings_conn(&profile_uid);

	let new_path = normalize_path(&new_path);

	lib_conn
		.execute("UPDATE tracks SET path = ?1 WHERE uid = ?2", [&new_path, &uid])
		.map_err(|e| e.to_string())?;

	let get = |key: &str, default: &str| -> String {
		db::settings_manager::get_setting(&settings_conn, key)
			.ok()
			.flatten()
			.unwrap_or_else(|| default.to_string())
	};

	let priority_title  = get("filename_priority_title",  "tag");
	let priority_artist = get("filename_priority_artist", "tag");
	let priority_album  = get("filename_priority_album",  "tag");
	let priority_year   = get("filename_priority_year",   "tag");
	let custom_pattern  = get("filename_custom_pattern",  "");

	let tag_delim_raw      = get("artist_tag_delimiters",      " / |; ");
	let filename_delim_raw = get("artist_filename_delimiters", " / |; | feat. | ft. | featuring ");
	let genre_delim_raw    = get("genre_delimiters",           " / |; |, ");

	let tag_delims:      Vec<&str> = tag_delim_raw.split('|').collect();
	let filename_delims: Vec<&str> = filename_delim_raw.split('|').collect();
	let genre_delims:    Vec<&str> = genre_delim_raw.split('|').collect();

	let try_ampersand = get("scan_try_parse_ampersand", "true") == "true";

	let path = std::path::Path::new(&new_path);
	if let Some(fresh_track) = scanner::read_track_with_settings(
		path,
		&priority_title,
		&priority_artist,
		&priority_album,
		&priority_year,
		&custom_pattern,
		&tag_delims,
		&filename_delims,
		&genre_delims,
		try_ampersand,
	) {
		let merged = MetadataUpdate {
			title:        fresh_track.title,
			artists:      fresh_track.artists,
			album_artist: fresh_track.album_artist,
			albums:       fresh_track.albums,
			year:         fresh_track.year,
			genres:       fresh_track.genres,
			bpm:          fresh_track.bpm,
			rating:       fresh_track.rating,
			tags:         None,
			key:          fresh_track.key,
			credits:      fresh_track.credits,
			label:        fresh_track.label,
			artwork_blob: fresh_track.artwork_blob,
			artwork_path: None,
			user_options: None,
		};
		db::track_manager::update_track_metadata_by_uid(&lib_conn, &uid, &merged)
			.map_err(|e| e.to_string())?;
	}

	app_handle
		.emit("library:updated", ())
		.map_err(|e| e.to_string())?;

	Ok(())
}

// ─────────────────────────────────────────────
// ALBUMS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_albums(state: State<AppState>) -> Result<Vec<Album>, String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	get_all_albums(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_album(state: State<AppState>, uid: String) -> Result<Option<Album>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	get_album_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_album_artwork(state: State<AppState>, uid: String) -> Result<Option<Vec<u8>>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	db::get_album_artwork_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_album_entry(state: State<AppState>, album: Album) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	create_album(&conn, &album).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_album_entry(
	state: State<AppState>,
	uid: String,
	update: AlbumUpdate,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	update_album_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_album_entry(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	delete_album_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// ARTISTS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_artists(state: State<AppState>) -> Result<Vec<Artist>, String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	get_all_artists(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_artist(state: State<AppState>, uid: String) -> Result<Option<Artist>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	get_artist_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_artist_profile_art(state: State<AppState>, uid: String) -> Result<Option<Vec<u8>>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	db::get_artist_profile_art_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_artist_banner_art(state: State<AppState>, uid: String) -> Result<Option<Vec<u8>>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	db::get_artist_banner_art_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_artist_entry(state: State<AppState>, artist: Artist) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	create_artist(&conn, &artist).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_artist_entry(
	state: State<AppState>,
	uid: String,
	update: ArtistUpdate,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	update_artist_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_artist_entry(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	delete_artist_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// PLAYLISTS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_playlists(state: State<AppState>) -> Result<Vec<Playlist>, String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	get_all_playlists(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlist(state: State<AppState>, uid: String) -> Result<Option<Playlist>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	get_playlist_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlist_artwork(state: State<AppState>, uid: String) -> Result<Option<Vec<u8>>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	db::get_playlist_artwork_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_playlist_entry(state: State<AppState>, playlist: Playlist) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	create_playlist(&conn, &playlist).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_playlist_entry(
	state: State<AppState>,
	uid: String,
	update: PlaylistUpdate,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	update_playlist_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_playlist_entry(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	delete_playlist_by_uid(&conn, &uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_playlist_folder(
	state: State<AppState>,
	old_path: String,
	new_path: String,
) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	db::rename_folder(&conn, &old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn move_playlist_folder(
	state: State<AppState>,
	old_folder: String,
	new_folder: String,
) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_lib_conn(&uid);
	db::move_playlists_to_folder(&conn, &old_folder, &new_folder).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// LYRICS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_track_lyrics(state: State<AppState>, uid: String) -> Result<Option<Lyrics>, String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	let track = db::get_track_by_uid(&conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or("Track not found")?;
	let track_id = track.id.ok_or("Track has no id")?;
	get_lyrics(&conn, track_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn fetch_track_lyrics(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
) -> Result<(), String> {
	use enrichment::lyrics::fetch_lyrics;

	let profile_uid = state.get_uid();

	let (track_id, title, artist, album, duration_secs) = {
		let conn = open_lib_conn(&profile_uid);
		let track = db::get_track_by_uid(&conn, &uid)
			.map_err(|e| e.to_string())?
			.ok_or("Track not found")?;
		let track_id = track.id.ok_or("Track has no id")?;
		let artist = track.album_artist.clone().or_else(|| {
			track.artists.as_deref().and_then(|a| {
				serde_json::from_str::<Vec<String>>(a)
					.ok()
					.and_then(|v| v.into_iter().next())
			})
		});
		let album = track.albums.as_deref().and_then(|a| {
			serde_json::from_str::<Vec<serde_json::Value>>(a)
				.ok()
				.and_then(|v| v.into_iter().next())
				.and_then(|e| e["name"].as_str().map(|s| s.to_string()))
		});
		let duration_secs = track.duration_ms.map(|ms| (ms / 1000) as u64);
		(track_id, track.title, artist, album, duration_secs)
	};

	let title = title.ok_or("Track has no title")?;
	let artist = artist.ok_or("Track has no artist")?;

	let client = enrichment::make_client()?;
	let result = fetch_lyrics(&client, &title, &artist, album.as_deref(), duration_secs).await;

	if let Some(lyrics) = result {
		let conn = open_lib_conn(&profile_uid);
		upsert_lyrics(
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
		app.emit("lyrics:updated", uid).ok();
	}

	Ok(())
}

#[tauri::command]
fn delete_track_lyrics(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);
	let track = db::get_track_by_uid(&conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or("Track not found")?;
	let track_id = track.id.ok_or("Track has no id")?;
	delete_lyrics(&conn, track_id).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// SETTINGS
// ─────────────────────────────────────────────

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<Vec<db::Setting>, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::get_all_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_uid_remap(
	state: State<AppState>,
	old_uid: String,
	new_uid: String,
	entity_type: String,
) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::settings_manager::add_uid_remap(&conn, &old_uid, &new_uid, &entity_type)
		.map_err(|e| e.to_string())
}
 
#[tauri::command]
fn resolve_uid(state: State<AppState>, uid: String) -> Result<String, String> {
	let profile_uid = state.get_uid();
	let conn = open_settings_conn(&profile_uid);
	db::settings_manager::resolve_uid(&conn, &uid).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// ENRICHMENT
// ─────────────────────────────────────────────

fn load_enrich_settings(conn: &Connection) -> enrichment::EnrichSettings {
	let g = |key: &str, default: &str| {
		db::get_setting(conn, key)
			.ok()
			.flatten()
			.unwrap_or_else(|| default.to_string())
	};
	enrichment::EnrichSettings {
		primary_api: g("enrich_primary_api", "musicbrainz"),
		lastfm_key: g("api_lastfm_key", ""),
		discogs_key: g("api_discogs_key", ""),
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
async fn enrich_track(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();

	let (track_input, numeric_id, settings) = {
		let lib_conn = open_lib_conn(&profile_uid);
		let settings_conn = open_settings_conn(&profile_uid);
		let track = db::get_track_by_uid(&lib_conn, &uid)
			.map_err(|e| e.to_string())?
			.ok_or("Track not found")?;
		let numeric_id = track.id.ok_or("Track has no id")?;
		let artwork = db::get_track_artwork(&lib_conn, numeric_id).ok().flatten();
		let settings = load_enrich_settings(&settings_conn);
		let input = enrichment::TrackInput {
			id: numeric_id,
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
		(input, numeric_id, settings)
	};

	let client = enrichment::make_client()?;
	let result = enrichment::enrich_track_async(&client, &track_input, &settings).await?;

	{
		let conn = open_lib_conn(&profile_uid);
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
			user_options: None,
		};
		db::update_track_metadata(&conn, numeric_id, &update).map_err(|e| e.to_string())?;
	}

	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn enrich_all(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
	let profile_uid = state.get_uid();

	let (track_inputs, settings) = {
		let lib_conn = open_lib_conn(&profile_uid);
		let settings_conn = open_settings_conn(&profile_uid);
		let tracks = get_all_tracks(&lib_conn).map_err(|e| e.to_string())?;
		let settings = load_enrich_settings(&settings_conn);
		let inputs: Vec<enrichment::TrackInput> = tracks
			.into_iter()
			.filter_map(|t| {
				let id = t.id?;
				let artwork = db::get_track_artwork(&lib_conn, id).ok().flatten();
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
			})
			.collect();
		(inputs, settings)
	};

	let total = track_inputs.len();
	let mut done = 0usize;
	let mut errors = 0usize;

	app.emit(
		"enrich:progress",
		serde_json::json!({ "done": 0, "total": total, "errors": 0 }),
	)
	.ok();

	let client = enrichment::make_client()?;

	for track_input in &track_inputs {
		let id = track_input.id;
		match enrichment::enrich_track_async(&client, track_input, &settings).await {
			Ok(result) => {
				let conn = open_lib_conn(&profile_uid);
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
					user_options: None,
				};
				db::update_track_metadata(&conn, id, &update).ok();
			}
			Err(_) => errors += 1,
		}
		done += 1;
		if done % 5 == 0 || done == total {
			app.emit(
				"enrich:progress",
				serde_json::json!({ "done": done, "total": total, "errors": errors }),
			)
			.ok();
		}
	}

	app.emit(
		"enrich:done",
		serde_json::json!({ "total": total, "errors": errors }),
	)
	.ok();
	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn enrich_album(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();

	let (album, settings) = {
		let lib_conn = open_lib_conn(&profile_uid);
		let settings_conn = open_settings_conn(&profile_uid);
		let album = get_album_by_uid(&lib_conn, &uid)
			.map_err(|e| e.to_string())?
			.ok_or("Album not found")?;
		let settings = load_enrich_settings(&settings_conn);
		(album, settings)
	};

	let artist = album
		.album_artist
		.clone()
		.or_else(|| {
			album.artists.as_deref().and_then(|a| {
				serde_json::from_str::<Vec<String>>(a)
					.ok()
					.and_then(|v| v.into_iter().next())
			})
		})
		.ok_or("Album has no artist")?;

	let client = enrichment::make_client()?;
	let result = enrichment::enrich_album_async(&client, &album.title, &artist, &settings).await;

	{
		let lib_conn = open_lib_conn(&profile_uid);
		let update = db::AlbumUpdate {
			title: None,
			format: result.format,
			rating: None,
			artists: None,
			album_artist: None,
			release_date: result.release_date,
			tags: None,
			genres: result.genres,
			tracks: None,
			credits: result.description,
			label: result.label,
			artwork_blob: result.artwork,
			artwork_path: None,
		};
		update_album_by_uid(&lib_conn, &uid, &update).map_err(|e| e.to_string())?;
	}

	let track_uids: Vec<String> = album
		.tracks
		.as_deref()
		.and_then(|t| serde_json::from_str::<Vec<serde_json::Value>>(t).ok())
		.unwrap_or_default()
		.into_iter()
		.filter_map(|e| e["uid"].as_str().map(|s| s.to_string()))
		.filter(|s| !s.is_empty())
		.collect();

	for track_uid in track_uids {
		let (track_input, numeric_id) = {
			let lib_conn = open_lib_conn(&profile_uid);
			let track = match db::get_track_by_uid(&lib_conn, &track_uid) {
				Ok(Some(t)) => t,
				_ => continue,
			};
			let numeric_id = match track.id {
				Some(id) => id,
				None => continue,
			};
			let artwork = db::get_track_artwork(&lib_conn, numeric_id).ok().flatten();
			let input = enrichment::TrackInput {
				id: numeric_id,
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
			(input, numeric_id)
		};

		if let Ok(track_result) =
			enrichment::enrich_track_async(&client, &track_input, &settings).await
		{
			let lib_conn = open_lib_conn(&profile_uid);
			let update = db::MetadataUpdate {
				title: track_result.title,
				artists: track_result.artists,
				album_artist: track_result.album_artist,
				albums: track_result.albums,
				year: track_result.year,
				genres: track_result.genres,
				bpm: track_result.bpm,
				rating: None,
				tags: None,
				key: track_result.key,
				credits: None,
				label: None,
				artwork_blob: track_result.artwork,
				artwork_path: None,
				user_options: None,
			};
			db::update_track_metadata(&lib_conn, numeric_id, &update).ok();
		}
	}

	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn enrich_artist(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();

	let (artist, settings) = {
		let lib_conn = open_lib_conn(&profile_uid);
		let settings_conn = open_settings_conn(&profile_uid);
		let artist = get_artist_by_uid(&lib_conn, &uid)
			.map_err(|e| e.to_string())?
			.ok_or("Artist not found")?;
		let settings = load_enrich_settings(&settings_conn);
		(artist, settings)
	};

	let client = enrichment::make_client()?;
	let result = enrichment::enrich_artist_async(&client, &artist.name, &settings).await;

	{
		let lib_conn = open_lib_conn(&profile_uid);
		let update = db::ArtistUpdate {
			name: None,
			aka: None,
			about: result.about,
			tags: None,
			genres: result.genres,
			websites: result.websites,
			members: None,
			profile_art_blob: result.profile_art,
			profile_art_path: None,
			banner_art_blob: result.banner_art,
			banner_art_path: None,
		};
		update_artist_by_uid(&lib_conn, &uid, &update).map_err(|e| e.to_string())?;
	}

	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn enrich_all_albums(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
	let profile_uid = state.get_uid();

	let (albums, settings) = {
		let lib_conn = open_lib_conn(&profile_uid);
		let settings_conn = open_settings_conn(&profile_uid);
		let albums = get_all_albums(&lib_conn).map_err(|e| e.to_string())?;
		let settings = load_enrich_settings(&settings_conn);
		(albums, settings)
	};

	let total = albums.len();
	let mut done = 0usize;
	let mut errors = 0usize;

	app.emit("enrich:progress", serde_json::json!({ "done": 0, "total": total, "errors": 0 })).ok();

	let client = enrichment::make_client()?;

	for album in &albums {
		let artist = album
			.album_artist
			.clone()
			.or_else(|| {
				album.artists.as_deref().and_then(|a| {
					serde_json::from_str::<Vec<String>>(a)
						.ok()
						.and_then(|v| v.into_iter().next())
				})
			});

		if let Some(artist) = artist {
			let result = enrichment::enrich_album_async(&client, &album.title, &artist, &settings).await;
			let lib_conn = open_lib_conn(&profile_uid);
			let update = db::AlbumUpdate {
				title: None,
				format: result.format,
				rating: None,
				artists: None,
				album_artist: None,
				release_date: result.release_date,
				tags: None,
				genres: result.genres,
				tracks: None,
				credits: result.description,
				label: result.label,
				artwork_blob: result.artwork,
				artwork_path: None,
			};
			if update_album_by_uid(&lib_conn, &album.uid, &update).is_err() {
				errors += 1;
			}
		} else {
			errors += 1;
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

#[tauri::command]
async fn enrich_all_artists(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
	let profile_uid = state.get_uid();

	let (artists, settings) = {
		let lib_conn = open_lib_conn(&profile_uid);
		let settings_conn = open_settings_conn(&profile_uid);
		let artists = get_all_artists(&lib_conn).map_err(|e| e.to_string())?;
		let settings = load_enrich_settings(&settings_conn);
		(artists, settings)
	};

	let total = artists.len();
	let mut done = 0usize;
	let mut errors = 0usize;

	app.emit("enrich:progress", serde_json::json!({ "done": 0, "total": total, "errors": 0 })).ok();

	let client = enrichment::make_client()?;

	for artist in &artists {
		let result = enrichment::enrich_artist_async(&client, &artist.name, &settings).await;
		let lib_conn = open_lib_conn(&profile_uid);
		let update = db::ArtistUpdate {
			name: None,
			aka: None,
			about: result.about,
			tags: None,
			genres: result.genres,
			websites: result.websites,
			members: None,
			profile_art_blob: result.profile_art,
			profile_art_path: None,
			banner_art_blob: result.banner_art,
			banner_art_path: None,
		};
		if update_artist_by_uid(&lib_conn, &artist.uid, &update).is_err() {
			errors += 1;
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
// ANALYTICS
// ─────────────────────────────────────────────

#[tauri::command]
fn log_scrobble(
	state: State<AppState>,
	track_uid: String,
	artist_uid: String,
) -> Result<String, String> {
	let uid = state.get_uid();
	let conn = open_analytics_conn(&uid).map_err(|e| e.to_string())?;
	let scrobble_uid = db::analytics_manager::new_scrobble_uid();
	let scrobble = db::analytics_manager::Scrobble {
		uid: scrobble_uid.clone(),
		timestamp: std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs() as i64,
		track_uid,
		artist_uid,
		duration_played: 0,
		did_seek: false,
		did_pause: false,
	};
	db::analytics_manager::log_scrobble(&conn, &scrobble).map_err(|e| e.to_string())?;
	Ok(scrobble_uid)
}

#[tauri::command]
fn update_scrobble(
	state: State<AppState>,
	uid: String,
	duration_played: i64,
	did_seek: bool,
	did_pause: bool,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_analytics_conn(&profile_uid).map_err(|e| e.to_string())?;
	let update = db::analytics_manager::ScrobbleUpdate { duration_played, did_seek, did_pause };
	db::analytics_manager::update_scrobble(&conn, &uid, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scrobbles(state: State<AppState>) -> Result<Vec<db::analytics_manager::Scrobble>, String> {
	let uid = state.get_uid();
	let conn = open_analytics_conn(&uid).map_err(|e| e.to_string())?;
	db::analytics_manager::get_all_scrobbles(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scrobbles_for_track(
	state: State<AppState>,
	track_uid: String,
) -> Result<Vec<db::analytics_manager::Scrobble>, String> {
	let uid = state.get_uid();
	let conn = open_analytics_conn(&uid).map_err(|e| e.to_string())?;
	db::analytics_manager::get_scrobbles_for_track(&conn, &track_uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_scrobble(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_analytics_conn(&profile_uid).map_err(|e| e.to_string())?;
	db::analytics_manager::delete_scrobble(&conn, &uid).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// LAST FM
// ─────────────────────────────────────────────

#[tauri::command]
async fn lastfm_get_auth_url(state: State<'_, AppState>) -> Result<String, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	let api_key = db::settings_manager::get_setting(&conn, "api_lastfm_key")
		.map_err(|e| e.to_string())?
		.unwrap_or_default();
	if api_key.is_empty() {
		return Err("Last.fm API key not configured. Add it in Settings → Metadata APIs.".into());
	}
	Ok(lastfm_auth::lastfm_auth_url(&api_key))
}

#[tauri::command]
async fn lastfm_exchange_token_cmd(
	token: String,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let uid = state.get_uid();
	lastfm_auth::lastfm_exchange_token(&uid, &token).await
}

#[tauri::command]
fn lastfm_disconnect_cmd(state: State<'_, AppState>) -> Result<(), String> {
	let uid = state.get_uid();
	lastfm_auth::lastfm_disconnect(&uid)
}

#[tauri::command]
fn lastfm_connection_status(state: State<'_, AppState>) -> Result<bool, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	Ok(lastfm_auth::lastfm_is_connected(&conn))
}

#[tauri::command]
async fn scrobble_track(
	uid: String,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	lastfm_auth::scrobble_track(&profile_uid, &uid).await
}

#[tauri::command]
async fn update_now_playing(
	uid: String,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	lastfm_auth::update_now_playing(&profile_uid, &uid).await
}

// ─────────────────────────────────────────────
// SPOTIFY
// ─────────────────────────────────────────────

#[tauri::command]
fn spotify_get_auth_url(state: State<'_, AppState>) -> Result<(String, String), String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	let client_id = db::settings_manager::get_setting(&conn, "spotify_client_id")
		.map_err(|e| e.to_string())?
		.unwrap_or_default();
	if client_id.is_empty() {
		return Err("Spotify client ID not configured. Add it in Settings → Connected Accounts.".into());
	}
	let (url, verifier): (String, String) = spotify_auth::spotify_auth_url(&client_id);
	state.set_spotify_verifier(verifier.clone());
	Ok((url, verifier))
}

#[tauri::command]
async fn spotify_exchange_code_cmd(
	code: String,
	verifier: String,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let uid = state.get_uid();
	spotify_auth::spotify_exchange_code(&uid, &code, &verifier).await
}

#[tauri::command]
fn spotify_disconnect_cmd(state: State<'_, AppState>) -> Result<(), String> {
	let uid = state.get_uid();
	spotify_auth::spotify_disconnect(&uid)
}

#[tauri::command]
fn spotify_connection_status(state: State<'_, AppState>) -> Result<bool, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	Ok(spotify_auth::spotify_is_connected(&conn))
}

#[tauri::command]
async fn spotify_get_playlists_cmd(
	state: State<'_, AppState>,
) -> Result<Vec<spotify_auth::SpotifyPlaylistSummary>, String> {
	let uid = state.get_uid();
	spotify_auth::spotify_get_playlists(&uid).await
}

#[tauri::command]
async fn spotify_import_playlist_cmd(
	spotify_playlist_id: String,
	playlist_name: String,
	owner: Option<String>,
	state: State<'_, AppState>,
) -> Result<String, String> {
	let uid = state.get_uid();
	spotify_auth::spotify_import_playlist(&uid, &spotify_playlist_id, &playlist_name, owner).await
}

#[tauri::command]
async fn spotify_enrich_track_cmd(
	uid: String,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let lib_conn = open_lib_conn(&profile_uid);
	let track = db::track_manager::get_track_by_uid(&lib_conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or_else(|| format!("Track not found: {}", uid))?;

	let title = track.title.as_deref().unwrap_or("");
	let artist = track
		.artists
		.as_deref()
		.and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
		.and_then(|v| v.into_iter().next())
		.unwrap_or_default();

	let Some(meta) = spotify_auth::enrich_track(&profile_uid, title, &artist).await? else {
		return Ok(());
	};

	let update = db::MetadataUpdate {
		title: meta.title,
		artists: meta.artists.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
		album_artist: None,
		albums: None,
		year: meta.year,
		genres: meta.genres.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
		bpm: None,
		rating: None,
		tags: None,
		key: None,
		credits: None,
		label: None,
		artwork_blob: None,
		artwork_path: None,
		user_options: None,
	};

	db::track_manager::update_track_metadata_by_uid(&lib_conn, &uid, &update)
		.map_err(|e| e.to_string())
}

#[tauri::command]
async fn spotify_enrich_album_cmd(
	uid: String,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let lib_conn = open_lib_conn(&profile_uid);
	let album = db::album_manager::get_album_by_uid(&lib_conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or_else(|| format!("Album not found: {}", uid))?;

	let artist = album
		.album_artist
		.as_deref()
		.or_else(|| album.artists.as_deref())
		.unwrap_or("")
		.to_string();

	let Some(meta) = spotify_auth::enrich_album(&profile_uid, &album.title, &artist).await? else {
		return Ok(());
	};

	let update = db::AlbumUpdate {
		title: None,
		format: None,
		rating: None,
		artists: None,
		album_artist: None,
		release_date: meta.release_date,
		tags: None,
		genres: meta.genres.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
		tracks: None,
		credits: None,
		label: meta.label,
		artwork_blob: None,
		artwork_path: None,
	};

	db::album_manager::update_album_by_uid(&lib_conn, &uid, &update)
		.map_err(|e| e.to_string())
}

#[tauri::command]
async fn spotify_enrich_artist_cmd(
	uid: String,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let lib_conn = open_lib_conn(&profile_uid);
	let artist = db::artist_manager::get_artist_by_uid(&lib_conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or_else(|| format!("Artist not found: {}", uid))?;

	let Some(meta) = spotify_auth::enrich_artist(&profile_uid, &artist.name).await? else {
		return Ok(());
	};

	let update = db::ArtistUpdate {
		name: None,
		aka: None,
		about: None,
		tags: None,
		genres: meta.genres.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
		websites: None,
		members: None,
		profile_art_blob: None,
		profile_art_path: None,
		banner_art_blob: None,
		banner_art_path: None,
	};

	db::artist_manager::update_artist_by_uid(&lib_conn, &uid, &update)
		.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// ENTRY POINT
// ─────────────────────────────────────────────

#[tauri::command]
fn open_in_explorer(path: String) -> Result<(), String> {
	#[cfg(target_os = "windows")]
	std::process::Command::new("explorer")
		.args(["/select,", &path.replace('/', "\\")])
		.spawn()
		.map_err(|e| e.to_string())?;

	#[cfg(target_os = "macos")]
	std::process::Command::new("open")
		.args(["-R", &path])
		.spawn()
		.map_err(|e| e.to_string())?;

	#[cfg(target_os = "linux")]
	std::process::Command::new("xdg-open")
		.arg(std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new("/")))
		.spawn()
		.map_err(|e| e.to_string())?;

	Ok(())
}

pub fn handle_deep_link(app: tauri::AppHandle, url: &str) {
	use tauri::Manager;
	eprintln!("[deep-link] received url: {}", url);

	let state = app.state::<crate::state::AppState>();
	let uid = state.get_uid();

	if let Ok(parsed) = url::Url::parse(url) {
		eprintln!("[deep-link] host: {:?}", parsed.host_str());
		match parsed.host_str() {
			Some("spotify-callback") => {
				eprintln!("[deep-link] matched spotify-callback");
				if let Some(code) = parsed
					.query_pairs()
					.find(|(k, _)| k == "code")
					.map(|(_, v)| v.to_string())
				{
					eprintln!("[deep-link] got code, exchanging...");
					let verifier = state.take_spotify_verifier();
					tauri::async_runtime::spawn(async move {
						let result = spotify_auth::spotify_exchange_code(&uid, &code, &verifier).await;
						eprintln!("[deep-link] exchange result: {:?}", result);
						let _ = app.emit("spotify:connected", ());
					});
				} else {
					eprintln!("[deep-link] no code found in query params");
				}
			}
			_ => {
				eprintln!("[deep-link] unmatched host");
			}
		}
	} else {
		eprintln!("[deep-link] failed to parse url");
	}
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let registry = read_registry();
	let initial_uid = registry.active.clone();

	if !initial_uid.is_empty() {
		let settings_conn = Connection::open(get_settings_db_path(&initial_uid))
			.expect("Failed to open settings db");
		init_settings_db(&settings_conn).expect("Failed to init settings db");

		let lib_conn =
			Connection::open(get_lib_db_path(&initial_uid)).expect("Failed to open lib db");
		init_lib_db(&lib_conn).expect("Failed to init lib db");
	}

	let app_state = AppState::new(initial_uid.clone());

	tauri::Builder::default()
		.plugin(tauri_plugin_clipboard_manager::init())
		.manage(app_state)
		.plugin(tauri_plugin_media::init())
		.plugin(tauri_plugin_log::Builder::new().build())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_deep_link::init())
		.plugin(tauri_plugin_shell::init())
		.setup(|app| {
			let handle = app.handle().clone();
			let uid = app.state::<AppState>().get_uid();

			if !uid.is_empty() {
				let settings_conn = open_settings_conn(&uid);
				let paths = get_library_paths(&settings_conn)
					.unwrap_or_default()
					.into_iter()
					.map(|p| p.path)
					.collect();
				watcher::start_watcher(handle.clone(), uid, paths);
			}

			app.deep_link().on_open_url(move |event| {
				for url in event.urls() {
					handle_deep_link(handle.clone(), url.as_str());
				}
			});

			#[cfg(desktop)]
			app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			needs_profile_setup,
			get_profiles,
			log_scrobble,
			update_scrobble,
			get_scrobbles,
			get_scrobbles_for_track,
			delete_scrobble,
			get_active_profile,
			get_profile_avatar,
			create_profile_cmd,
			update_profile_cmd,
			delete_profile_cmd,
			switch_profile,
			add_path,
			remove_path,
			get_paths,
			rescan,
			add_track,
			get_track,
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
			rename_playlist_folder,
			move_playlist_folder,
			get_settings,
			save_setting,
			enrich_track,
			enrich_all,
			enrich_album,
			enrich_artist,
			enrich_all_albums,
			enrich_all_artists,
			get_track_lyrics,
			fetch_track_lyrics,
			delete_track_lyrics,
			lastfm_get_auth_url,
			lastfm_exchange_token_cmd,
			lastfm_disconnect_cmd,
			lastfm_connection_status,
			scrobble_track,
			update_now_playing,
			spotify_get_auth_url,
			spotify_exchange_code_cmd,
			spotify_disconnect_cmd,
			spotify_connection_status,
			spotify_get_playlists_cmd,
			spotify_import_playlist_cmd,
			spotify_enrich_track_cmd,
			spotify_enrich_album_cmd,
			spotify_enrich_artist_cmd,
			add_uid_remap,
			resolve_uid,
			replace_track_path,
			open_in_explorer
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}