use crate::db::delete_track;
use crate::profiles::get_lib_db_path;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "wav", "aac", "m4a", "opus", "aiff"];

fn is_audio_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
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

        let mut debounce: HashMap<PathBuf, std::time::Instant> = HashMap::new();
        let debounce_duration = Duration::from_millis(500);

        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(Ok(event)) => {
                    eprintln!("[watcher] Event: {:?}", event);

                    let now = std::time::Instant::now();

                    for path in &event.paths {
                        // Skip non-audio files early
                        if !is_audio_file(path) {
                            continue;
                        }

                        // Debounce
                        if debounce
                            .get(path)
                            .map(|&l| now.duration_since(l) < debounce_duration)
                            .unwrap_or(false)
                        {
                            continue;
                        }
                        debounce.insert(path.clone(), now);

                        let lib_path = get_lib_db_path(&profile_uid);
                        let conn = match Connection::open(&lib_path) {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("[watcher] DB open error: {e}");
                                continue;
                            }
                        };

                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                // Wait briefly for the file to be fully written
                                std::thread::sleep(Duration::from_millis(300));

                                if !path.exists() {
                                    eprintln!("[watcher] Path gone after delay: {:?}", path);
                                    continue;
                                }

                                match crate::scanner::read_track(path) {
                                    Some(track) => {
                                        match crate::db::upsert_track(&conn, &track) {
                                            Ok(_) => {
                                                crate::scanner::process_track(&conn, &track);
                                                eprintln!("[watcher] Upserted + processed: {:?}", path);
                                                app.emit("library:updated", ()).ok();
                                            }
                                            Err(e) => {
                                                eprintln!("[watcher] upsert_track error: {e}");
                                            }
                                        }
                                    }
                                    None => {
                                        eprintln!("[watcher] read_track returned None for: {:?}", path);
                                    }
                                }
                            }
                            EventKind::Remove(_) => {
                                let path_str = path.to_string_lossy().to_string();
                                delete_track(&conn, &path_str).ok();
                                app.emit("library:updated", ()).ok();
                                eprintln!("[watcher] Deleted: {:?}", path);
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Err(e)) => eprintln!("[watcher] notify error: {e}"),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("[watcher] channel disconnected, exiting");
                    break;
                }
            }
        }
    });
}
