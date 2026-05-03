use crate::db::{add_library_path, get_library_paths, remove_library_path, LibraryPath};
use crate::library_manager;
use crate::profiles::{get_lib_db_path, get_library_db_path, get_settings_db_path};
use crate::state::AppState;
use crate::{open_settings_conn, open_local_library_conn};
use rusqlite::Connection;
use tauri::{AppHandle, State};
use tauri::Manager;

fn normalize_path(path: &str) -> String {
	path.replace('\\', "/")
}

#[tauri::command]
pub fn add_path(
	app: AppHandle,
	state: State<AppState>,
	path: String,
	lib_uid: Option<String>,
) -> Result<(), String> {
	let path = normalize_path(&path);
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);

	let target_lib_uid = match lib_uid {
		Some(ref l) => l.clone(),
		None => {
			library_manager::ensure_default_library(&uid)?;
			crate::db::library_registry::get_default_library(&settings_conn)
				.map_err(|e| e.to_string())?
				.ok_or("No default library found")?
				.uid
		}
	};

	add_library_path(&settings_conn, &path).map_err(|e| e.to_string())?;
	settings_conn
		.execute(
			"UPDATE library_paths SET lib_uid = ?1 WHERE path = ?2",
			rusqlite::params![target_lib_uid, path],
		)
		.ok();

	let app_clone = app.clone();
	let path_clone = path.clone();
	let uid_clone = uid.clone();
	let lib_uid_clone = target_lib_uid.clone();

	std::thread::spawn(move || {
		let lib_path = get_library_db_path(&uid_clone, &lib_uid_clone);
		let lib_conn = Connection::open(&lib_path).expect("Failed to open library db");
		let settings_path = get_settings_db_path(&uid_clone);
		lib_conn
			.execute_batch(&format!(
				"ATTACH DATABASE '{}' AS settings;",
				settings_path.to_string_lossy().replace('\'', "''")
			))
			.ok();
		crate::scanner::scan_directory_with_progress(&lib_conn, &path_clone, &app_clone);

		let _ = library_manager::incremental_update(&uid_clone, &[lib_uid_clone.clone()]);

		let settings_conn2 = open_settings_conn(&uid_clone);
		let auto_tracks = crate::db::get_setting(&settings_conn2, "auto_enrich_tracks")
			.ok()
			.flatten()
			.map(|v| v == "true")
			.unwrap_or(false);
		let auto_albums = crate::db::get_setting(&settings_conn2, "auto_enrich_albums")
			.ok()
			.flatten()
			.map(|v| v == "true")
			.unwrap_or(false);
		let auto_artists = crate::db::get_setting(&settings_conn2, "auto_enrich_artists")
			.ok()
			.flatten()
			.map(|v| v == "true")
			.unwrap_or(false);

		if auto_tracks || auto_albums || auto_artists {
			let app_enrich = app_clone.clone();
			tauri::async_runtime::spawn(async move {
				let state = app_enrich.state::<AppState>();
				if auto_tracks {
					let _ = crate::commands::enrichment::enrich_all(app_enrich.clone(), state.clone()).await;
				}
				if auto_albums {
					let _ = crate::commands::enrichment::enrich_all_albums(app_enrich.clone(), state.clone()).await;
				}
				if auto_artists {
					let _ = crate::commands::enrichment::enrich_all_artists(app_enrich.clone(), state.clone()).await;
				}
			});
		}

		let settings_conn3 = open_settings_conn(&uid_clone);
		let paths = get_library_paths(&settings_conn3)
			.unwrap_or_default()
			.into_iter()
			.map(|p| p.path)
			.collect();
		crate::watcher::start_watcher(app_clone, uid_clone, lib_uid_clone, paths);
	});

	Ok(())
}

#[tauri::command]
pub fn remove_path(state: State<AppState>, path: String) -> Result<(), String> {
	let path = normalize_path(&path);
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);

	let lib_uid: Option<String> = settings_conn
		.query_row(
			"SELECT lib_uid FROM library_paths WHERE path = ?1",
			rusqlite::params![path],
			|row| row.get(0),
		)
		.ok();

	remove_library_path(&settings_conn, &path).map_err(|e| e.to_string())?;

	if let Some(ref luid) = lib_uid {
		let lib_path = get_library_db_path(&uid, luid);
		if let Ok(lib_conn) = Connection::open(&lib_path) {
			lib_conn
				.execute(
					"DELETE FROM tracks WHERE path LIKE ?1",
					rusqlite::params![format!("{}%", path)],
				)
				.ok();
			lib_conn.execute(
				"DELETE FROM albums WHERE uid NOT IN (
					SELECT DISTINCT json_extract(json_each.value, '$.uid')
					FROM tracks, json_each(tracks.albums)
					WHERE json_extract(json_each.value, '$.uid') IS NOT NULL
					AND json_extract(json_each.value, '$.uid') != ''
				)",
				[],
			).ok();
			lib_conn.execute(
				"DELETE FROM artists WHERE name NOT IN (
					SELECT DISTINCT json_each.value
					FROM tracks, json_each(tracks.artists)
					WHERE tracks.artists IS NOT NULL AND tracks.artists != '[]'
				) AND name NOT IN (
					SELECT DISTINCT album_artist FROM tracks WHERE album_artist IS NOT NULL
				)",
				[],
			).ok();
			let _ = library_manager::incremental_update(&uid, &[luid.clone()]);
		}
	} else {
		if let Ok(lib_conn) = Connection::open(get_lib_db_path(&uid)) {
			lib_conn
				.execute(
					"DELETE FROM tracks WHERE path LIKE ?1",
					rusqlite::params![format!("{}%", path)],
				)
				.ok();
		}
	}

	Ok(())
}

#[tauri::command]
pub fn get_paths(state: State<AppState>) -> Result<Vec<LibraryPath>, String> {
	let uid = state.get_uid();
	let conn = open_settings_conn(&uid);
	get_library_paths(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rescan(app: AppHandle, state: State<AppState>) -> Result<(), String> {
	let uid = state.get_uid();
	let settings_conn = open_settings_conn(&uid);
	let paths = get_library_paths(&settings_conn).map_err(|e| e.to_string())?;

	let default_lib_uid = crate::db::library_registry::get_default_library(&settings_conn)
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
		lib_conn
			.execute_batch(&format!(
				"ATTACH DATABASE '{}' AS settings;",
				settings_path.to_string_lossy().replace('\'', "''")
			))
			.ok();

		for p in &path_strings {
			crate::scanner::scan_directory_with_progress(&lib_conn, p, &app_clone);
		}

		let _ = library_manager::incremental_update(&uid_clone, &[lib_uid_clone.clone()]);
		crate::watcher::start_watcher(app_clone, uid_clone, lib_uid_clone, path_strings);
	});

	Ok(())
}
