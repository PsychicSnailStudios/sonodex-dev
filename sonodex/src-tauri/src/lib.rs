mod db;
mod scanner;
mod watcher;

use db::{
    add_library_path, get_all_tracks, get_db_path, get_library_paths, init_db, remove_library_path,
    LibraryPath, Track,
};
use rusqlite::Connection;
use tauri::AppHandle;

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
fn get_tracks() -> Result<Vec<Track>, String> {
    let conn = open_conn();
    get_all_tracks(&conn).map_err(|e| e.to_string())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = open_conn();
    init_db(&conn).expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            add_path,
            remove_path,
            get_tracks,
            get_paths,
            rescan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}