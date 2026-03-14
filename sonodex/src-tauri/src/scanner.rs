use crate::db::{
    create_album, create_artist, get_all_albums, get_all_artists, get_setting, update_album,
    update_artist, upsert_track, Album, AlbumUpdate, Artist, ArtistUpdate, Track,
};
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rusqlite::Connection;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "aac", "wav", "aiff", "ogg"];

// Safe delimiters for splitting artists in tag fields.
// " / " and "; " are the only separators standardised by ID3/MusicBrainz Picard
// that do not conflict with real artist names.
const ARTIST_TAG_DELIMITERS: &[&str] = &[" / ", "; "];

// Additional delimiters safe for splitting artists found in filenames only.
// feat./ft./featuring are common filename conventions and don't appear in band names.
const ARTIST_FILENAME_DELIMITERS: &[&str] = &[" / ", "; ", " feat. ", " ft. ", " featuring "];

// Delimiters used to split genre strings. Comma is included here because
// genre fields are less likely to contain commas as part of the genre name.
const GENRE_DELIMITERS: &[&str] = &[" / ", "; ", ", "];

// ─────────────────────────────────────────────
// UTILITIES
// ─────────────────────────────────────────────

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

/// Splits a string on any of the provided delimiters.
fn split_on_delimiters(input: &str, delimiters: &[&str]) -> Vec<String> {
    let mut results = vec![input.to_string()];
    for delim in delimiters {
        results = results
            .into_iter()
            .flat_map(|s| {
                s.split(delim)
                    .map(|p| p.trim().to_string())
                    .collect::<Vec<_>>()
            })
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

fn resolve_field(
    tag_value: Option<String>,
    filename_value: Option<String>,
    priority: &str,
) -> Option<String> {
    let tag_value = tag_value.filter(|s| !s.trim().is_empty());
    let filename_value = filename_value.filter(|s| !s.trim().is_empty());
    match priority {
        "filename" => filename_value.or(tag_value),
        _ => tag_value.or(filename_value),
    }
}

// ─────────────────────────────────────────────
// FILENAME PARSING
// ─────────────────────────────────────────────

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

    FilenameMetadata {
        title: Some(stem),
        artist: None,
        album: None,
        year: None,
    }
}

fn try_parse_pattern(stem: &str, pattern: &[&str]) -> Option<FilenameMetadata> {
    let parts: Vec<&str> = stem.splitn(pattern.len(), " - ").collect();
    if parts.len() != pattern.len() {
        return None;
    }

    let mut meta = FilenameMetadata {
        title: None,
        artist: None,
        album: None,
        year: None,
    };

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

    let mut meta = FilenameMetadata {
        title: None,
        artist: None,
        album: None,
        year: None,
    };
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

// ─────────────────────────────────────────────
// FOLDER PARSING
// ─────────────────────────────────────────────

fn parse_folder_path(path: &Path) -> FilenameMetadata {
    let components: Vec<&str> = path
        .parent()
        .map(|p| {
            p.components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect()
        })
        .unwrap_or_default();

    if components.len() < 2 {
        return FilenameMetadata {
            title: None,
            artist: None,
            album: None,
            year: None,
        };
    }

    let folder = components[components.len() - 1];
    let artist = components[components.len() - 2].to_string();

    if components.len() >= 3 {
        let maybe_year = components[components.len() - 3];
        if maybe_year.len() == 4 && maybe_year.parse::<u32>().is_ok() {
            return FilenameMetadata {
                title: None,
                artist: Some(artist),
                album: Some(folder.to_string()),
                year: Some(maybe_year.to_string()),
            };
        }
    }

    if let Some(captures) = extract_year_from_folder(folder) {
        return FilenameMetadata {
            title: None,
            artist: Some(artist),
            album: Some(captures.0),
            year: Some(captures.1),
        };
    }

    FilenameMetadata {
        title: None,
        artist: Some(artist),
        album: Some(folder.to_string()),
        year: None,
    }
}

fn extract_year_from_folder(folder: &str) -> Option<(String, String)> {
    if let Some(pos) = folder.find(" - ") {
        let maybe_year = &folder[..pos];
        if maybe_year.len() == 4 && maybe_year.parse::<u32>().is_ok() {
            return Some((folder[pos + 3..].trim().to_string(), maybe_year.to_string()));
        }
    }

    if folder.ends_with(')') {
        if let Some(open) = folder.rfind('(') {
            let maybe_year = &folder[open + 1..folder.len() - 1];
            if maybe_year.len() == 4 && maybe_year.parse::<u32>().is_ok() {
                return Some((folder[..open].trim().to_string(), maybe_year.to_string()));
            }
        }
    }

    None
}

// ─────────────────────────────────────────────
// TRACKS
// ─────────────────────────────────────────────

pub fn read_track(path: &Path) -> Option<Track> {
    read_track_with_settings(
        path,
        "tag",
        "tag",
        "tag",
        "tag",
        "",
        false,
        false,
        false,
        ARTIST_TAG_DELIMITERS,
        GENRE_DELIMITERS,
    )
}

pub fn read_track_with_settings(
    path: &Path,
    priority_title: &str,
    priority_artist: &str,
    priority_album: &str,
    priority_year: &str,
    custom_pattern: &str,
    folder_fallback_artist: bool,
    folder_fallback_album: bool,
    folder_fallback_year: bool,
    artist_tag_delimiters: &[&str],
    genre_delimiters: &[&str],
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

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let filename_meta = parse_filename(path, custom_pattern);

    let (
        tag_title,
        tag_artist,
        tag_album,
        tag_album_artist,
        tag_genres,
        tag_year,
        tag_rating,
        tag_track_number,
        bpm,
        key,
        artwork_blob,
    ) = if let Some(tag) = tag {
        let artwork_blob = tag.pictures().first().map(|p| p.data().to_vec());
        let rating = tag
            .get_string(&lofty::tag::ItemKey::Popularimeter)
            .and_then(|s| normalize_rating(s));
        let track_number = tag
            .get_string(&lofty::tag::ItemKey::TrackNumber)
            .and_then(|s| {
                s.split('/')
                    .next()
                    .and_then(|n| n.trim().parse::<u32>().ok())
            });
        (
            tag.title().map(|s| s.to_string()),
            tag.artist().map(|s| s.to_string()),
            tag.album().map(|s| s.to_string()),
            tag.get_string(&lofty::tag::ItemKey::AlbumArtist)
                .map(|s| s.to_string()),
            tag.genre().map(|s| s.to_string()),
            tag.year().map(|y| y.to_string()),
            rating,
            track_number,
            tag.get_string(&lofty::tag::ItemKey::Bpm)
                .and_then(|s| s.parse::<f32>().ok()),
            tag.get_string(&lofty::tag::ItemKey::InitialKey)
                .map(|s| s.to_string()),
            artwork_blob,
        )
    } else {
        (
            None, None, None, None, None, None, None, None, None, None, None,
        )
    };

    let folder_meta = parse_folder_path(path);

    let title = resolve_field(tag_title, filename_meta.title, priority_title);

    let artist_str = resolve_field(tag_artist, filename_meta.artist.clone(), priority_artist)
        .or_else(|| {
            if folder_fallback_artist {
                folder_meta.artist.clone()
            } else {
                None
            }
        });

    let artists = artist_str.as_deref().map(|s| {
        let parts = split_on_delimiters(s, artist_tag_delimiters);
        serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string())
    });

    let album_str = resolve_field(tag_album, filename_meta.album, priority_album).or_else(|| {
        if folder_fallback_album {
            folder_meta.album.clone()
        } else {
            None
        }
    });

    let album_artist = tag_album_artist.or_else(|| {
        artists
            .as_deref()
            .and_then(|a| serde_json::from_str::<Vec<String>>(a).ok())
            .and_then(|v| v.into_iter().next())
    });

    let albums = album_str.map(|a| {
        serde_json::to_string(&vec![serde_json::json!({
            "name": a,
            "track_number": tag_track_number
        })])
        .unwrap_or_else(|_| "[]".to_string())
    });

    let genres = tag_genres.map(|g| {
        let parts = split_on_delimiters(&g, genre_delimiters);
        serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string())
    });

    let year = resolve_field(tag_year, filename_meta.year, priority_year).or_else(|| {
        if folder_fallback_year {
            folder_meta.year.clone()
        } else {
            None
        }
    });

    Some(Track {
        id: None,
        uid: Uuid::new_v4().to_string(),
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
        credits: None,
        label: None,
        artwork_blob,
        artwork_path: None,
    })
}

// ─────────────────────────────────────────────
// ALBUMS
// ─────────────────────────────────────────────

/// Finds an existing album by title + album_artist (case-insensitive), or creates one.
/// Returns the album's integer id and uid.
fn find_or_create_album(
    conn: &Connection,
    title: &str,
    album_artist: Option<&str>,
    year: Option<&str>,
    genres: Option<&str>,
    artwork_blob: Option<Vec<u8>>,
    track_id: i64,
    track_title: Option<&str>,
) -> Option<(i64, String)> {
    let existing = get_all_albums(conn).ok()?;

    let title_lower = title.to_lowercase();
    let artist_lower = album_artist.map(|a| a.to_lowercase());

    if let Some(album) = existing.iter().find(|a| {
        a.title.to_lowercase() == title_lower
            && a.album_artist.as_deref().map(|s| s.to_lowercase()) == artist_lower
    }) {
        let album_id = album.id?;
        let uid = album.uid.clone();

        let existing_tracks: Vec<serde_json::Value> = album
            .tracks
            .as_deref()
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or_default();

        if !existing_tracks
            .iter()
            .any(|t| t["id"].as_i64() == Some(track_id))
        {
            let mut updated = existing_tracks;
            updated.push(serde_json::json!({
                "id": track_id,
                "name": track_title.unwrap_or("")
            }));
            let tracks_json = serde_json::to_string(&updated).ok()?;

            update_album(
                conn,
                album_id,
                &AlbumUpdate {
                    format: None,
                    title: None,
                    rating: None,
                    artists: None,
                    album_artist: None,
                    release_date: None,
                    tags: None,
                    genres: None,
                    tracks: Some(tracks_json),
                    credits: None,
                    label: None,
                    artwork_blob: None,
                    artwork_path: None,
                },
            )
            .ok()?;
        }

        return Some((album_id, uid));
    }

    let uid = Uuid::new_v4().to_string();
    let tracks_json = serde_json::to_string(&vec![serde_json::json!({
        "id": track_id,
        "name": track_title.unwrap_or("")
    })])
    .unwrap_or_else(|_| "[]".to_string());

    let album = Album {
        id: None,
        uid: uid.clone(),
        format: None,
        title: title.to_string(),
        rating: None,
        artists: album_artist
            .map(|a| serde_json::to_string(&vec![a]).unwrap_or_else(|_| "[]".to_string())),
        album_artist: album_artist.map(|s| s.to_string()),
        release_date: year.map(|s| s.to_string()),
        tags: None,
        genres: genres.map(|s| s.to_string()),
        tracks: Some(tracks_json),
        credits: None,
        label: None,
        artwork_blob,
        artwork_path: None,
    };

    create_album(conn, &album).ok()?;

    let created = get_all_albums(conn).ok()?;
    let created_album = created.iter().find(|a| a.uid == uid)?;

    Some((created_album.id?, uid))
}

// ─────────────────────────────────────────────
// ARTISTS
// ─────────────────────────────────────────────

/// Finds an existing artist by name (case-insensitive), or creates one.
/// Returns the artist's integer id and uid.
fn find_or_create_artist(conn: &Connection, name: &str) -> Option<(i64, String)> {
    let existing = get_all_artists(conn).ok()?;
    let name_lower = name.to_lowercase();

    if let Some(artist) = existing
        .iter()
        .find(|a| a.name.to_lowercase() == name_lower)
    {
        return Some((artist.id?, artist.uid.clone()));
    }

    let uid = Uuid::new_v4().to_string();
    let artist = Artist {
        id: None,
        uid: uid.clone(),
        name: name.to_string(),
        aka: None,
        about: None,
        tags: None,
        genres: None,
        websites: None,
        members: None,
        profile_art_blob: None,
        profile_art_path: None,
        banner_art_blob: None,
        banner_art_path: None,
    };

    create_artist(conn, &artist).ok()?;

    let created = get_all_artists(conn).ok()?;
    let created_artist = created.iter().find(|a| a.uid == uid)?;

    Some((created_artist.id?, uid))
}

// ─────────────────────────────────────────────
// SCAN
// ─────────────────────────────────────────────

pub fn scan_directory_with_progress(conn: &Connection, dir: &str, app: &AppHandle) {
    let priority_title = get_setting(conn, "filename_priority_title")
        .ok()
        .flatten()
        .unwrap_or_else(|| "tag".to_string());
    let priority_artist = get_setting(conn, "filename_priority_artist")
        .ok()
        .flatten()
        .unwrap_or_else(|| "tag".to_string());
    let priority_album = get_setting(conn, "filename_priority_album")
        .ok()
        .flatten()
        .unwrap_or_else(|| "tag".to_string());
    let priority_year = get_setting(conn, "filename_priority_year")
        .ok()
        .flatten()
        .unwrap_or_else(|| "tag".to_string());
    let custom_pattern = get_setting(conn, "filename_custom_pattern")
        .ok()
        .flatten()
        .unwrap_or_default();
    let folder_fallback_artist = get_setting(conn, "folder_fallback_artist")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);
    let folder_fallback_album = get_setting(conn, "folder_fallback_album")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);
    let folder_fallback_year = get_setting(conn, "folder_fallback_year")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);

    // Load user-overridden delimiter lists, falling back to the compiled-in constants.
    // Each setting stores delimiters as a pipe-separated string e.g. " / | ; ".
    // Individual delimiters are the substrings between | characters.
    let artist_tag_delimiters_owned: Vec<String> = get_setting(conn, "artist_tag_delimiters")
        .ok()
        .flatten()
        .map(|v| {
            v.split('|')
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_else(|| {
            ARTIST_TAG_DELIMITERS
                .iter()
                .map(|s| s.to_string())
                .collect()
        });
    let artist_tag_delimiters: Vec<&str> = artist_tag_delimiters_owned
        .iter()
        .map(|s| s.as_str())
        .collect();

    let genre_delimiters_owned: Vec<String> = get_setting(conn, "genre_delimiters")
        .ok()
        .flatten()
        .map(|v| {
            v.split('|')
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_else(|| GENRE_DELIMITERS.iter().map(|s| s.to_string()).collect());
    let genre_delimiters: Vec<&str> = genre_delimiters_owned.iter().map(|s| s.as_str()).collect();

    let create_artists = get_setting(conn, "scan_create_artists")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(true);
    let create_albums = get_setting(conn, "scan_create_albums")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(true);

    let all_files: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_supported(e.path()))
        .collect();

    let total = all_files.len();
    let mut scanned = 0;

    app.emit(
        "scan:progress",
        serde_json::json!({ "scanned": 0, "total": total }),
    )
    .ok();

    for entry in all_files {
        if let Some(track) = read_track_with_settings(
            entry.path(),
            &priority_title,
            &priority_artist,
            &priority_album,
            &priority_year,
            &custom_pattern,
            folder_fallback_artist,
            folder_fallback_album,
            folder_fallback_year,
            &artist_tag_delimiters,
            &genre_delimiters,
        ) {
            if upsert_track(conn, &track).is_ok() {
                let track_id = conn.last_insert_rowid();
                let track_title = track.title.as_deref();

                if create_albums {
                    if let Some(albums_json) = &track.albums {
                        if let Ok(albums_arr) =
                            serde_json::from_str::<Vec<serde_json::Value>>(albums_json)
                        {
                            for album_entry in albums_arr {
                                if let Some(album_name) =
                                    album_entry["name"].as_str().filter(|s| !s.is_empty())
                                {
                                    find_or_create_album(
                                        conn,
                                        album_name,
                                        track.album_artist.as_deref(),
                                        track.year.as_deref(),
                                        track.genres.as_deref(),
                                        track.artwork_blob.clone(),
                                        track_id,
                                        track_title,
                                    );
                                }
                            }
                        }
                    }
                }

                if create_artists {
                    if let Some(artists_json) = &track.artists {
                        if let Ok(artist_names) = serde_json::from_str::<Vec<String>>(artists_json)
                        {
                            for name in artist_names {
                                if !name.is_empty() {
                                    find_or_create_artist(conn, &name);
                                }
                            }
                        }
                    }
                    if let Some(album_artist) = &track.album_artist {
                        if !album_artist.is_empty() {
                            find_or_create_artist(conn, album_artist);
                        }
                    }
                }
            }
        }

        scanned += 1;
        if scanned % 25 == 0 || scanned == total {
            app.emit(
                "scan:progress",
                serde_json::json!({ "scanned": scanned, "total": total }),
            )
            .ok();
        }
    }

    let duplicates = crate::db::find_duplicates(conn).unwrap_or_default();
    if !duplicates.is_empty() {
        app.emit("duplicates:found", duplicates.len()).ok();
    }

    app.emit("scan:done", ()).ok();
}
