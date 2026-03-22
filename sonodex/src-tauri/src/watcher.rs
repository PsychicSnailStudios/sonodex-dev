use crate::db::delete_track;
use crate::profiles::{get_lib_db_path, get_settings_db_path};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn start_watcher(app: AppHandle, profile_uid: String, paths: Vec<String>) {
	std::thread::spawn(move || {
		let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

		let mut watcher = RecommendedWatcher::new(
			tx,
			Config::default().with_poll_interval(Duration::from_secs(2)),
		)
		.expect("Failed to create watcher");

		for path in &paths {
			watcher
				.watch(PathBuf::from(path).as_path(), RecursiveMode::Recursive)
				.ok();
		}

		let mut debounce: HashMap<PathBuf, std::time::Instant> = HashMap::new();
		let debounce_duration = Duration::from_millis(500);

		loop {
			match rx.recv_timeout(Duration::from_secs(1)) {
				Ok(Ok(event)) => {
					let now = std::time::Instant::now();

					for path in &event.paths {
						let last = debounce.get(path).copied();
						if last
							.map(|l| now.duration_since(l) < debounce_duration)
							.unwrap_or(false)
						{
							continue;
						}
						debounce.insert(path.clone(), now);

						let lib_path = get_lib_db_path(&profile_uid);
						let conn = Connection::open(&lib_path).unwrap();

						match event.kind {
							EventKind::Create(_) | EventKind::Modify(_) => {
								if let Some(track) = crate::scanner::read_track(path) {
									crate::db::upsert_track(&conn, &track).ok();
									app.emit("library:updated", ()).ok();
								}
							}
							EventKind::Remove(_) => {
								let path_str = path.to_string_lossy().to_string();
								delete_track(&conn, &path_str).ok();
								app.emit("library:updated", ()).ok();
							}
							_ => {}
						}
					}
				}
				Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
				Err(mpsc::RecvTimeoutError::Disconnected) => break,
			}
		}
	});
}