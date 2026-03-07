use lofty::file::AudioFile;
use crate::db::{Track, upsert_track};
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rusqlite::Connection;
use std::path::Path;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "aac", "wav", "aiff", "ogg"];

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn get_last_modified(path: &Path) -> i64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn read_track(path: &Path) -> Option<Track> {
    let last_modified = get_last_modified(path);
    let path_str = path.to_string_lossy().to_string();

    let tagged_file = Probe::open(path)
        .ok()?
        .guess_file_type()
        .ok()?
        .read()
        .ok()?;

    let properties = tagged_file.properties();
    let duration_ms = Some(properties.duration().as_millis() as i64);

    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    let (title, artist, album, album_artist, genre, year, track_number, bpm, key, artwork) =
        if let Some(tag) = tag {
            let artwork = tag.pictures().first().map(|p| p.data().to_vec());
            (
                tag.title().map(|s| s.to_string()),
                tag.artist().map(|s| s.to_string()),
                tag.album().map(|s| s.to_string()),
                tag.get_string(&lofty::tag::ItemKey::AlbumArtist).map(|s| s.to_string()),
                tag.genre().map(|s| s.to_string()),
                tag.year().map(|y| y as i32),
                tag.track().map(|t| t as i32),
                tag.get_string(&lofty::tag::ItemKey::Bpm)
                    .and_then(|s| s.parse::<f32>().ok()),
                tag.get_string(&lofty::tag::ItemKey::InitialKey).map(|s| s.to_string()),
                artwork,
            )
        } else {
            (None, None, None, None, None, None, None, None, None, None)
        };

    Some(Track {
        id: None,
        path: path_str,
        last_modified,
        title,
        artist,
        album,
        album_artist,
        genre,
        year,
        duration_ms,
        bpm,
        key,
        track_number,
        artwork,
    })
}

pub fn scan_directory(conn: &Connection, dir: &str) -> (usize, usize) {
    let mut scanned = 0;
    let mut failed = 0;

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if !is_supported(path) {
            continue;
        }

        match read_track(path) {
            Some(track) => {
                if upsert_track(conn, &track).is_ok() {
                    scanned += 1;
                } else {
                    failed += 1;
                }
            }
            None => {
                failed += 1;
            }
        }
    }

    (scanned, failed)
}