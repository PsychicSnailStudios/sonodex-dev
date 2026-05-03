mod connections;
mod db;
mod enrichment;
mod profiles;
mod scanner;
mod state;
mod watcher;
mod library_manager;
pub mod thumb;

use db::{
    add_library_path, create_album, create_artist, create_playlist, delete_album_by_uid,
    delete_artist_by_uid, delete_lyrics, delete_playlist_by_uid, get_album_by_uid, get_all_albums,
    get_all_artists, get_all_playlists, get_all_tracks, get_artist_by_uid, get_library_paths,
    get_lyrics, get_playlist_by_uid, init_lib_db, init_settings_db, remove_library_path,
    update_album_by_uid, update_artist_by_uid, update_playlist_by_uid, upsert_lyrics, upsert_track,
    Album, AlbumUpdate, Artist, ArtistUpdate, LibraryPath, Lyrics, Playlist, PlaylistUpdate, Track,
};

use crate::connections::lastfm_auth;
use crate::connections::spotify_auth;
use crate::db::MetadataUpdate;

use profiles::{
	create_profile as new_profile, get_lib_db_path, get_library_db_path,
	get_local_library_db_path, get_settings_db_path, read_registry, write_registry, Profile,
};
use argon2::{
	password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
	Argon2,
};
use rusqlite::Connection;
use state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;

use crate::db::tag_manager::{self, Tag, TagGroup, TagKind};

// use tauri::Manager;
// use window_vibrancy::{apply_blur, apply_mica, apply_acrylic, clear_mica, apply_vibrancy, NSVisualEffectMaterial};

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

pub fn open_local_library_conn(uid: &str) -> Connection {
	let path = get_local_library_db_path(uid);
	let conn = Connection::open(&path).expect("Failed to open local library database");
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
	db::init_lib_db(&lib_conn).map_err(|e| e.to_string())?;

	// Bootstrap the new federated library structure
	library_manager::ensure_default_library(&profile.uid)?;

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

#[tauri::command]
fn set_profile_password(uid: String, password: String) -> Result<String, String> {
	let argon2 = Argon2::default();

	let pw_salt = SaltString::generate(&mut OsRng);
	let pw_hash = argon2
		.hash_password(password.as_bytes(), &pw_salt)
		.map_err(|e| e.to_string())?
		.to_string();

	let recovery_key: String = {
		use rand::Rng;
		let mut rng = rand::thread_rng();
		let segments: Vec<String> = (0..6)
			.map(|_| {
				(0..4)
					.map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
					.collect::<String>()
					.to_uppercase()
			})
			.collect();
		segments.join("-")
	};

	let rk_salt = SaltString::generate(&mut OsRng);
	let rk_hash = argon2
		.hash_password(recovery_key.as_bytes(), &rk_salt)
		.map_err(|e| e.to_string())?
		.to_string();

	let mut registry = read_registry();
	if let Some(profile) = registry.profiles.iter_mut().find(|p| p.uid == uid) {
		profile.password_hash = Some(pw_hash);
		profile.recovery_key_hash = Some(rk_hash);
	} else {
		return Err("Profile not found".to_string());
	}
	write_registry(&registry);

	Ok(recovery_key)
}

#[tauri::command]
fn remove_profile_password(uid: String) -> Result<(), String> {
	let mut registry = read_registry();
	if let Some(profile) = registry.profiles.iter_mut().find(|p| p.uid == uid) {
		profile.password_hash = None;
		profile.recovery_key_hash = None;
	} else {
		return Err("Profile not found".to_string());
	}
	write_registry(&registry);
	Ok(())
}

#[tauri::command]
fn verify_profile_password(uid: String, password: String) -> Result<bool, String> {
	let registry = read_registry();
	let profile = registry
		.profiles
		.iter()
		.find(|p| p.uid == uid)
		.ok_or("Profile not found")?;

	let hash_str = match &profile.password_hash {
		Some(h) => h,
		None => return Ok(true),
	};

	let parsed = PasswordHash::new(hash_str).map_err(|e| e.to_string())?;
	Ok(Argon2::default()
		.verify_password(password.as_bytes(), &parsed)
		.is_ok())
}

#[tauri::command]
fn verify_recovery_key(uid: String, key: String) -> Result<bool, String> {
	let registry = read_registry();
	let profile = registry
		.profiles
		.iter()
		.find(|p| p.uid == uid)
		.ok_or("Profile not found")?;

	let hash_str = match &profile.recovery_key_hash {
		Some(h) => h,
		None => return Ok(false),
	};

	let parsed = PasswordHash::new(hash_str).map_err(|e| e.to_string())?;
	Ok(Argon2::default()
		.verify_password(key.as_bytes(), &parsed)
		.is_ok())
}

#[tauri::command]
fn profile_has_password(uid: String) -> Result<bool, String> {
	let registry = read_registry();
	let profile = registry
		.profiles
		.iter()
		.find(|p| p.uid == uid)
		.ok_or("Profile not found")?;
	Ok(profile.password_hash.is_some())
}

// ─────────────────────────────────────────────
// LIBRARY PATHS
// ─────────────────────────────────────────────

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[tauri::command]
fn add_path(
	app: AppHandle,
	state: State<AppState>,
	path: String,
	lib_uid: Option<String>,
) -> Result<(), String> {
	let path = normalize_path(&path);
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);

	// Resolve which library to scan into — default to the local default library
	let target_lib_uid = match lib_uid {
		Some(ref l) => l.clone(),
		None => {
			library_manager::ensure_default_library(&uid)?;
			db::library_registry::get_default_library(&settings_conn)
				.map_err(|e| e.to_string())?
				.ok_or("No default library found")?
				.uid
		}
	};

	add_library_path(&settings_conn, &path).map_err(|e| e.to_string())?;

	// Also store lib_uid alongside the path so we know which library owns it
	settings_conn.execute(
		"UPDATE library_paths SET lib_uid = ?1 WHERE path = ?2",
		rusqlite::params![target_lib_uid, path],
	).ok();

	let app_clone = app.clone();
	let path_clone = path.clone();
	let uid_clone = uid.clone();
	let lib_uid_clone = target_lib_uid.clone();

	std::thread::spawn(move || {
		let lib_path = get_library_db_path(&uid_clone, &lib_uid_clone);
		let lib_conn = Connection::open(&lib_path).expect("Failed to open library db");
		let settings_path = get_settings_db_path(&uid_clone);
		lib_conn.execute_batch(&format!(
			"ATTACH DATABASE '{}' AS settings;",
			settings_path.to_string_lossy().replace('\'', "''")
		)).ok();
		scanner::scan_directory_with_progress(&lib_conn, &path_clone, &app_clone);

		// Incremental merge after scan
		let _ = library_manager::incremental_update(&uid_clone, &[lib_uid_clone.clone()]);

		let settings_conn2 = open_settings_conn(&uid_clone);
		let auto_tracks = db::get_setting(&settings_conn2, "auto_enrich_tracks")
			.ok().flatten().map(|v| v == "true").unwrap_or(false);
		let auto_albums = db::get_setting(&settings_conn2, "auto_enrich_albums")
			.ok().flatten().map(|v| v == "true").unwrap_or(false);
		let auto_artists = db::get_setting(&settings_conn2, "auto_enrich_artists")
			.ok().flatten().map(|v| v == "true").unwrap_or(false);

		if auto_tracks || auto_albums || auto_artists {
			let app_enrich = app_clone.clone();
			tauri::async_runtime::spawn(async move {
				let state = app_enrich.state::<AppState>();
				if auto_tracks { let _ = enrich_all(app_enrich.clone(), state.clone()).await; }
				if auto_albums { let _ = enrich_all_albums(app_enrich.clone(), state.clone()).await; }
				if auto_artists { let _ = enrich_all_artists(app_enrich.clone(), state.clone()).await; }
			});
		}

		let settings_conn3 = open_settings_conn(&uid_clone);
		let paths = get_library_paths(&settings_conn3)
			.unwrap_or_default()
			.into_iter()
			.map(|p| p.path)
			.collect();
		watcher::start_watcher(app_clone, uid_clone, lib_uid_clone, paths);
	});

	Ok(())
}

#[tauri::command]
fn remove_path(state: State<AppState>, path: String) -> Result<(), String> {
	let path = normalize_path(&path);
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);

	// Find which library owns this path before removing it
	let lib_uid: Option<String> = settings_conn.query_row(
		"SELECT lib_uid FROM library_paths WHERE path = ?1",
		rusqlite::params![path],
		|row| row.get(0),
	).ok();

	remove_library_path(&settings_conn, &path).map_err(|e| e.to_string())?;

	// Remove tracks from the correct library db
	if let Some(ref luid) = lib_uid {
		let lib_path = get_library_db_path(&uid, luid);
		if let Ok(lib_conn) = Connection::open(&lib_path) {
			lib_conn.execute(
				"DELETE FROM tracks WHERE path LIKE ?1",
				rusqlite::params![format!("{}%", path)],
			).ok();
			lib_conn.execute(
				"DELETE FROM albums WHERE uid NOT IN (
					SELECT DISTINCT json_extract(json_each.value, '$.uid')
					FROM tracks, json_each(tracks.albums)
					WHERE json_extract(json_each.value, '$.uid') IS NOT NULL
					AND json_extract(json_each.value, '$.uid') != ''
				)", [],
			).ok();
			lib_conn.execute(
				"DELETE FROM artists WHERE name NOT IN (
					SELECT DISTINCT json_each.value
					FROM tracks, json_each(tracks.artists)
					WHERE tracks.artists IS NOT NULL AND tracks.artists != '[]'
				) AND name NOT IN (
					SELECT DISTINCT album_artist FROM tracks WHERE album_artist IS NOT NULL
				)", [],
			).ok();

			// Update merged cache
			let _ = library_manager::incremental_update(&uid, &[luid.clone()]);
		}
	} else {
		// Fallback: try the legacy lib.db
		if let Ok(lib_conn) = Connection::open(get_lib_db_path(&uid)) {
			lib_conn.execute(
				"DELETE FROM tracks WHERE path LIKE ?1",
				rusqlite::params![format!("{}%", path)],
			).ok();
		}
	}

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

	let default_lib_uid = db::library_registry::get_default_library(&settings_conn)
		.map_err(|e| e.to_string())?
		.map(|l| l.uid)
		.unwrap_or_default();

	let app_clone = app.clone();
	let uid_clone = uid.clone();
	let lib_uid_clone = default_lib_uid.clone();
	let path_strings: Vec<String> = paths.into_iter().map(|p| p.path).collect();

	std::thread::spawn(move || {
		let lib_path = get_library_db_path(&uid_clone, &lib_uid_clone);
		let lib_conn = Connection::open(&lib_path).expect("Failed to open library db");
		let settings_path = get_settings_db_path(&uid_clone);
		lib_conn.execute_batch(&format!(
			"ATTACH DATABASE '{}' AS settings;",
			settings_path.to_string_lossy().replace('\'', "''")
		)).ok();

		for p in &path_strings {
			scanner::scan_directory_with_progress(&lib_conn, p, &app_clone);
		}

		let _ = library_manager::incremental_update(&uid_clone, &[lib_uid_clone.clone()]);
		watcher::start_watcher(app_clone, uid_clone, lib_uid_clone, path_strings);
	});

	Ok(())
}

// ─────────────────────────────────────────────
// FEDERATED LIBRARY MANAGEMENT
// ─────────────────────────────────────────────

#[tauri::command]
fn get_libraries(state: State<AppState>) -> Result<Vec<db::library_registry::Library>, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::library_registry::get_all_libraries(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_library_cmd(state: State<AppState>, name: String) -> Result<db::library_registry::Library, String> {
	let uid = state.get_uid();
	library_manager::create_local_library(&uid, &name)
}

#[tauri::command]
async fn import_library_cmd(
	state: State<'_, AppState>,
	name: String,
	sync_url: String,
	sync_meta_url: String,
	write_token: Option<String>,
) -> Result<db::library_registry::Library, String> {
	let uid = state.get_uid();
	library_manager::import_library(&uid, &name, &sync_url, &sync_meta_url, write_token.as_deref()).await
}

#[tauri::command]
fn export_library_cmd(state: State<AppState>, lib_uid: String, dest_path: String) -> Result<(), String> {
	let uid = state.get_uid();
	library_manager::export_library(&uid, &lib_uid, &dest_path)
}

#[tauri::command]
fn update_library_cmd(
	state: State<AppState>,
	lib_uid: String,
	update: db::library_registry::LibraryUpdate,
) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::library_registry::update_library(&conn, &lib_uid, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_library_cmd(
	state: State<AppState>,
	lib_uid: String,
	delete_file: bool,
) -> Result<(), String> {
	let uid = state.get_uid();
	library_manager::delete_local_library(&uid, &lib_uid, delete_file)
}

#[tauri::command]
async fn sync_library_cmd(
	app: AppHandle,
	state: State<'_, AppState>,
	lib_uid: String,
) -> Result<bool, String> {
	let uid = state.get_uid();
	let pulled = library_manager::try_pull_library(&uid, &lib_uid).await?;
	if pulled {
		library_manager::full_rebuild(&uid)?;
		app.emit("library:updated", ()).ok();
	}
	Ok(pulled)
}

#[tauri::command]
async fn push_library_cmd(
	state: State<'_, AppState>,
	lib_uid: String,
) -> Result<bool, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	let lib = db::library_registry::get_library_by_uid(&conn, &lib_uid)
		.map_err(|e| e.to_string())?
		.ok_or("Library not found")?;
	Ok(library_manager::try_push_library(&lib).await)
}

#[tauri::command]
async fn check_write_permission_cmd(
	state: State<'_, AppState>,
	lib_uid: String,
) -> Result<bool, String> {
	let uid = state.get_uid();
	library_manager::check_write_permission(&uid, &lib_uid).await
}

#[tauri::command]
fn rebuild_merged_cmd(state: State<AppState>) -> Result<library_manager::MergeResult, String> {
	let uid = state.get_uid();
	library_manager::full_rebuild(&uid)
}

#[tauri::command]
fn get_blocklist(state: State<AppState>) -> Result<Vec<db::blocklist_manager::BlocklistEntry>, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::blocklist_manager::get_all_blocklist_entries(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_to_blocklist_cmd(
	state: State<AppState>,
	uid: String,
	entity_type: String,
	cascade: bool,
	source_lib_uid: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_settings_conn(&profile_uid);
	db::blocklist_manager::add_to_blocklist(
		&conn, &uid, &entity_type, Some("hidden by user"), cascade, &source_lib_uid,
	).map_err(|e| e.to_string())?;

	// Remove from merged immediately
	let merged_path = crate::profiles::get_merged_db_path(&profile_uid);
	if merged_path.exists() {
		if let Ok(merged_conn) = Connection::open(&merged_path) {
			merged_conn.execute(
				&format!("DELETE FROM {} WHERE uid = ?1", entity_type),
				rusqlite::params![uid],
			).ok();
		}
	}
	Ok(())
}

#[tauri::command]
fn remove_from_blocklist_cmd(state: State<AppState>, uid: String) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_settings_conn(&profile_uid);
	db::blocklist_manager::remove_from_blocklist(&conn, &uid).map_err(|e| e.to_string())?;
	// Trigger a full rebuild so the unblocked record reappears
	library_manager::full_rebuild(&profile_uid)?;
	Ok(())
}

#[tauri::command]
fn get_delete_preference_cmd(
	state: State<AppState>,
	lib_uid: String,
) -> Result<Option<db::blocklist_manager::LibraryDeletePreference>, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::blocklist_manager::get_delete_preference(&conn, &lib_uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_delete_preference_cmd(
	state: State<AppState>,
	lib_uid: String,
	cascade_delete: i64,
) -> Result<(), String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	db::blocklist_manager::set_delete_preference(&conn, &lib_uid, cascade_delete)
		.map_err(|e| e.to_string())
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
fn merge_remote_local_tracks(
	app: AppHandle,
	state: State<AppState>,
	keep_uid: String,
	drop_uid: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_lib_conn(&profile_uid);

	let drop_track = db::get_track_by_uid(&conn, &drop_uid)
		.map_err(|e| e.to_string())?
		.ok_or("Drop track not found")?;

	scanner::merge_paths_into_existing(&conn, &keep_uid, &drop_track);
	db::delete_track_by_uid(&conn, &drop_uid).map_err(|e| e.to_string())?;

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
    db::update_track_metadata_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())?;

    if let Some(ref tags_json) = update.tags {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(tags_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Tag,
                );
            }
        }
    }
    if let Some(ref genres_json) = update.genres {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Genre,
                );
            }
        }
    }

    Ok(())
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

    if let Some(ref tags_json) = update.tags {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(tags_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Tag,
                );
            }
        }
    }
    if let Some(ref genres_json) = update.genres {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Genre,
                );
            }
        }
    }

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
        .execute(
            "UPDATE tracks SET path = ?1 WHERE uid = ?2",
            [&new_path, &uid],
        )
        .map_err(|e| e.to_string())?;

    let get = |key: &str, default: &str| -> String {
        db::settings_manager::get_setting(&settings_conn, key)
            .ok()
            .flatten()
            .unwrap_or_else(|| default.to_string())
    };

    let priority_title = get("filename_priority_title", "tag");
    let priority_artist = get("filename_priority_artist", "tag");
    let priority_album = get("filename_priority_album", "tag");
    let priority_year = get("filename_priority_year", "tag");
    let custom_pattern = get("filename_custom_pattern", "");

    let tag_delim_raw = get("artist_tag_delimiters", " / |; ");
    let filename_delim_raw = get(
        "artist_filename_delimiters",
        " / |; | feat. | ft. | featuring ",
    );
    let genre_delim_raw = get("genre_delimiters", " / |; |, ");

    let tag_delims: Vec<&str> = tag_delim_raw.split('|').collect();
    let filename_delims: Vec<&str> = filename_delim_raw.split('|').collect();
    let genre_delims: Vec<&str> = genre_delim_raw.split('|').collect();

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
            title: fresh_track.title,
            artists: fresh_track.artists,
            album_artist: fresh_track.album_artist,
            albums: fresh_track.albums,
            year: fresh_track.year,
            genres: fresh_track.genres,
            bpm: fresh_track.bpm,
            rating: fresh_track.rating,
            key: fresh_track.key,
            credits: fresh_track.credits,
            label: fresh_track.label,
            artwork_blob: fresh_track.artwork_blob,
            format: fresh_track.format,
            bitrate: fresh_track.bitrate,
            ..Default::default()
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
    update_album_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())?;

    if let Some(ref tags_json) = update.tags {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(tags_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Tag,
                );
            }
        }
    }
    if let Some(ref genres_json) = update.genres {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Genre,
                );
            }
        }
    }

    Ok(())
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
    update_artist_by_uid(&conn, &uid, &update).map_err(|e| e.to_string())?;

    if let Some(ref tags_json) = update.tags {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(tags_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Tag,
                );
            }
        }
    }
    if let Some(ref genres_json) = update.genres {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
            for name in names {
                crate::db::tag_manager::ensure_tag(
                    &conn,
                    &name,
                    crate::db::tag_manager::TagKind::Genre,
                );
            }
        }
    }

    Ok(())
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
    let result = enrichment::enrich_track_async(&client, &track_input, &settings, Some(&profile_uid)).await?;

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
            key: result.key,
            artwork_blob: result.artwork,
            ..Default::default()
        };

        if let Some(ref genres_json) = update.genres {
            if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
                for name in names {
                    crate::db::tag_manager::ensure_tag(
                        &conn,
                        &name,
                        crate::db::tag_manager::TagKind::Genre,
                    );
                }
            }
        }
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
        match enrichment::enrich_track_async(&client, track_input, &settings, Some(&profile_uid)).await {
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
                    key: result.key,
                    artwork_blob: result.artwork,
                    ..Default::default()
                };
                if let Some(ref genres_json) = update.genres {
                    if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
                        for name in names {
                            crate::db::tag_manager::ensure_tag(
                                &conn,
                                &name,
                                crate::db::tag_manager::TagKind::Genre,
                            );
                        }
                    }
                }
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

	let track_uids: Vec<String> = album
		.tracks
		.as_deref()
		.and_then(|t| serde_json::from_str::<Vec<serde_json::Value>>(t).ok())
		.unwrap_or_default()
		.into_iter()
		.filter_map(|e| e["uid"].as_str().map(|s| s.to_string()))
		.filter(|s| !s.is_empty())
		.collect();

	let client = enrichment::make_client()?;
	let result = enrichment::enrich_album_async(&client, &album.title, &artist, &settings, Some(&profile_uid)).await;

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
            emulate_type: None,
		};
		if let Some(ref genres_json) = update.genres {
			if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
				for name in names {
					crate::db::tag_manager::ensure_tag(
						&lib_conn,
						&name,
						crate::db::tag_manager::TagKind::Genre,
					);
				}
			}
		}
		update_album_by_uid(&lib_conn, &uid, &update).map_err(|e| e.to_string())?;

		if let Some(ref art) = update.artwork_blob {
			for track_uid in &track_uids {
				let lib_conn = open_lib_conn(&profile_uid);
				let has_art = db::get_track_artwork_by_uid(&lib_conn, track_uid)
					.ok()
					.flatten()
					.map(|b| !b.is_empty())
					.unwrap_or(false);
				if !has_art {
					let art_update = db::MetadataUpdate {
						artwork_blob: Some(art.clone()),
						..Default::default()
					};
					db::update_track_metadata_by_uid(&lib_conn, track_uid, &art_update).ok();
				}
			}
		}
	}

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

		if let Ok(track_result) = enrichment::enrich_track_async(&client, &track_input, &settings, Some(&profile_uid)).await {
			let lib_conn = open_lib_conn(&profile_uid);
			let update = db::MetadataUpdate {
				title: track_result.title,
				artists: track_result.artists,
				album_artist: track_result.album_artist,
				albums: track_result.albums,
				year: track_result.year,
				genres: track_result.genres,
				bpm: track_result.bpm,
				key: track_result.key,
				artwork_blob: track_result.artwork,
                ..Default::default()
			};
			if let Some(ref genres_json) = update.genres {
				if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
					for name in names {
						crate::db::tag_manager::ensure_tag(
							&lib_conn,
							&name,
							crate::db::tag_manager::TagKind::Genre,
						);
					}
				}
			}
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
    let result = enrichment::enrich_artist_async(&client, &artist.name, &settings, Some(&profile_uid)).await;

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
		if let Some(ref genres_json) = update.genres {
			if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
				for name in names {
					crate::db::tag_manager::ensure_tag(
						&lib_conn,
						&name,
						crate::db::tag_manager::TagKind::Genre,
					);
				}
			}
		}
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

    app.emit(
        "enrich:progress",
        serde_json::json!({ "done": 0, "total": total, "errors": 0 }),
    )
    .ok();

    let client = enrichment::make_client()?;

    for album in &albums {
        let artist = album.album_artist.clone().or_else(|| {
            album.artists.as_deref().and_then(|a| {
                serde_json::from_str::<Vec<String>>(a)
                    .ok()
                    .and_then(|v| v.into_iter().next())
            })
        });

        if let Some(artist) = artist {
            let result = enrichment::enrich_album_async(&client, &album.title, &artist, &settings, Some(&profile_uid)).await;
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
                emulate_type: None,
            };
            if let Some(ref genres_json) = update.genres {
                if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
                    for name in names {
                        crate::db::tag_manager::ensure_tag(
                            &lib_conn,
                            &name,
                            crate::db::tag_manager::TagKind::Genre,
                        );
                    }
                }
            }
            if update_album_by_uid(&lib_conn, &album.uid, &update).is_err() {
                errors += 1;
            }
        } else {
            errors += 1;
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

    app.emit(
        "enrich:progress",
        serde_json::json!({ "done": 0, "total": total, "errors": 0 }),
    )
    .ok();

    let client = enrichment::make_client()?;

    for artist in &artists {
        let result = enrichment::enrich_artist_async(&client, &artist.name, &settings, Some(&profile_uid)).await;
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
        if let Some(ref genres_json) = update.genres {
			if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
				for name in names {
					crate::db::tag_manager::ensure_tag(
						&lib_conn,
						&name,
						crate::db::tag_manager::TagKind::Genre,
					);
				}
			}
		}
		if update_artist_by_uid(&lib_conn, &artist.uid, &update).is_err() {
			errors += 1;
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
// ANALYTICS
// ─────────────────────────────────────────────

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

#[tauri::command]
fn log_scrobble(
	state: State<AppState>,
	track_uid: String,
	artist_uid: String,
	reason_start: Option<String>,
	shuffle: Option<bool>,
	offline: Option<bool>,
	playing_local: Option<bool>,
	track_name: Option<String>,
	track_artist: Option<String>,
	track_album: Option<String>,
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
		reason_start,
		reason_end: None,
		shuffle,
		skipped: None,
		offline,
		playing_local,
		track_name,
		track_artist,
		track_album,
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
	reason_end: Option<String>,
	skipped: Option<bool>,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let conn = open_analytics_conn(&profile_uid).map_err(|e| e.to_string())?;
	let update = db::analytics_manager::ScrobbleUpdate {
		duration_played,
		did_seek,
		did_pause,
		reason_end,
		skipped,
	};
	db::analytics_manager::update_scrobble(&conn, &uid, &update).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// LISTENBRAINZ
// ─────────────────────────────────────────────

#[tauri::command]
fn listenbrainz_connection_status(state: State<AppState>) -> Result<bool, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	Ok(crate::connections::listenbrainz::listenbrainz_is_connected(&conn))
}

#[tauri::command]
async fn listenbrainz_validate_token_cmd(token: String) -> Result<bool, String> {
	crate::connections::listenbrainz::listenbrainz_validate_token(&token).await
}

#[tauri::command]
fn listenbrainz_connect_cmd(state: State<AppState>, token: String) -> Result<(), String> {
	let uid = state.get_uid();
	crate::connections::listenbrainz::listenbrainz_connect(&uid, token)
}

#[tauri::command]
fn listenbrainz_disconnect_cmd(state: State<AppState>) -> Result<(), String> {
	let uid = state.get_uid();
	crate::connections::listenbrainz::listenbrainz_disconnect(&uid)
}

// ─────────────────────────────────────────────
// SPOTIFY HISTORY IMPORT
// ─────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct SpotifyStreamEntry {
	ts: Option<String>,
	ms_played: Option<i64>,
	master_metadata_track_name: Option<String>,
	master_metadata_album_artist_name: Option<String>,
	master_metadata_album_album_name: Option<String>,
	reason_start: Option<String>,
	reason_end: Option<String>,
	shuffle: Option<bool>,
	skipped: Option<bool>,
	offline: Option<bool>,
}

#[derive(serde::Serialize)]
struct ImportResult {
	imported: usize,
	skipped: usize,
}

#[tauri::command]
fn import_spotify_history_cmd(state: State<AppState>, zip_path: String) -> Result<ImportResult, String> {
	use std::io::Read;

	let uid = state.get_uid();
	let file = std::fs::File::open(&zip_path).map_err(|e| format!("open zip: {}", e))?;
	let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("read zip: {}", e))?;

	let lib_conn = open_lib_conn(&uid);
	let analytics_conn = open_analytics_conn(&uid).map_err(|e| e.to_string())?;

	let all_tracks = db::get_all_tracks(&lib_conn).unwrap_or_default();
	let all_artists = db::get_all_artists(&lib_conn).unwrap_or_default();

	let mut imported = 0usize;
	let mut skipped = 0usize;

	for i in 0..archive.len() {
		let mut entry = archive.by_index(i).map_err(|e| format!("zip entry: {}", e))?;

		if entry.is_dir() {
			continue;
		}

		let name = entry.name().to_string();
		if !name.contains("Streaming_History_Audio") || !name.ends_with(".json") {
			continue;
		}

		let mut content = String::new();
		if entry.read_to_string(&mut content).is_err() {
			continue;
		}

		let entries: Vec<SpotifyStreamEntry> = match serde_json::from_str(&content) {
			Ok(v) => v,
			Err(_) => continue,
		};

		for item in entries {
			let ms_played = item.ms_played.unwrap_or(0);
			if ms_played < 5000 {
				skipped += 1;
				continue;
			}

			let timestamp = item.ts.as_deref().and_then(|ts| {
				chrono::DateTime::parse_from_rfc3339(ts)
					.ok()
					.map(|dt| dt.timestamp())
			}).unwrap_or(0);

			let track_name = item.master_metadata_track_name.clone();
			let artist_name = item.master_metadata_album_artist_name.clone();
			let album_name = item.master_metadata_album_album_name.clone();

			let track_uid = track_name.as_deref().and_then(|tn| {
				let tn_lower = tn.to_lowercase();
				all_tracks.iter().find(|t| {
					t.title.as_deref().map(|s| s.to_lowercase()) == Some(tn_lower.clone())
					&& artist_name.as_deref().map_or(true, |an| {
						let an_lower = an.to_lowercase();
						t.album_artist.as_deref().map(|s| s.to_lowercase()) == Some(an_lower.clone())
						|| t.artists.as_deref()
							.and_then(|a| serde_json::from_str::<Vec<String>>(a).ok())
							.map_or(false, |v| v.iter().any(|s| s.to_lowercase() == an_lower))
					})
				}).map(|t| t.uid.clone())
			}).unwrap_or_default();

			let artist_uid = artist_name.as_deref().and_then(|an| {
				let an_lower = an.to_lowercase();
				all_artists.iter().find(|a| a.name.to_lowercase() == an_lower)
					.map(|a| a.uid.clone())
			}).unwrap_or_default();

			let scrobble_uid = db::analytics_manager::new_scrobble_uid();
			let scrobble = db::analytics_manager::Scrobble {
				uid: scrobble_uid,
				timestamp,
				track_uid,
				artist_uid,
				duration_played: ms_played,
				did_seek: false,
				did_pause: false,
				reason_start: item.reason_start,
				reason_end: item.reason_end,
				shuffle: item.shuffle,
				skipped: item.skipped,
				offline: item.offline,
				playing_local: Some(false),
				track_name,
				track_artist: artist_name,
				track_album: album_name,
			};

			match db::analytics_manager::log_scrobble(&analytics_conn, &scrobble) {
				Ok(_) => imported += 1,
				Err(_) => skipped += 1,
			}
		}
	}

	Ok(ImportResult { imported, skipped })
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
async fn scrobble_track(uid: String, state: State<'_, AppState>) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let timestamp = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs() as i64;
 
	let _ = lastfm_auth::scrobble_track(&profile_uid, &uid).await;
	let _ = crate::connections::listenbrainz::submit_listen(&profile_uid, &uid, timestamp).await;
 
	Ok(())
}

#[tauri::command]
async fn update_now_playing(uid: String, state: State<'_, AppState>) -> Result<(), String> {
	let profile_uid = state.get_uid();
 
	let _ = lastfm_auth::update_now_playing(&profile_uid, &uid).await;
	let _ = crate::connections::listenbrainz::update_now_playing(&profile_uid, &uid).await;
 
	Ok(())
}

// ─────────────────────────────────────────────
// DOWNLOAD
// ─────────────────────────────────────────────

#[tauri::command]
async fn download_track_cmd(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
) -> Result<String, String> {
	use std::path::PathBuf;
	use tauri_plugin_shell::ShellExt;

	let profile_uid = state.get_uid();
	let lib_conn = open_lib_conn(&profile_uid);
	let settings_conn = open_settings_conn(&profile_uid);

	let track = db::get_track_by_uid(&lib_conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or("Track not found")?;

	let remote_path = track.remote_path.clone().ok_or("Track has no remote path")?;

	let download_base = db::settings_manager::get_setting(&settings_conn, "download_path")
		.ok()
		.flatten()
		.unwrap_or_default();
	if download_base.is_empty() {
		return Err("Download path not configured".to_string());
	}

	let path_style = db::settings_manager::get_setting(&settings_conn, "download_path_style")
		.ok()
		.flatten()
		.unwrap_or_else(|| "{artist}/{album}".to_string());

	let filename_style = db::settings_manager::get_setting(&settings_conn, "download_filename_style")
		.ok()
		.flatten()
		.unwrap_or_else(|| "{track_number} - {title}".to_string());

	let convert_mp3 = db::settings_manager::get_setting(&settings_conn, "download_convert_mp3")
		.ok()
		.flatten()
		.map(|v| v == "true")
		.unwrap_or(false);

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
		.unwrap_or_else(|| "Unknown Artist".to_string());

	let album = track
		.albums
		.as_deref()
		.and_then(|a| serde_json::from_str::<Vec<serde_json::Value>>(a).ok())
		.and_then(|v| v.into_iter().next())
		.and_then(|e| e["name"].as_str().map(|s| s.to_string()))
		.unwrap_or_else(|| "Unknown Album".to_string());

	let track_number = track
		.albums
		.as_deref()
		.and_then(|a| serde_json::from_str::<Vec<serde_json::Value>>(a).ok())
		.and_then(|v| v.into_iter().next())
		.and_then(|e| e["track_number"].as_u64())
		.map(|n| format!("{:02}", n))
		.unwrap_or_else(|| "00".to_string());

	let year = track.year.clone().unwrap_or_else(|| "Unknown Year".to_string());
	let title = track.title.clone().unwrap_or_else(|| "Unknown Title".to_string());

	let sanitize = |s: &str| -> String {
		s.chars()
			.map(|c| match c {
				'/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
				c => c,
			})
			.collect()
	};

	let folder = path_style
		.replace("{artist}", &sanitize(&artist))
		.replace("{album}", &sanitize(&album))
		.replace("{year}", &sanitize(&year));

	let ext = if convert_mp3 {
		"mp3".to_string()
	} else {
		remote_path
			.rsplit('.')
			.next()
			.unwrap_or("mp3")
			.to_string()
			.to_lowercase()
	};

	let filename_base = filename_style
		.replace("{track_number}", &track_number)
		.replace("{title}", &sanitize(&title))
		.replace("{artist}", &sanitize(&artist))
		.replace("{album}", &sanitize(&album))
		.replace("{year}", &sanitize(&year));

	let dest_dir = PathBuf::from(&download_base).join(&folder);
	std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
	let dest_path = dest_dir.join(format!("{}.{}", filename_base, ext));
	let dest_str = dest_path.to_string_lossy().to_string();

	if convert_mp3 {
		let ffmpeg_output = app
			.shell()
			.sidecar("ffmpeg")
			.map_err(|e| e.to_string())?
			.args(["-y", "-i", &remote_path, "-b:a", "320k", &dest_str])
			.output()
			.await
			.map_err(|e| e.to_string())?;

		if !ffmpeg_output.status.success() {
			let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr).to_string();
			return Err(format!("FFmpeg failed: {}", stderr));
		}
	} else {
		if remote_path.starts_with("http://") || remote_path.starts_with("https://") {
			let client = enrichment::make_client()?;
			let bytes = client
				.get(&remote_path)
				.send()
				.await
				.map_err(|e| e.to_string())?
				.bytes()
				.await
				.map_err(|e| e.to_string())?;
			std::fs::write(&dest_path, &bytes).map_err(|e| e.to_string())?;
		} else {
			std::fs::copy(&remote_path, &dest_path).map_err(|e| e.to_string())?;
		}
	}

	let format = ext.to_uppercase();

	let bitrate: Option<i64> = if let Ok(metadata) = std::fs::metadata(&dest_path) {
		let duration_secs = track.duration_ms.unwrap_or(0) as f64 / 1000.0;
		if duration_secs > 0.0 {
			Some(((metadata.len() as f64 * 8.0) / duration_secs / 1000.0) as i64)
		} else {
			None
		}
	} else {
		None
	};

	let track_data = serde_json::json!({
		"bitrate": bitrate,
		"format": format,
		"is_ghost": false
	})
	.to_string();

	db::update_track_metadata_by_uid(
		&lib_conn,
		&uid,
		&db::MetadataUpdate {
			format: Some(format),
			bitrate,
			track_data: Some(track_data),
			..Default::default()
		},
	)
	.map_err(|e| e.to_string())?;

	lib_conn
		.execute(
			"UPDATE tracks SET path = ?1 WHERE uid = ?2",
			rusqlite::params![dest_str, uid],
		)
		.map_err(|e| e.to_string())?;

	app.emit("library:updated", ()).ok();

	Ok(dest_str)
}

#[tauri::command]
async fn download_album_tracks_cmd(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
	subscribe: bool,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let lib_conn = open_lib_conn(&profile_uid);

	let album = db::get_album_by_uid(&lib_conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or("Album not found")?;

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
		let track = match db::get_track_by_uid(&lib_conn, &track_uid) {
			Ok(Some(t)) => t,
			_ => continue,
		};
		if track.remote_path.is_none() {
			continue;
		}
		let local_path_ok = !track.path.is_empty()
			&& !track.path.starts_with("t-")
			&& std::path::Path::new(&track.path).exists();
		if local_path_ok {
			continue;
		}
		let _ = download_track_cmd(app.clone(), state.clone(), track_uid).await;
	}

	if subscribe {
		let settings_conn = open_settings_conn(&profile_uid);
		let mut subs: Vec<String> = db::settings_manager::get_setting(&settings_conn, "offline_subscriptions")
			.ok()
			.flatten()
			.and_then(|v| serde_json::from_str(&v).ok())
			.unwrap_or_default();
		if !subs.contains(&uid) {
			subs.push(uid);
			db::settings_manager::set_setting(
				&settings_conn,
				"offline_subscriptions",
				&serde_json::to_string(&subs).unwrap_or_else(|_| "[]".to_string()),
			)
			.ok();
		}
	}

	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn download_playlist_tracks_cmd(
	app: AppHandle,
	state: State<'_, AppState>,
	uid: String,
	subscribe: bool,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let lib_conn = open_lib_conn(&profile_uid);

	let playlist = db::get_playlist_by_uid(&lib_conn, &uid)
		.map_err(|e| e.to_string())?
		.ok_or("Playlist not found")?;

	let track_uids: Vec<String> = playlist
		.tracks
		.as_deref()
		.and_then(|t| serde_json::from_str::<Vec<serde_json::Value>>(t).ok())
		.unwrap_or_default()
		.into_iter()
		.filter_map(|e| e["uid"].as_str().map(|s| s.to_string()))
		.filter(|s| !s.is_empty())
		.collect();

	for track_uid in track_uids {
		let track = match db::get_track_by_uid(&lib_conn, &track_uid) {
			Ok(Some(t)) => t,
			_ => continue,
		};
		if track.remote_path.is_none() {
			continue;
		}
		let local_path_ok = !track.path.is_empty()
			&& !track.path.starts_with("t-")
			&& std::path::Path::new(&track.path).exists();
		if local_path_ok {
			continue;
		}
		let _ = download_track_cmd(app.clone(), state.clone(), track_uid).await;
	}

	if subscribe {
		let settings_conn = open_settings_conn(&profile_uid);
		let mut subs: Vec<String> = db::settings_manager::get_setting(&settings_conn, "offline_subscriptions")
			.ok()
			.flatten()
			.and_then(|v| serde_json::from_str(&v).ok())
			.unwrap_or_default();
		if !subs.contains(&uid) {
			subs.push(uid);
			db::settings_manager::set_setting(
				&settings_conn,
				"offline_subscriptions",
				&serde_json::to_string(&subs).unwrap_or_else(|_| "[]".to_string()),
			)
			.ok();
		}
	}

	app.emit("library:updated", ()).ok();
	Ok(())
}

#[tauri::command]
async fn sync_offline_subscriptions_cmd(
	app: AppHandle,
	state: State<'_, AppState>,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let settings_conn = open_settings_conn(&profile_uid);

	let subs: Vec<String> = db::settings_manager::get_setting(&settings_conn, "offline_subscriptions")
		.ok()
		.flatten()
		.and_then(|v| serde_json::from_str(&v).ok())
		.unwrap_or_default();

	for sub_uid in subs {
		match sub_uid.chars().take(2).collect::<String>().as_str() {
			"a-" => {
				let _ = download_album_tracks_cmd(app.clone(), state.clone(), sub_uid, false).await;
			}
			"p-" => {
				let _ = download_playlist_tracks_cmd(app.clone(), state.clone(), sub_uid, false).await;
			}
			_ => {}
		}
	}

	Ok(())
}

#[tauri::command]
async fn unsubscribe_offline_cmd(
	state: State<'_, AppState>,
	uid: String,
) -> Result<(), String> {
	let profile_uid = state.get_uid();
	let settings_conn = open_settings_conn(&profile_uid);

	let mut subs: Vec<String> = db::settings_manager::get_setting(&settings_conn, "offline_subscriptions")
		.ok()
		.flatten()
		.and_then(|v| serde_json::from_str(&v).ok())
		.unwrap_or_default();

	subs.retain(|s| s != &uid);

	db::settings_manager::set_setting(
		&settings_conn,
		"offline_subscriptions",
		&serde_json::to_string(&subs).unwrap_or_else(|_| "[]".to_string()),
	)
	.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────
// SPOTIFY
// ─────────────────────────────────────────────

#[tauri::command]
async fn spotify_get_playlist_info_cmd(
	playlist_id: String,
	state: State<'_, AppState>,
) -> Result<spotify_auth::SpotifyPlaylistInfo, String> {
	let uid = state.get_uid();
	spotify_auth::spotify_get_playlist_info(&uid, &playlist_id).await
}

#[tauri::command]
fn spotify_get_auth_url(state: State<'_, AppState>) -> Result<(String, String), String> {
    let uid = state.get_uid();
    let conn = open_settings_conn(&uid);
    let client_id = db::settings_manager::get_setting(&conn, "spotify_client_id")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    if client_id.is_empty() {
        return Err(
            "Spotify client ID not configured. Add it in Settings → Connected Accounts.".into(),
        );
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
async fn spotify_enrich_track_cmd(uid: String, state: State<'_, AppState>) -> Result<(), String> {
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

    let client = enrichment::make_client()?;
    let Some(meta) = enrichment::spotify::enrich_track(&client, &profile_uid, title, &artist).await else {
        return Ok(());
    };

    let update = db::MetadataUpdate {
        title: meta.title,
        artists: meta
            .artists
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default()),
        year: meta.year,
        genres: meta
            .genres
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default()),
        ..Default::default()
    };

	if let Some(ref genres_json) = update.genres {
		if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
			for name in names {
				crate::db::tag_manager::ensure_tag(
					&lib_conn,
					&name,
					crate::db::tag_manager::TagKind::Genre,
				);
			}
		}
	}
	db::track_manager::update_track_metadata_by_uid(&lib_conn, &uid, &update)
		.map_err(|e| e.to_string())
}

#[tauri::command]
async fn spotify_enrich_album_cmd(uid: String, state: State<'_, AppState>) -> Result<(), String> {
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

    let client = enrichment::make_client()?;
    let Some(meta) = enrichment::spotify::enrich_album(&client, &profile_uid, &album.title, &artist).await else {
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
        genres: meta
            .genres
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default()),
        tracks: None,
        credits: None,
        label: meta.label,
        artwork_blob: None,
        artwork_path: None,
        emulate_type: None,
    };

	if let Some(ref genres_json) = update.genres {
		if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
			for name in names {
				crate::db::tag_manager::ensure_tag(
					&lib_conn,
					&name,
					crate::db::tag_manager::TagKind::Genre,
				);
			}
		}
	}
	db::album_manager::update_album_by_uid(&lib_conn, &uid, &update).map_err(|e| e.to_string())
}

#[tauri::command]
async fn spotify_enrich_artist_cmd(uid: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_uid = state.get_uid();
    let lib_conn = open_lib_conn(&profile_uid);
    let artist = db::artist_manager::get_artist_by_uid(&lib_conn, &uid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Artist not found: {}", uid))?;

    let client = enrichment::make_client()?;
    let Some(meta) = enrichment::spotify::enrich_artist(&client, &profile_uid, &artist.name).await else {
        return Ok(());
    };

    let update = db::ArtistUpdate {
        name: None,
        aka: None,
        about: None,
        tags: None,
        genres: meta
            .genres
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default()),
        websites: None,
        members: None,
        profile_art_blob: None,
        profile_art_path: None,
        banner_art_blob: None,
        banner_art_path: None,
    };

	if let Some(ref genres_json) = update.genres {
		if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
			for name in names {
				crate::db::tag_manager::ensure_tag(
					&lib_conn,
					&name,
					crate::db::tag_manager::TagKind::Genre,
				);
			}
		}
	}
	db::artist_manager::update_artist_by_uid(&lib_conn, &uid, &update).map_err(|e| e.to_string())
}

// ── Tags ──────────────────────────────────────────────────────────────────────

/// Return every tag and genre in the dictionary, ordered by kind then name.
#[tauri::command]
async fn get_all_tags_cmd(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let uid = state.get_uid();
    let conn = open_lib_conn(&uid);
    tag_manager::get_all_tags(&conn).map_err(|e| e.to_string())
}

/// Add a new tag (kind = "tag") to the dictionary.
/// Returns the uid of the created (or already-existing) tag.
#[tauri::command]
async fn add_tag_cmd(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<String, String> {
    let uid = state.get_uid();
    let conn = open_lib_conn(&uid);
    tag_manager::add_tag(&conn, &name, TagKind::Tag, color.as_deref()).map_err(|e| e.to_string())
}

/// Add a new genre to the dictionary.
#[tauri::command]
async fn add_genre_cmd(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<String, String> {
    let uid = state.get_uid();
    let conn = open_lib_conn(&uid);
    tag_manager::add_tag(&conn, &name, TagKind::Genre, color.as_deref()).map_err(|e| e.to_string())
}

/// Rename a tag/genre and propagate the change to every entity that uses it.
#[tauri::command]
async fn rename_tag_cmd(
    state: State<'_, AppState>,
    uid: String,
    new_name: String,
) -> Result<(), String> {
    let profile_uid = state.get_uid();
    let conn = open_lib_conn(&profile_uid);
    tag_manager::rename_tag(&conn, &uid, &new_name).map_err(|e| e.to_string())
}

/// Delete a tag/genre and remove it from every entity that uses it.
#[tauri::command]
async fn delete_tag_cmd(state: State<'_, AppState>, uid: String) -> Result<(), String> {
    let profile_uid = state.get_uid();
    let conn = open_lib_conn(&profile_uid);
    tag_manager::delete_tag(&conn, &uid).map_err(|e| e.to_string())
}

/// Update the display color of a tag/genre (UI hint only, not stored on entities).
#[tauri::command]
async fn update_tag_color_cmd(
    state: State<'_, AppState>,
    uid: String,
    color: Option<String>,
) -> Result<(), String> {
    let profile_uid = state.get_uid();
    let conn = open_lib_conn(&profile_uid);
    tag_manager::update_tag_color(&conn, &uid, color.as_deref()).map_err(|e| e.to_string())
}

// ── Tag Groups ────────────────────────────────────────────────────────────────

/// Return all tag groups with their member uid lists.
#[tauri::command]
async fn get_tag_groups_cmd(state: State<'_, AppState>) -> Result<Vec<TagGroup>, String> {
    let uid = state.get_uid();
    let conn = open_lib_conn(&uid);
    tag_manager::get_all_tag_groups(&conn).map_err(|e| e.to_string())
}

/// Create a new tag group.
///
/// `kind`       — "tag" or "genre"
/// `member_uids` — list of tag UIDs to include
#[tauri::command]
async fn create_tag_group_cmd(
    state: State<'_, AppState>,
    name: String,
    kind: String,
    color: Option<String>,
    member_uids: Vec<String>,
) -> Result<TagGroup, String> {
    let profile_uid = state.get_uid();
    let conn = open_lib_conn(&profile_uid);
    tag_manager::create_tag_group(
        &conn,
        &name,
        TagKind::from_str(&kind),
        color.as_deref(),
        &member_uids,
    )
    .map_err(|e| e.to_string())
}

/// Update an existing tag group's name, color, and/or member list.
/// Pass `null` for any field you don't want to change.
#[tauri::command]
async fn update_tag_group_cmd(
    state: State<'_, AppState>,
    uid: String,
    name: Option<String>,
    color: Option<Option<String>>, // Some(None) = clear color, None = don't touch
    member_uids: Option<Vec<String>>,
) -> Result<(), String> {
    let profile_uid = state.get_uid();
    let conn = open_lib_conn(&profile_uid);
    tag_manager::update_tag_group(
        &conn,
        &uid,
        name.as_deref(),
        color.as_ref().map(|c| c.as_deref()),
        member_uids.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Delete a tag group (does NOT delete the member tags themselves).
#[tauri::command]
async fn delete_tag_group_cmd(state: State<'_, AppState>, uid: String) -> Result<(), String> {
    let profile_uid = state.get_uid();
    let conn = open_lib_conn(&profile_uid);
    tag_manager::delete_tag_group(&conn, &uid).map_err(|e| e.to_string())
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
        .arg(
            std::path::Path::new(&path)
                .parent()
                .unwrap_or(std::path::Path::new("/")),
        )
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
                        let result =
                            spotify_auth::spotify_exchange_code(&uid, &code, &verifier).await;
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
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // On Windows the deep-link URL arrives here as a CLI argument
            if let Some(url) = argv.iter().find(|a| a.starts_with("sonodex://")) {
                handle_deep_link(app.clone(), url);
            }
            // Bring the existing window to front
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(app_state)
        .plugin(tauri_plugin_media::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let handle = app.handle().clone();
            let uid = app.state::<AppState>().get_uid();

            if !initial_uid.is_empty() {
                let settings_conn = Connection::open(get_settings_db_path(&initial_uid))
                    .expect("Failed to open settings db");
                init_settings_db(&settings_conn).expect("Failed to init settings db");

                let lib_conn = Connection::open(get_lib_db_path(&initial_uid))
                    .expect("Failed to open lib db");
                db::init_lib_db(&lib_conn).expect("Failed to init lib db");

                // Bootstrap federated library structure and run on-load merge
                library_manager::ensure_default_library(&initial_uid)
                    .expect("Failed to ensure default library");
                library_manager::on_load_sync(&initial_uid)
                    .unwrap_or_else(|e| {
                        eprintln!("[startup] merge sync failed: {e}");
                        library_manager::MergeResult { rebuilt: false, libraries_processed: 0 }
                    });
            }

            if !uid.is_empty() {
                let settings_conn = open_settings_conn(&uid);
                let default_lib_uid = db::library_registry::get_default_library(&settings_conn)
                    .unwrap_or(None)
                    .map(|l| l.uid)
                    .unwrap_or_default();

                let paths = get_library_paths(&settings_conn)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| p.path)
                    .collect();

                watcher::start_watcher(handle.clone(), uid, default_lib_uid, paths);
            }

            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    handle_deep_link(handle.clone(), url.as_str());
                }
            });

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_window_state::Builder::default().build())?;

            /* let window = app.get_window("main").unwrap();

                       #[cfg(target_os = "macos")]
                       apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                               .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

                       #[cfg(target_os = "windows")]
                       apply_blur(&window, Some((18, 18, 18, 125)))
                               .expect("Unsupported platform! 'apply_blur' is only supported on Windows");
            */
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
            set_profile_password,
            remove_profile_password,
            verify_profile_password,
            verify_recovery_key,
            profile_has_password,
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
            spotify_get_playlist_info_cmd,
            add_uid_remap,
            resolve_uid,
            replace_track_path,
            merge_remote_local_tracks,
            open_in_explorer,
            get_all_tags_cmd,
            add_tag_cmd,
            add_genre_cmd,
            rename_tag_cmd,
            delete_tag_cmd,
            update_tag_color_cmd,
            get_tag_groups_cmd,
            create_tag_group_cmd,
            update_tag_group_cmd,
            delete_tag_group_cmd,
            download_track_cmd,
            download_album_tracks_cmd,
            download_playlist_tracks_cmd,
            sync_offline_subscriptions_cmd,
            unsubscribe_offline_cmd,
            listenbrainz_connection_status,
            listenbrainz_validate_token_cmd,
            listenbrainz_connect_cmd,
            listenbrainz_disconnect_cmd,
            import_spotify_history_cmd,
            get_libraries,
            create_library_cmd,
            import_library_cmd,
            export_library_cmd,
            update_library_cmd,
            delete_library_cmd,
            sync_library_cmd,
            push_library_cmd,
            check_write_permission_cmd,
            rebuild_merged_cmd,
            get_blocklist,
            add_to_blocklist_cmd,
            remove_from_blocklist_cmd,
            get_delete_preference_cmd,
            set_delete_preference_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
