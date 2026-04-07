use crate::db::{
	create_album, create_artist, generate_uid, get_all_albums, get_all_artists, get_setting,
	update_album, update_artist, update_track_metadata_by_uid, upsert_track, Album, AlbumUpdate,
	Artist, ArtistUpdate, MetadataUpdate, Track,
};
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

const ARTIST_TAG_DELIMITERS: &[&str] = &[" / ", "; ", ", "];

const ARTIST_FILENAME_DELIMITERS: &[&str] = &[" / ", "; ", " feat. ", " ft. ", " featuring "];

const GENRE_DELIMITERS: &[&str] = &[" / ", "; ", ", "];

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
	folder_value: Option<String>,
	priority: &str,
) -> Option<String> {
	let tag_value = tag_value.filter(|s| !s.trim().is_empty());
	let filename_value = filename_value.filter(|s| !s.trim().is_empty());
	let folder_value = folder_value.filter(|s| !s.trim().is_empty());
	match priority {
		"filename" => filename_value.or(tag_value).or(folder_value),
		"folder" => folder_value.or(tag_value).or(filename_value),
		_ => tag_value.or(filename_value).or(folder_value),
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

pub fn read_track(path: &Path) -> Option<Track> {
	read_track_with_settings(
		path,
		"tag",
		"tag",
		"tag",
		"tag",
		"",
		ARTIST_TAG_DELIMITERS,
		ARTIST_FILENAME_DELIMITERS,
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
	artist_tag_delimiters: &[&str],
	artist_filename_delimiters: &[&str],
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
	let folder_meta = parse_folder_path(path);

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

	let tag_artist = tag_artist.filter(|s| !s.trim().is_empty());
	let tag_album = tag_album.filter(|s| !s.trim().is_empty());
	let tag_album_artist = tag_album_artist.filter(|s| !s.trim().is_empty());

	let title = resolve_field(tag_title, filename_meta.title, None, priority_title);

	let tag_artists: Option<Vec<String>> = tag_artist
		.map(|s| split_on_delimiters(&s, artist_tag_delimiters));

	let filename_artists: Option<Vec<String>> = filename_meta
		.artist
		.clone()
		.filter(|s| !s.trim().is_empty())
		.map(|s| split_on_delimiters(&s, artist_filename_delimiters));

	let folder_artists: Option<Vec<String>> = folder_meta
		.artist
		.clone()
		.filter(|s| !s.trim().is_empty())
		.map(|s| split_on_delimiters(&s, artist_tag_delimiters));

	let resolved_artists = match priority_artist {
		"filename" => filename_artists.or(tag_artists).or(folder_artists),
		"folder" => folder_artists.or(tag_artists).or(filename_artists),
		_ => tag_artists.or(filename_artists).or(folder_artists),
	};

	let artists = resolved_artists
		.as_ref()
		.map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()));

	let album_artist = tag_album_artist.or_else(|| {
		resolved_artists
			.as_ref()
			.and_then(|v| v.first().cloned())
	});

	let tag_album_name = resolve_field(
		tag_album,
		filename_meta.album,
		folder_meta.album.clone(),
		priority_album,
	);

	let folder_album_name = folder_meta.album.clone().filter(|s| !s.trim().is_empty());

	let mut album_entries: Vec<serde_json::Value> = Vec::new();

	if let Some(ref a) = tag_album_name {
		album_entries.push(serde_json::json!({
			"uid": "",
			"name": a,
			"track_number": tag_track_number
		}));
	}

	if let Some(ref fa) = folder_album_name {
		let already_present = album_entries
			.iter()
			.any(|e| e["name"].as_str().map(|n| n.to_lowercase()) == Some(fa.to_lowercase()));
		if !already_present {
			album_entries.push(serde_json::json!({
				"uid": "",
				"name": fa,
				"track_number": tag_track_number
			}));
		}
	}

	let albums = if album_entries.is_empty() {
		None
	} else {
		serde_json::to_string(&album_entries).ok()
	};

	let genres = tag_genres.map(|g| {
		let parts = split_on_delimiters(&g, genre_delimiters);
		serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string())
	});

	let year = resolve_field(
		tag_year,
		filename_meta.year,
		folder_meta.year.clone(),
		priority_year,
	);

	Some(Track {
		id: None,
		uid: generate_uid("t"),
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
		user_options: None,
		credits: None,
		label: None,
		artwork_blob,
		artwork_path: None,
	})
}

fn find_or_create_album(
	conn: &Connection,
	title: &str,
	album_artist: Option<&str>,
	year: Option<&str>,
	genres: Option<&str>,
	artwork_blob: Option<Vec<u8>>,
	track_uid: &str,
	track_title: Option<&str>,
	track_number: Option<u32>,
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
			.any(|t| t["uid"].as_str() == Some(track_uid))
		{
			let mut updated = existing_tracks;
			updated.push(serde_json::json!({
				"uid": track_uid,
				"name": track_title.unwrap_or(""),
				"track_number": track_number
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

	let uid = generate_uid("a");
	let tracks_json = serde_json::to_string(&vec![serde_json::json!({
		"uid": track_uid,
		"name": track_title.unwrap_or(""),
		"track_number": track_number
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

fn find_or_create_artist(conn: &Connection, name: &str) -> Option<(i64, String)> {
	let existing = get_all_artists(conn).ok()?;
	let name_lower = name.to_lowercase();

	if let Some(artist) = existing
		.iter()
		.find(|a| a.name.to_lowercase() == name_lower)
	{
		return Some((artist.id?, artist.uid.clone()));
	}

	let uid = generate_uid("ar");
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

/// Shared post-upsert logic: creates/updates albums and artists for a track.
/// Used by both the full scanner and the file watcher.
pub fn process_track(conn: &Connection, track: &Track) {
	let track_uid = track.uid.clone();
	let track_title = track.title.as_deref();

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

	if create_albums {
		if let Some(albums_json) = &track.albums {
			if let Ok(albums_arr) = serde_json::from_str::<Vec<serde_json::Value>>(albums_json) {
				let mut updated_album_entries: Vec<serde_json::Value> = Vec::new();

				for album_entry in &albums_arr {
					if let Some(album_name) =
						album_entry["name"].as_str().filter(|s| !s.is_empty())
					{
						let track_number =
							album_entry["track_number"].as_u64().map(|n| n as u32);

						if let Some((_album_id, album_uid)) = find_or_create_album(
							conn,
							album_name,
							track.album_artist.as_deref(),
							track.year.as_deref(),
							track.genres.as_deref(),
							track.artwork_blob.clone(),
							&track_uid,
							track_title,
							track_number,
						) {
							updated_album_entries.push(serde_json::json!({
								"uid": album_uid,
								"name": album_name,
								"track_number": track_number
							}));
						}
					}
				}

				if !updated_album_entries.is_empty() {
					if let Ok(patched_albums) = serde_json::to_string(&updated_album_entries) {
						let _ = update_track_metadata_by_uid(
							conn,
							&track_uid,
							&MetadataUpdate {
								title: None,
								artists: None,
								album_artist: None,
								albums: Some(patched_albums),
								year: None,
								genres: None,
								bpm: None,
								rating: None,
								tags: None,
								key: None,
								user_options: None,
								credits: None,
								label: None,
								artwork_blob: None,
								artwork_path: None,
							},
						);
					}
				}
			}
		}
	}

	if create_artists {
		if let Some(artists_json) = &track.artists {
			if let Ok(artist_names) = serde_json::from_str::<Vec<String>>(artists_json) {
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

	let artist_filename_delimiters_owned: Vec<String> =
		get_setting(conn, "artist_filename_delimiters")
			.ok()
			.flatten()
			.map(|v| {
				v.split('|')
					.map(|s| s.to_string())
					.filter(|s| !s.is_empty())
					.collect()
			})
			.unwrap_or_else(|| {
				ARTIST_FILENAME_DELIMITERS
					.iter()
					.map(|s| s.to_string())
					.collect()
			});
	let artist_filename_delimiters: Vec<&str> = artist_filename_delimiters_owned
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
			&artist_tag_delimiters,
			&artist_filename_delimiters,
			&genre_delimiters,
		) {
			if upsert_track(conn, &track).is_ok() {
				process_track(conn, &track);
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
