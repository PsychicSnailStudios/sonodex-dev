use crate::profiles::get_lib_db_path;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "wav", "aac", "m4a", "opus", "aiff"];

fn is_audio_file(path: &PathBuf) -> bool {
	path.extension()
		.and_then(|e| e.to_str())
		.map(|e| AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
		.unwrap_or(false)
}

fn is_remote_path(path: &str) -> bool {
	path.starts_with("http://")
		|| path.starts_with("https://")
		|| path.starts_with("\\\\")
        || path.starts_with("//")
}

pub fn start_watcher(app: AppHandle, profile_uid: String, paths: Vec<String>) {
	std::thread::spawn(move || {
		let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
		let mut watcher = RecommendedWatcher::new(
			tx,
			Config::default().with_poll_interval(Duration::from_secs(2)),
		)
		.expect("Failed to create watcher");

		for path in &paths {
			if let Err(e) = watcher.watch(PathBuf::from(path).as_path(), RecursiveMode::Recursive) {
				eprintln!("[watcher] Failed to watch {path}: {e}");
			} else {
				eprintln!("[watcher] Watching: {path}");
			}
		}

		// Debounce map: path -> (last_seen, event_kind)
		// We track the kind so a rapid create+modify pair collapses to one upsert
		let mut debounce: HashMap<PathBuf, (Instant, EventKind)> = HashMap::new();
		let debounce_duration = Duration::from_millis(3000);

		loop {
			match rx.recv_timeout(Duration::from_millis(500)) {
				Ok(Ok(event)) => {
					let now = Instant::now();

					for path in &event.paths {
						if !is_audio_file(path) {
							continue;
						}

						let prev = debounce.get(path);
						let too_soon = prev
							.map(|(last, _)| now.duration_since(*last) < debounce_duration)
							.unwrap_or(false);

						if too_soon {
							// Still within debounce window — update timestamp but keep earliest kind
							// so a create event isn't downgraded to a modify
							let kind = match prev.map(|(_, k)| k) {
								Some(EventKind::Create(_)) => event.kind.clone(),
								_ => event.kind.clone(),
							};
							debounce.insert(path.clone(), (now, kind));
							continue;
						}

						debounce.insert(path.clone(), (now, event.kind.clone()));
					}
				}
				Ok(Err(e)) => eprintln!("[watcher] notify error: {e}"),
				Err(mpsc::RecvTimeoutError::Timeout) => {
					// Flush any debounced events whose window has expired
					let now = Instant::now();
					let ready: Vec<(PathBuf, EventKind)> = debounce
						.iter()
						.filter(|(_, (last, _))| now.duration_since(*last) >= debounce_duration)
						.map(|(p, (_, k))| (p.clone(), k.clone()))
						.collect();

					for (path, kind) in ready {
						debounce.remove(&path);
						process_event(&app, &profile_uid, &path, kind);
					}
				}
				Err(mpsc::RecvTimeoutError::Disconnected) => {
					eprintln!("[watcher] channel disconnected, exiting");
					break;
				}
			}
		}
	});
}

fn process_event(app: &AppHandle, profile_uid: &str, path: &PathBuf, kind: EventKind) {
	let lib_path = get_lib_db_path(profile_uid);
	let conn = match Connection::open(&lib_path) {
		Ok(c) => c,
		Err(e) => {
			eprintln!("[watcher] DB open error: {e}");
			return;
		}
	};

	let settings_path = crate::profiles::get_settings_db_path(profile_uid);
	if let Err(e) = conn.execute_batch(&format!(
		"ATTACH DATABASE '{}' AS settings;",
		settings_path.to_string_lossy().replace('\'', "''")
	)) {
		eprintln!("[watcher] Failed to attach settings.db: {e}");
	}

	match kind {
		EventKind::Create(_) | EventKind::Modify(_) => {
            // Wait and confirm file size has stabilised (handles mid-paste duplicates)
            let path_str = path.to_string_lossy().to_string();
            if !is_remote_path(&path_str) {
                let mut last_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                for _ in 0..6 {
                    std::thread::sleep(Duration::from_millis(500));
                    let current_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    if current_size == last_size && current_size > 0 {
                        break;
                    }
                    last_size = current_size;
                }
                if !path.exists() || std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) == 0 {
                    eprintln!("[watcher] Path gone or empty after wait: {:?}", path);
                    return;
                }
            }

			let g = |key: &str, default: &str| -> String {
				crate::db::get_setting(&conn, key)
					.ok()
					.flatten()
					.unwrap_or_else(|| default.to_string())
			};

			let priority_title = g("filename_priority_title", "tag");
			let priority_artist = g("filename_priority_artist", "tag");
			let priority_album = g("filename_priority_album", "tag");
			let priority_year = g("filename_priority_year", "tag");
			let custom_pattern = g("filename_custom_pattern", "");
			let try_ampersand = g("scan_try_parse_ampersand", "true") == "true";

			let tag_delim_raw = g("artist_tag_delimiters", " / |; |, |,");
			let filename_delim_raw = g("artist_filename_delimiters", " / |; | feat. | ft. | featuring ");
			let genre_delim_raw = g("genre_delimiters", " / |; |, ");

			let tag_delims: Vec<String> = tag_delim_raw.split('|').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
			let filename_delims: Vec<String> = filename_delim_raw.split('|').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
			let genre_delims: Vec<String> = genre_delim_raw.split('|').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();

			let tag_delims_ref: Vec<&str> = tag_delims.iter().map(|s| s.as_str()).collect();
			let filename_delims_ref: Vec<&str> = filename_delims.iter().map(|s| s.as_str()).collect();
			let genre_delims_ref: Vec<&str> = genre_delims.iter().map(|s| s.as_str()).collect();

			let track_opt = crate::scanner::read_track_with_settings(
				path,
				&priority_title,
				&priority_artist,
				&priority_album,
				&priority_year,
				&custom_pattern,
				&tag_delims_ref,
				&filename_delims_ref,
				&genre_delims_ref,
				try_ampersand,
			);

			match track_opt {
				Some(mut track) => {
					// Route path to remote_path if it's a remote source
					if is_remote_path(&track.path) {
						track.remote_path = Some(track.path.clone());
						track.path = String::new();
					}

					match crate::scanner::find_remote_local_counterpart(&conn, &track) {
                        Some(existing_uid) => {
                            crate::scanner::merge_paths_into_existing(&conn, &existing_uid, &track);
                            app.emit("library:updated", ()).ok();
                            return;
                        }
                        None => {}
                    }
                    match crate::db::upsert_track(&conn, &track) {
                        Ok(_) => {
                            crate::scanner::process_track(&conn, &track);

							if let Some(ref tags_json) = track.tags {
								if let Ok(names) = serde_json::from_str::<Vec<String>>(tags_json) {
									for name in names {
										crate::db::tag_manager::ensure_tag(&conn, &name, crate::db::tag_manager::TagKind::Tag);
									}
								}
							}
							if let Some(ref genres_json) = track.genres {
								if let Ok(names) = serde_json::from_str::<Vec<String>>(genres_json) {
									for name in names {
										crate::db::tag_manager::ensure_tag(&conn, &name, crate::db::tag_manager::TagKind::Genre);
									}
								}
							}

							eprintln!("[watcher] Upserted + processed: {:?}", path);
							app.emit("library:updated", ()).ok();
						}
						Err(e) => eprintln!("[watcher] upsert_track error: {e}"),
					}
				}
				None => eprintln!("[watcher] read_track returned None for: {:?}", path),
			}
		}
		EventKind::Remove(_) => {
			let path_str = path.to_string_lossy().to_string();
			// For remote paths, delete by remote_path instead of path
			if is_remote_path(&path_str) {
				conn.execute(
					"DELETE FROM tracks WHERE remote_path = ?1",
					rusqlite::params![path_str],
				).ok();
			} else {
				crate::db::delete_track(&conn, &path_str).ok();
			}
			app.emit("library:updated", ()).ok();
			eprintln!("[watcher] Deleted: {:?}", path);
		}
		_ => {}
	}
}