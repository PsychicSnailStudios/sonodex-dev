mod db;
mod scanner;
mod watcher;

use db::{
  add_library_path, get_all_tracks, get_db_path, get_library_paths, init_db, remove_library_path,
  LibraryPath, Track,
};
use rusqlite::Connection;
use tauri::{AppHandle, Emitter};

fn open_conn() -> Connection {
    Connection::open(get_db_path()).expect("Failed to open database")
}

#[tauri::command]
fn add_path(app: AppHandle, path: String) -> Result<(), String> {
    let conn = open_conn();
    add_library_path(&conn, &path).map_err(|e| e.to_string())?;

    let app_clone = app.clone();
    let path_clone = path.clone();
    std::thread::spawn(move || {
        eprintln!("Starting scan of: {}", path_clone);
        let conn = open_conn();
        scanner::scan_directory_with_progress(&conn, &path_clone, &app_clone);
        eprintln!("Scan complete");
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
            get_tracks,
            get_paths,
            rescan,
            get_settings,
            save_setting,
            get_track_artwork,
            get_duplicates,
            remove_track_from_library,
            delete_track_file,
            update_track_metadata,
            write_track_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}