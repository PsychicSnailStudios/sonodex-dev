mod db;
mod enrichment;
mod profiles;
mod scanner;
mod state;
mod watcher;

use db::{
	add_library_path, create_album, create_artist, create_playlist, delete_album_by_uid,
	delete_artist_by_uid, delete_lyrics, delete_playlist_by_uid, get_album_by_uid, get_all_albums,
	get_all_artists, get_all_playlists, get_all_tracks, get_artist_by_uid, get_library_paths,
	get_lyrics, get_playlist_by_uid, init_lib_db, init_settings_db, remove_library_path,
	update_album_by_uid, update_artist_by_uid, update_playlist_by_uid, upsert_track, upsert_lyrics, Album,
	AlbumUpdate, Artist, ArtistUpdate, LibraryPath, Lyrics, Playlist, PlaylistUpdate, Track,
};
use profiles::{
	create_profile as new_profile, get_lib_db_path, get_settings_db_path, read_registry,
	write_registry, Profile,
};
use rusqlite::Connection;
use state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};

fn open_settings_conn(uid: &str) -> Connection {
	Connection::open(get_settings_db_path(uid)).expect("Failed to open settings database")
}

fn open_lib_conn(uid: &str) -> Connection {
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
	registry.profiles.push(profile.clone());
	if is_first {
		registry.active = profile.uid.clone();
	}
	write_registry(&registry);

	if is_first {
		state.set_uid(profile.uid.clone());
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

#[tauri::command]
fn add_path(app: AppHandle, state: State<AppState>, path: String) -> Result<(), String> {
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
		let paths = get_library_paths(&settings_conn)
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
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);
	let lib_conn = open_lib_conn(&uid);
	remove_library_path(&settings_conn, &path).map_err(|e| e.to_string())?;
	lib_conn
		.execute(
			"DELETE FROM tracks WHERE path LIKE ?1",
			rusqlite::params![format!("{}%", path)],
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
	get_lyrics(&conn, track_id).map_err(|e: rusqlite::Error| e.to_string())
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
		.map_err(|e: rusqlite::Error| e.to_string())?;
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
	delete_lyrics(&conn, track_id).map_err(|e: rusqlite::Error| e.to_string())
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

// ─────────────────────────────────────────────
// ENTRY POINT
// ─────────────────────────────────────────────

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
				watcher::start_watcher(handle, uid, paths);
			}

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			needs_profile_setup,
			get_profiles,
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
			get_track_lyrics,
			fetch_track_lyrics,
			delete_track_lyrics,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}