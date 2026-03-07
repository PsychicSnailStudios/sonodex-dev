use crate::db::{get_setting, Track, upsert_track};
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rusqlite::Connection;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "aac", "wav", "aiff", "ogg"];

const ARTIST_DELIMITERS: &[&str] = &[" ft. ", " feat. ", " & ", " x ", "/", ";"];
const GENRE_DELIMITERS: &[&str] = &["/", ";", ","];

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

fn split_values(input: &str, delimiters: &[&str]) -> Vec<String> {
    let mut results = vec![input.to_string()];
    for delim in delimiters {
        results = results
            .into_iter()
            .flat_map(|s| s.split(delim).map(|p| p.trim().to_string()).collect::<Vec<_>>())
            .filter(|s| !s.is_empty())
            .collect();
    }
    results
}

fn normalize_rating(raw: &str) -> Option<f32> {
    let val = raw.trim().parse::<f32>().ok()?;
    if val <= 10.0 {
        Some(val)
    } else if val <= 255.0 {
        Some((val / 255.0) * 10.0)
    } else {
        None
    }
}

#[derive(Debug)]
struct FilenameMetadata {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<String>,
}

fn parse_filename(path: &Path, custom_pattern: &str) -> FilenameMetadata {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    if !custom_pattern.is_empty() {
        return parse_custom_pattern(&stem, custom_pattern);
    }

    let patterns: &[&[&str]] = &[
        &["{artist}", "{album}", "{year}", "{title}"],
        &["{year}", "{artist}", "{album}", "{title}"],
        &["{artist}", "{title}"],
        &["{title}"],
    ];

    for pattern in patterns {
        if let Some(meta) = try_parse_pattern(&stem, pattern) {
            return meta;
        }
    }

    FilenameMetadata { title: Some(stem), artist: None, album: None, year: None }
}

fn try_parse_pattern(stem: &str, pattern: &[&str]) -> Option<FilenameMetadata> {
    let parts: Vec<&str> = stem.splitn(pattern.len(), " - ").collect();
    if parts.len() != pattern.len() {
        return None;
    }

    let mut meta = FilenameMetadata { title: None, artist: None, album: None, year: None };

    for (i, &field) in pattern.iter().enumerate() {
        let value = parts[i].trim().to_string();
        match field {
            "{title}" => meta.title = Some(value),
            "{artist}" => meta.artist = Some(value),
            "{album}" => meta.album = Some(value),
            "{year}" => {
                if value.len() == 4 && value.parse::<u32>().is_ok() {
                    meta.year = Some(value);
                } else {
                    return None;
                }
            }
            _ => {}
        }
    }

    Some(meta)
}

fn parse_custom_pattern(stem: &str, pattern: &str) -> FilenameMetadata {
    let delimiters: Vec<&str> = pattern
        .split(|c: char| c == '{' || c == '}')
        .filter(|s| !["title", "artist", "album", "year"].contains(s) && !s.is_empty())
        .collect();

    let fields: Vec<&str> = pattern
        .split(|c: char| !c.is_alphanumeric() && c != '{' && c != '}')
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| s.trim_matches(|c| c == '{' || c == '}'))
        .collect();

    let mut remaining = stem;
    let mut values: Vec<String> = Vec::new();

    for (i, delim) in delimiters.iter().enumerate() {
        if i < fields.len() {
            if let Some(pos) = remaining.find(delim) {
                values.push(remaining[..pos].trim().to_string());
                remaining = &remaining[pos + delim.len()..];
            } else {
                break;
            }
        }
    }
    values.push(remaining.trim().to_string());

    let mut meta = FilenameMetadata { title: None, artist: None, album: None, year: None };
    for (i, field) in fields.iter().enumerate() {
        if let Some(value) = values.get(i) {
            match *field {
                "title" => meta.title = Some(value.clone()),
                "artist" => meta.artist = Some(value.clone()),
                "album" => meta.album = Some(value.clone()),
                "year" => meta.year = Some(value.clone()),
                _ => {}
            }
        }
    }

    meta
}

fn resolve_field(tag_value: Option<String>, filename_value: Option<String>, priority: &str) -> Option<String> {
    match priority {
        "filename" => filename_value.or(tag_value),
        "filename_fallback" => tag_value.or(filename_value),
        _ => tag_value.or(filename_value),
    }
}

pub fn read_track(path: &Path) -> Option<Track> {
    read_track_with_settings(path, "tag", "tag", "tag", "tag", "")
}

pub fn read_track_with_settings(
    path: &Path,
    priority_title: &str,
    priority_artist: &str,
    priority_album: &str,
    priority_year: &str,
    custom_pattern: &str,
) -> Option<Track> {
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

    let filename_meta = parse_filename(path, custom_pattern);

    let (tag_title, tag_artist, tag_album, tag_album_artist, tag_genres, tag_year, tag_rating, bpm, key, artwork) =
        if let Some(tag) = tag {
            let artwork = tag.pictures().first().map(|p| p.data().to_vec());
            let rating = tag
				.get_string(&lofty::tag::ItemKey::Popularimeter)
				.and_then(|s| normalize_rating(s));
            (
                tag.title().map(|s| s.to_string()),
                tag.artist().map(|s| s.to_string()),
                tag.album().map(|s| s.to_string()),
                tag.get_string(&lofty::tag::ItemKey::AlbumArtist).map(|s| s.to_string()),
                tag.genre().map(|s| s.to_string()),
                tag.year().map(|y| y.to_string()),
                rating,
                tag.get_string(&lofty::tag::ItemKey::Bpm)
                    .and_then(|s| s.parse::<f32>().ok()),
                tag.get_string(&lofty::tag::ItemKey::InitialKey).map(|s| s.to_string()),
                artwork,
            )
        } else {
            (None, None, None, None, None, None, None, None, None, None)
        };

    let title = resolve_field(tag_title, filename_meta.title, priority_title);

    let artist_str = resolve_field(tag_artist, filename_meta.artist, priority_artist);
    let artists = artist_str
        .as_deref()
        .map(|s| {
            let parts = split_values(s, ARTIST_DELIMITERS);
            serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string())
        });

    let album_str = resolve_field(tag_album, filename_meta.album, priority_album);
    let album_artist = tag_album_artist.or_else(|| {
        artists
            .as_deref()
            .and_then(|a| serde_json::from_str::<Vec<String>>(a).ok())
            .and_then(|v| v.into_iter().next())
    });

    let albums = album_str.map(|a| {
        serde_json::to_string(&vec![serde_json::json!({ "name": a, "track_number": null })])
            .unwrap_or_else(|_| "[]".to_string())
    });

    let genres = tag_genres.map(|g| {
        let parts = split_values(&g, GENRE_DELIMITERS);
        serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string())
    });

    let year = resolve_field(tag_year, filename_meta.year, priority_year);

    Some(Track {
        id: None,
        path: path_str,
        last_modified,
        title,
        artists,
        album_artist,
        albums,
        genres,
        year,
        rating: tag_rating,
        tags: None,
        duration_ms,
        bpm,
        key,
        artwork,
    })
}

pub fn scan_directory_with_progress(conn: &Connection, dir: &str, app: &AppHandle) {
    let priority_title = get_setting(conn, "filename_priority_title").ok().flatten().unwrap_or_else(|| "tag".to_string());
    let priority_artist = get_setting(conn, "filename_priority_artist").ok().flatten().unwrap_or_else(|| "tag".to_string());
    let priority_album = get_setting(conn, "filename_priority_album").ok().flatten().unwrap_or_else(|| "tag".to_string());
    let priority_year = get_setting(conn, "filename_priority_year").ok().flatten().unwrap_or_else(|| "tag".to_string());
    let custom_pattern = get_setting(conn, "filename_custom_pattern").ok().flatten().unwrap_or_default();

    let all_files: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_supported(e.path()))
        .collect();

    let total = all_files.len();
    let mut scanned = 0;

    app.emit("scan:progress", serde_json::json!({ "scanned": 0, "total": total })).ok();

    for entry in all_files {
        if let Some(track) = read_track_with_settings(
            entry.path(),
            &priority_title,
            &priority_artist,
            &priority_album,
            &priority_year,
            &custom_pattern,
        ) {
            upsert_track(conn, &track).ok();
        }
        scanned += 1;
        if scanned % 25 == 0 || scanned == total {
            app.emit("scan:progress", serde_json::json!({ "scanned": scanned, "total": total })).ok();
        }
    }

    app.emit("scan:done", ()).ok();
}