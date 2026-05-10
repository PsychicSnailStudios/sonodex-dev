use crate::db::{
	create_album, create_artist, generate_uid, get_all_albums, get_all_artists, get_setting,
	update_album, update_track_metadata_by_uid, upsert_track, Album, AlbumUpdate,
	Artist, MetadataUpdate, Track, 
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

const ARTIST_TAG_DELIMITERS: &[&str] = &[" / ", "; ", ", ", ","];
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

fn try_parse_ampersand(artists: Vec<String>) -> Vec<String> {
	let lower_existing: Vec<String> = artists.iter().map(|a| a.to_lowercase()).collect();
	let mut result: Vec<String> = Vec::new();

	for artist in &artists {
		if artist.contains('&') {
			let parts: Vec<String> = artist
				.split('&')
				.map(|p| p.trim().to_string())
				.filter(|p| !p.is_empty())
				.collect();

			let new_parts: Vec<&String> = parts
				.iter()
				.filter(|p| !lower_existing.contains(&p.to_lowercase()))
				.collect();

			if new_parts.is_empty() || new_parts.len() <= 2 {
				for part in &parts {
					let lower = part.to_lowercase();
					if !result.iter().any(|r: &String| r.to_lowercase() == lower) {
						result.push(part.clone());
					}
				}
				continue;
			}
		}

		let lower = artist.to_lowercase();
		if !result.iter().any(|r: &String| r.to_lowercase() == lower) {
			result.push(artist.clone());
		}
	}

	result
}

fn extract_feat_artists(title: &str) -> Vec<String> {
	let lower = title.to_lowercase();
	let patterns = ["feat.", "ft.", "featuring"];

	for pat in &patterns {
		if let Some(pos) = lower.find(pat) {
			let after_pat = &title[pos + pat.len()..];
			let raw = after_pat
				.trim_start_matches(|c: char| c == '.' || c == ' ')
				.trim_end_matches(')')
				.trim_end_matches(']')
				.trim();
			let parts: Vec<String> = raw
				.split(&[',', '&', '/'][..])
				.map(|p| {
					p.trim()
						.trim_matches(|c| c == '(' || c == '[' || c == ')' || c == ']')
						.trim()
						.to_string()
				})
				.filter(|p| !p.is_empty())
				.collect();
			if !parts.is_empty() {
				return parts;
			}
		}
	}
	Vec::new()
}

fn is_verified_folder_structure(path: &Path) -> bool {
	let components: Vec<&str> = path
		.parent()
		.map(|p| {
			p.components()
				.filter_map(|c| c.as_os_str().to_str())
				.collect()
		})
		.unwrap_or_default();

	if components.len() < 2 {
		return false;
	}

	let folder = components[components.len() - 1];
	let artist = components[components.len() - 2];

	let generic = [
		"music",
		"audio",
		"downloads",
		"files",
		"media",
		"tracks",
		"songs",
	];
	if generic.contains(&folder.to_lowercase().as_str())
		|| generic.contains(&artist.to_lowercase().as_str())
	{
		return false;
	}

	folder.len() >= 2 && artist.len() >= 2
}

#[derive(Debug)]
struct FilenameMetadata {
	title: Option<String>,
	artist: Option<String>,
	album: Option<String>,
	year: Option<String>,
	disc: Option<u32>,
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
		disc: None,
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
		disc: None,
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
		disc: None,
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

fn is_disc_folder(name: &str) -> bool {
	let lower = name.to_lowercase();
	let prefixes = ["disc", "disk", "cd", "side", "volume", "vol", "digital media"];
	for prefix in &prefixes {
		if lower.starts_with(prefix) {
			let rest = lower[prefix.len()..].trim_start_matches(|c: char| c == '.' || c == ' ' || c == '-');
			if rest.is_empty() {
				return true;
			}
			let num_str: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
			if !num_str.is_empty() {
				return true;
			}
		}
	}
	false
}

fn parse_disc_from_folder(path: &Path) -> Option<u32> {
	let folder = path
		.parent()
		.and_then(|p| p.file_name())
		.and_then(|n| n.to_str())?;

	let lower = folder.to_lowercase();

	let prefixes = ["disc", "disk", "cd", "side", "volume", "vol", "digital media"];
	for prefix in &prefixes {
		if lower.starts_with(prefix) {
			let rest = lower[prefix.len()..].trim_start_matches(|c: char| c == '.' || c == ' ' || c == '-');
			if rest.is_empty() {
				return Some(1);
			}
			let num_str: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
			if num_str.is_empty() {
				continue;
			}
			if let Ok(n) = num_str.parse::<u32>() {
				return Some(n);
			}
			let letter_disc = match num_str.as_str() {
				"a" => Some(1u32),
				"b" => Some(2),
				"c" => Some(3),
				"d" => Some(4),
				"e" => Some(5),
				"f" => Some(6),
				_ => None,
			};
			if letter_disc.is_some() {
				return letter_disc;
			}
		}
	}
	None
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
			disc: None,
		};
	}

	let immediate = components[components.len() - 1];

	let (folder, artist_idx) = if is_disc_folder(immediate) && components.len() >= 3 {
		(components[components.len() - 2], components.len() - 3)
	} else {
		(immediate, components.len() - 2)
	};

	if artist_idx >= components.len() {
		return FilenameMetadata {
			title: None,
			artist: None,
			album: None,
			year: None,
			disc: None,
		};
	}

	let artist = components[artist_idx].to_string();

	if artist_idx >= 1 {
		let maybe_year = components[artist_idx - 1];
		if maybe_year.len() == 4 && maybe_year.parse::<u32>().is_ok() {
			return FilenameMetadata {
				title: None,
				artist: Some(artist),
				album: Some(folder.to_string()),
				year: Some(maybe_year.to_string()),
				disc: None,
			};
		}
	}

	if let Some(captures) = extract_year_from_folder(folder) {
		return FilenameMetadata {
			title: None,
			artist: Some(artist),
			album: Some(captures.0),
			year: Some(captures.1),
			disc: None,
		};
	}

	FilenameMetadata {
		title: None,
		artist: Some(artist),
		album: Some(folder.to_string()),
		year: None,
		disc: None,
	}
}

fn artists_to_names(artists: &[crate::db::ArtistEntry]) -> Vec<String> {
	artists.iter().map(|a| a.name.clone()).collect()
}

fn names_to_artists(names: &[String]) -> Vec<crate::db::ArtistEntry> {
	names.iter().map(|n| crate::db::ArtistEntry { uid: String::new(), name: n.clone() }).collect()
}

pub fn find_remote_local_counterpart(conn: &Connection, incoming: &Track) -> Option<String> {
	let incoming_title = incoming.title.as_deref()?.to_lowercase();

	// Extract artist name from the new {uid, name} format
	let incoming_artist_owned: Option<String> = incoming
		.album_artist
		.as_ref()
		.map(|a| a.name.to_lowercase())
		.or_else(|| {
			incoming.artists.as_ref().and_then(|v| v.first()).map(|a| a.name.to_lowercase())
		});

	let incoming_duration = incoming.duration_ms.unwrap_or(0);
	let incoming_is_remote = incoming
		.remote_path
		.as_deref()
		.map(|rp| !rp.is_empty())
		.unwrap_or(false)
		|| incoming.path.is_empty();

	let mut stmt = conn
		.prepare(
			"SELECT uid, path, remote_path, duration_ms, album_artist, artists
				FROM tracks
				WHERE LOWER(TRIM(title)) = LOWER(TRIM(?1))",
		)
		.ok()?;

	let rows: Vec<(String, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>)> = {
		stmt.query_map(rusqlite::params![incoming_title], |row| {
			Ok((
				row.get::<_, String>(0)?,
				row.get::<_, Option<String>>(1)?,
				row.get::<_, Option<String>>(2)?,
				row.get::<_, Option<i64>>(3)?,
				row.get::<_, Option<String>>(4)?,
				row.get::<_, Option<String>>(5)?,
			))
		})
		.ok()?
		.filter_map(|r| r.ok())
		.collect()
	};

	for (uid, path, remote_path, duration_ms, album_artist_json, artists_json) in rows {
		let existing_is_remote = remote_path.as_deref().map(|rp| !rp.is_empty()).unwrap_or(false)
			|| path.as_deref().map(|p| p.is_empty()).unwrap_or(true);

		if existing_is_remote == incoming_is_remote {
			continue;
		}

		let dur = duration_ms.unwrap_or(0);
		if (dur - incoming_duration).abs() > 1000 {
			continue;
		}

		let existing_artist: Option<String> = album_artist_json
			.as_deref()
			.and_then(|s| serde_json::from_str::<crate::db::ArtistEntry>(s).ok())
			.map(|a| a.name.to_lowercase())
			.or_else(|| {
				artists_json
					.as_deref()
					.and_then(|s| serde_json::from_str::<Vec<crate::db::ArtistEntry>>(s).ok())
					.and_then(|v| v.into_iter().next())
					.map(|a| a.name.to_lowercase())
			});

		if existing_artist == incoming_artist_owned {
			return Some(uid);
		}
	}

	None
}

pub fn merge_paths_into_existing(conn: &Connection, existing_uid: &str, incoming: &Track) {
	let row: Option<(Option<String>, Option<String>)> = conn
		.query_row(
			"SELECT path, remote_path FROM tracks WHERE uid = ?1",
			rusqlite::params![existing_uid],
			|row| Ok((row.get(0)?, row.get(1)?)),
		)
		.ok();

	let (existing_path, existing_remote) = match row {
		Some(r) => r,
		None => return,
	};

	let need_path = existing_path.as_deref().map(|p| p.is_empty()).unwrap_or(true);
	let need_remote = existing_remote.as_deref().map(|rp| rp.is_empty()).unwrap_or(true);

	if need_path {
		if !incoming.path.is_empty() {
			conn.execute(
				"UPDATE tracks SET path = ?1 WHERE uid = ?2",
				rusqlite::params![incoming.path, existing_uid],
			)
			.ok();
		}
	}

	if need_remote {
		if let Some(ref rp) = incoming.remote_path {
			if !rp.is_empty() {
				conn.execute(
					"UPDATE tracks SET remote_path = ?1 WHERE uid = ?2",
					rusqlite::params![rp, existing_uid],
				)
				.ok();
			}
		}
	}

	let _ = conn.execute(
		"UPDATE tracks SET
			title        = COALESCE(title,        ?2),
			artists      = COALESCE(artists,      ?3),
			album_artist = COALESCE(album_artist, ?4),
			albums       = COALESCE(albums,       ?5),
			genres       = COALESCE(genres,       ?6),
			year         = COALESCE(year,         ?7),
			duration_ms  = COALESCE(duration_ms,  ?8),
			bpm          = COALESCE(bpm,          ?9),
			key          = COALESCE(key,          ?10),
			label        = COALESCE(label,        ?11),
			artwork_blob = COALESCE(artwork_blob, ?12),
			artwork_path = COALESCE(artwork_path, ?13),
			format       = COALESCE(format,       ?14),
			bitrate      = COALESCE(bitrate,      ?15)
		WHERE uid = ?1",
		rusqlite::params![
			existing_uid,
			incoming.title,
			crate::db::track_manager::OptJson(incoming.artists.as_ref()),
			crate::db::track_manager::OptJson(incoming.album_artist.as_ref()),
			crate::db::track_manager::OptJson(incoming.albums.as_ref()),
			crate::db::track_manager::OptJson(incoming.genres.as_ref()),
			incoming.year,
			incoming.duration_ms,
			incoming.bpm,
			incoming.key,
			incoming.label,
			incoming.artwork_blob,
			incoming.artwork_path,
			incoming.format,
			incoming.bitrate,
		],
	);
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
		false,
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
	try_ampersand: bool,
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
	let bitrate = properties.overall_bitrate().map(|b| b as i64);

	let tag = tagged_file
		.primary_tag()
		.or_else(|| tagged_file.first_tag());

	let filename_meta = parse_filename(path, custom_pattern);
	let folder_meta = parse_folder_path(path);
	let folder_verified = is_verified_folder_structure(path);

	let (
		tag_title,
		tag_artist,
		tag_album,
		tag_album_artist,
		tag_genres,
		tag_year,
		tag_rating,
		tag_track_number,
		tag_disc_number,
		bpm,
		key,
		artwork_blob,
		tag_label,
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
		let disc_number = tag
			.get_string(&lofty::tag::ItemKey::DiscNumber)
			.and_then(|s| {
				s.split('/')
					.next()
					.and_then(|n| n.trim().parse::<u32>().ok())
			});
		let label = tag
			.get_string(&lofty::tag::ItemKey::Publisher)
			.map(|s| s.to_string())
			.or_else(|| {
				tag.get_string(&lofty::tag::ItemKey::CopyrightMessage)
					.map(|s| s.to_string())
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
			disc_number,
			tag.get_string(&lofty::tag::ItemKey::Bpm)
				.and_then(|s| s.parse::<f32>().ok()),
			tag.get_string(&lofty::tag::ItemKey::InitialKey)
				.map(|s| s.to_string()),
			artwork_blob,
			label,
		)
	} else {
		(
			None, None, None, None, None, None, None, None, None, None, None, None, None,
		)
	};

	let tag_artist = tag_artist.filter(|s| !s.trim().is_empty());
	let tag_album = tag_album.filter(|s| !s.trim().is_empty());
	let tag_album_artist = tag_album_artist.filter(|s| !s.trim().is_empty());

	let title = if priority_title == "filename" {
		filename_meta
			.title
			.clone()
			.filter(|s| !s.trim().is_empty())
			.or_else(|| tag_title.clone().filter(|s| !s.trim().is_empty()))
	} else {
		tag_title
			.clone()
			.filter(|s| !s.trim().is_empty())
			.or_else(|| filename_meta.title.clone().filter(|s| !s.trim().is_empty()))
	};

	let tag_artists: Option<Vec<String>> = tag_artist
		.as_deref()
		.map(|s| split_on_delimiters(s, artist_tag_delimiters));

	let filename_artists: Option<Vec<String>> = filename_meta
		.artist
		.as_deref()
		.filter(|s| !s.trim().is_empty())
		.map(|s| split_on_delimiters(s, artist_filename_delimiters));

	let has_tag_artist = tag_artist.is_some();
	let has_filename_artist = filename_meta
		.artist
		.as_deref()
		.map(|s| !s.trim().is_empty())
		.unwrap_or(false);

	let folder_artists: Option<Vec<String>> =
		if folder_verified && !has_tag_artist && !has_filename_artist {
			folder_meta
				.artist
				.as_deref()
				.filter(|s| !s.trim().is_empty())
				.map(|s| split_on_delimiters(s, artist_tag_delimiters))
		} else {
			None
		};

	let mut resolved_artists: Option<Vec<String>> = match priority_artist {
		"filename" => filename_artists.or(tag_artists).or(folder_artists),
		"folder" => folder_artists.or(tag_artists).or(filename_artists),
		_ => tag_artists.or(filename_artists).or(folder_artists),
	};

	let title_str = title.as_deref().unwrap_or("");
	let filename_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
	let mut feat_artists = extract_feat_artists(title_str);
	for fa in extract_feat_artists(filename_stem) {
		if !feat_artists
			.iter()
			.any(|e: &String| e.to_lowercase() == fa.to_lowercase())
		{
			feat_artists.push(fa);
		}
	}

	if !feat_artists.is_empty() {
		let base = resolved_artists.unwrap_or_default();
		let mut merged = base;
		for fa in feat_artists {
			if !merged
				.iter()
				.any(|e: &String| e.to_lowercase() == fa.to_lowercase())
			{
				merged.push(fa);
			}
		}
		resolved_artists = Some(merged);
	}

	if try_ampersand {
		if let Some(artists) = resolved_artists {
			resolved_artists = Some(try_parse_ampersand(artists));
		}
	}

	// Resolve album_artist name (plain string at this stage)
	let album_artist_name: Option<String> = tag_album_artist
		.or_else(|| resolved_artists.as_ref().and_then(|v| v.first().cloned()));

	// Ensure album_artist appears first in the artists list
	let mut resolved_artists = resolved_artists;
	if let Some(ref aa) = album_artist_name {
		if let Some(ref mut list) = resolved_artists {
			let aa_lower = aa.to_lowercase();
			if let Some(pos) = list.iter().position(|a| a.to_lowercase() == aa_lower) {
				if pos != 0 {
					let item = list.remove(pos);
					list.insert(0, item);
				}
			} else if !list.is_empty() {
				list.insert(0, aa.clone());
			}
		}
	}

	// Build typed artist structs — UIDs patched in by process_track
	let artists = resolved_artists
		.as_ref()
		.map(|names| names_to_artists(names));

	// Build typed album_artist struct — UID patched in by process_track
	let album_artist = album_artist_name
		.as_deref()
		.map(|name| crate::db::ArtistEntry { uid: String::new(), name: name.to_string() });

	let tag_album_val = tag_album.filter(|s| !s.trim().is_empty());
	let filename_album_val = filename_meta.album.filter(|s| !s.trim().is_empty());

	let primary_album_name: Option<String> = match priority_album {
		"filename" => filename_album_val.or(tag_album_val),
		_ => tag_album_val.or(filename_album_val),
	};

	let folder_disc = if tag_disc_number.is_none() {
		parse_disc_from_folder(path)
	} else {
		None
	};
	let effective_disc = tag_disc_number.or(folder_disc);

	let mut album_entries: Vec<crate::db::TrackAlbumEntry> = Vec::new();

	if let Some(ref a) = primary_album_name {
		album_entries.push(crate::db::TrackAlbumEntry {
			uid: String::new(),
			name: a.clone(),
			track: tag_track_number.unwrap_or(0) as i32,
			disc: effective_disc.unwrap_or(0) as i32,
		});
	}

	if folder_verified {
		if let Some(ref fa) = folder_meta.album {
			let fa_trimmed = fa.trim();
			if !fa_trimmed.is_empty() {
				let already_present = album_entries.iter().any(|e| {
					e.name.to_lowercase() == fa_trimmed.to_lowercase()
				});
				if !already_present {
					album_entries.push(crate::db::TrackAlbumEntry {
						uid: String::new(),
						name: fa_trimmed.to_string(),
						track: tag_track_number.unwrap_or(0) as i32,
						disc: effective_disc.unwrap_or(0) as i32,
					});
				}
			}
		}
	}

	let albums = if album_entries.is_empty() {
		None
	} else {
		Some(album_entries)
	};

	let genres = tag_genres.map(|g| split_on_delimiters(&g, genre_delimiters));

	let tag_year_val = tag_year.filter(|s| !s.trim().is_empty());
	let filename_year_val = filename_meta.year.filter(|s| !s.trim().is_empty());
	let folder_year_val = if folder_verified {
		folder_meta.year.filter(|s| !s.trim().is_empty())
	} else {
		None
	};

	let year = match priority_year {
		"filename" => filename_year_val.or(tag_year_val).or(folder_year_val),
		"folder" => folder_year_val.or(tag_year_val).or(filename_year_val),
		_ => tag_year_val.or(filename_year_val).or(folder_year_val),
	};

	let format = path
		.extension()
		.and_then(|e| e.to_str())
		.map(|e| e.to_uppercase());

	let is_remote = path_str.starts_with("http://")
		|| path_str.starts_with("https://")
		|| path_str.starts_with("\\\\")
		|| path_str.starts_with("//");

	let local_path = if is_remote {
		String::new()
	} else {
		path_str.clone()
	};

	let remote_path = if is_remote {
		Some(path_str.clone())
	} else {
		Some(String::new())
	};

	Some(Track {
		id: None,
		uid: generate_uid("t"),
		path: local_path,
		remote_path,
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
		label: tag_label,
		artwork_blob: artwork_blob.clone(),
		artwork_path: None,
		artwork_thumb: artwork_blob.as_deref().and_then(crate::thumb::make_thumb),
		format: format.clone(),
		bitrate,
		remote_data: None,
		track_data: Some(crate::db::TrackData {
			format: format.clone(),
			bitrate,
		}),
	})
}

fn find_or_create_album(
	conn: &Connection,
	title: &str,
	album_artist_name: Option<&str>,
	year: Option<&str>,
	genres: Option<&str>,
	artwork_blob: Option<Vec<u8>>,
	track_uid: &str,
	track_title: Option<&str>,
	track_number: Option<u32>,
	label: Option<&str>,
) -> Option<(i64, String)> {
	let existing = get_all_albums(conn).ok()?;

	let title_lower = title.to_lowercase();
	let artist_lower = album_artist_name.map(|a| a.to_lowercase());

	if let Some(album) = existing.iter().find(|a| {
		a.title.to_lowercase() == title_lower
			&& a.album_artist
				.as_ref()
				.map(|ar| ar.name.to_lowercase())
				== artist_lower
	}) {
		let album_id = album.id?;
		let uid = album.uid.clone();

		let existing_tracks: Vec<serde_json::Value> = album
			.tracks
			.as_deref()
			.and_then(|t| serde_json::from_str(t).ok())
			.unwrap_or_default();

		let needs_label_update = album.label.is_none() && label.is_some();
		let track_already_present = existing_tracks
			.iter()
			.any(|t| t["uid"].as_str() == Some(track_uid));

		if !track_already_present || needs_label_update {
			let mut updated = existing_tracks;
			if !track_already_present {
				updated.push(serde_json::json!({
					"uid": track_uid,
					"name": track_title.unwrap_or(""),
					"track_number": track_number
				}));
			}
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
					label: if needs_label_update { label.map(|s| s.to_string()) } else { None },
					artwork_blob: None,
					artwork_path: None,
					emulate_type: None,
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

	let album_artist_struct = album_artist_name
		.map(|n| crate::db::ArtistEntry { uid: String::new(), name: n.to_string() });
	let artists_struct = album_artist_name
		.map(|n| vec![crate::db::ArtistEntry { uid: String::new(), name: n.to_string() }]);
	let genres_parsed: Option<Vec<String>> = genres
		.and_then(|g| serde_json::from_str(g).ok());

	let album = Album {
		id: None,
		uid: uid.clone(),
		format: None,
		title: title.to_string(),
		rating: None,
		artists: artists_struct,
		album_artist: album_artist_struct,
		release_date: year.map(|s| s.to_string()),
		tags: None,
		genres: genres_parsed,
		tracks: Some(tracks_json),
		credits: None,
		label: label.map(|s| s.to_string()),
		artwork_blob: artwork_blob.clone(),
		artwork_thumb: artwork_blob.as_deref().and_then(crate::thumb::make_thumb),
		artwork_path: None,
		emulate_type: None,
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
		profile_art_thumb: None,
		banner_art_thumb: None,
	};

	create_artist(conn, &artist).ok()?;

	let created = get_all_artists(conn).ok()?;
	let created_artist = created.iter().find(|a| a.uid == uid)?;

	Some((created_artist.id?, uid))
}

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

	// Collect artist name → uid mappings as we create/find them, so we can
	// patch UIDs back into the track and album records in one pass.
	let mut artist_uid_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

	if create_albums {
		if let Some(albums_arr) = &track.albums {
			let mut updated_album_entries: Vec<crate::db::TrackAlbumEntry> = Vec::new();

			let album_artist_name: Option<&str> = track.album_artist.as_ref().map(|a| a.name.as_str());
			let genres_str: Option<String> = track.genres.as_ref()
				.map(|v| serde_json::to_string(v).unwrap_or_default());

			for album_entry in albums_arr {
				if !album_entry.name.is_empty() {
					let track_number = if album_entry.track > 0 { Some(album_entry.track as u32) } else { None };

					if let Some((_album_id, album_uid)) = find_or_create_album(
						conn,
						&album_entry.name,
						album_artist_name,
						track.year.as_deref(),
						genres_str.as_deref(),
						track.artwork_blob.clone(),
						&track_uid,
						track_title,
						track_number,
						track.label.as_deref(),
					) {
						updated_album_entries.push(crate::db::TrackAlbumEntry {
							uid: album_uid,
							name: album_entry.name.clone(),
							track: album_entry.track,
							disc: album_entry.disc,
						});
					}
				}
			}

			if !updated_album_entries.is_empty() {
				let _ = update_track_metadata_by_uid(
					conn,
					&track_uid,
					&MetadataUpdate {
						albums: Some(updated_album_entries),
						..Default::default()
					},
				);
			}
		}
	}

	if create_artists {
		let mut all_names: Vec<String> = Vec::new();

		if let Some(ref artists) = track.artists {
			for a in artists {
				if !all_names.iter().any(|n: &String| n.to_lowercase() == a.name.to_lowercase()) {
					all_names.push(a.name.clone());
				}
			}
		}
		if let Some(ref aa) = track.album_artist {
			if !aa.name.is_empty() && !all_names.iter().any(|n: &String| n.to_lowercase() == aa.name.to_lowercase()) {
				all_names.push(aa.name.clone());
			}
		}

		for name in &all_names {
			if !name.is_empty() {
				if let Some((_, uid)) = find_or_create_artist(conn, name) {
					artist_uid_map.insert(name.to_lowercase(), uid);
				}
			}
		}

		// Patch artist UIDs back into the track's artists field
		if let Some(ref artists) = track.artists {
			let patched: Vec<crate::db::ArtistEntry> = artists.iter().map(|a| {
				let uid = artist_uid_map.get(&a.name.to_lowercase()).cloned().unwrap_or_default();
				crate::db::ArtistEntry { uid, name: a.name.clone() }
			}).collect();
			let _ = update_track_metadata_by_uid(
				conn,
				&track_uid,
				&MetadataUpdate {
					artists: Some(patched),
					..Default::default()
				},
			);
		}

		// Patch artist UID back into the track's album_artist field
		if let Some(ref aa) = track.album_artist {
			let uid = artist_uid_map.get(&aa.name.to_lowercase()).cloned().unwrap_or_default();
			let patched = crate::db::ArtistEntry { uid, name: aa.name.clone() };
			let _ = update_track_metadata_by_uid(
				conn,
				&track_uid,
				&MetadataUpdate {
					album_artist: Some(patched),
					..Default::default()
				},
			);
		}

		// Patch artist UIDs into any album records this track belongs to
		if let Some(ref album_entries) = track.albums {
			for album_entry in album_entries {
				if !album_entry.uid.is_empty() {
					let albums_all = get_all_albums(conn).unwrap_or_default();
					if let Some(album) = albums_all.iter().find(|a| a.uid == album_entry.uid) {
						let mut needs_update = false;
						let mut update = AlbumUpdate {
							format: None, title: None, rating: None,
							artists: None, album_artist: None, release_date: None,
							tags: None, genres: None, tracks: None,
							credits: None, label: None, artwork_blob: None,
							artwork_path: None, emulate_type: None,
						};

						if let Some(ref artists) = album.artists {
							let patched: Vec<crate::db::ArtistEntry> = artists.iter().map(|a| {
								let uid = artist_uid_map.get(&a.name.to_lowercase()).cloned().unwrap_or_default();
								crate::db::ArtistEntry { uid, name: a.name.clone() }
							}).collect();
							update.artists = serde_json::to_string(&patched).ok();
							needs_update = true;
						}

						if let Some(ref aa) = album.album_artist {
							if let Some(uid) = artist_uid_map.get(&aa.name.to_lowercase()) {
								let patched = crate::db::ArtistEntry { uid: uid.clone(), name: aa.name.clone() };
								update.album_artist = serde_json::to_string(&patched).ok();
								needs_update = true;
							}
						}

						if needs_update {
							if let Some(album_id) = album.id {
								let _ = update_album(conn, album_id, &update);
							}
						}
					}
				}
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
	let try_ampersand = get_setting(conn, "scan_try_parse_ampersand")
		.ok()
		.flatten()
		.map(|v| v == "true")
		.unwrap_or(true);
	let auto_enrich_tracks = get_setting(conn, "auto_enrich_tracks")
		.ok()
		.flatten()
		.map(|v| v == "true")
		.unwrap_or(false);
	let auto_enrich_albums = get_setting(conn, "auto_enrich_albums")
		.ok()
		.flatten()
		.map(|v| v == "true")
		.unwrap_or(false);
	let auto_fetch_lyrics = get_setting(conn, "auto_fetch_lyrics")
		.ok()
		.flatten()
		.map(|v| v == "true")
		.unwrap_or(false);

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
	let genre_delimiters: Vec<&str> = genre_delimiters_owned
		.iter()
		.map(|s| s.as_str())
		.collect();

	let enrich_settings: Option<crate::enrichment::EnrichSettings> =
		if auto_enrich_tracks || auto_enrich_albums {
			Some(crate::enrichment::EnrichSettings {
				primary_api: get_setting(conn, "enrich_primary_api")
					.ok()
					.flatten()
					.unwrap_or_else(|| "musicbrainz".to_string()),
				lastfm_key: get_setting(conn, "api_lastfm_key")
					.ok()
					.flatten()
					.unwrap_or_default(),
				discogs_key: get_setting(conn, "api_discogs_key")
					.ok()
					.flatten()
					.unwrap_or_default(),
				audiodb_key: get_setting(conn, "api_audiodb_key")
					.ok()
					.flatten()
					.unwrap_or_default(),
				priority_title: get_setting(conn, "enrich_priority_title")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_artists: get_setting(conn, "enrich_priority_artists")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_album_artist: get_setting(conn, "enrich_priority_album_artist")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_album: get_setting(conn, "enrich_priority_album")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_year: get_setting(conn, "enrich_priority_year")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_genres: get_setting(conn, "enrich_priority_genres")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_bpm: get_setting(conn, "enrich_priority_bpm")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_key: get_setting(conn, "enrich_priority_key")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
				priority_artwork: get_setting(conn, "enrich_priority_artwork")
					.ok()
					.flatten()
					.unwrap_or_else(|| "local".to_string()),
			})
		} else {
			None
		};

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

	let rt = tokio::runtime::Runtime::new().ok();
	let http_client = if auto_enrich_tracks || auto_enrich_albums || auto_fetch_lyrics {
		crate::enrichment::make_client().ok()
	} else {
		None
	};

	let mut scanned_album_uids: Vec<String> = Vec::new();

	for entry in all_files {
		if let Some(mut track) = read_track_with_settings(
			entry.path(),
			&priority_title,
			&priority_artist,
			&priority_album,
			&priority_year,
			&custom_pattern,
			&artist_tag_delimiters,
			&artist_filename_delimiters,
			&genre_delimiters,
			try_ampersand,
		) {
			if let Some(existing_uid) = find_remote_local_counterpart(conn, &track) {
				merge_paths_into_existing(conn, &existing_uid, &track);
				scanned += 1;
				if scanned % 25 == 0 || scanned == total {
					app.emit(
						"scan:progress",
						serde_json::json!({ "scanned": scanned, "total": total }),
					)
					.ok();
				}
				continue;
			}

			if auto_enrich_tracks {
				if let (Some(ref settings), Some(ref client), Some(ref rt)) =
					(&enrich_settings, &http_client, &rt)
				{
					let artists_plain = track.artists.as_ref().map(|v| {
						let names: Vec<String> = v.iter().map(|a| a.name.clone()).collect();
						serde_json::to_string(&names).unwrap_or_default()
					});
					let album_artist_plain = track.album_artist.as_ref().map(|a| a.name.clone());
					let albums_str = track.albums.as_ref()
						.and_then(|v| serde_json::to_string(v).ok());
					let genres_str = track.genres.as_ref()
						.and_then(|v| serde_json::to_string(v).ok());

					let track_input = crate::enrichment::TrackInput {
						id: 0,
						title: track.title.clone(),
						artists: artists_plain,
						album_artist: album_artist_plain,
						albums: albums_str,
						year: track.year.clone(),
						genres: genres_str,
						bpm: track.bpm,
						key: track.key.clone(),
						existing_artwork: track.artwork_blob.clone(),
					};

					if let Ok(result) = rt.block_on(crate::enrichment::enrich_track_async(
						client,
						&track_input,
						settings,
						None,
					)) {
						track.title = result.title.or(track.title);
						if let Some(enriched_artists) = result.artists {
							if let Ok(names) = serde_json::from_str::<Vec<String>>(&enriched_artists) {
								track.artists = Some(names_to_artists(&names));
							}
						}
						if let Some(enriched_aa) = result.album_artist {
							track.album_artist = Some(crate::db::ArtistEntry { uid: String::new(), name: enriched_aa });
						}
						if let Some(albums_str) = result.albums {
							track.albums = serde_json::from_str(&albums_str).ok().or(track.albums);
						}
						track.year = result.year.or(track.year);
						if let Some(genres_str) = result.genres {
							track.genres = serde_json::from_str(&genres_str).ok().or(track.genres);
						}
						track.bpm = result.bpm.or(track.bpm);
						track.key = result.key.or(track.key);
						track.artwork_blob = result.artwork.or(track.artwork_blob);
					}
				}
			}

			if upsert_track(conn, &track).is_ok() {
				let actual_uid = if let Some(ref rp) = track.remote_path {
					if !rp.is_empty() {
						conn.query_row(
							"SELECT uid FROM tracks WHERE remote_path = ?1",
							rusqlite::params![rp],
							|row| row.get::<_, String>(0),
						)
						.unwrap_or_else(|_| track.uid.clone())
					} else {
						track.uid.clone()
					}
				} else {
					track.uid.clone()
				};
				let track_for_process = crate::db::Track { uid: actual_uid, ..track.clone() };
				process_track(conn, &track_for_process);

				if let Some(ref tags) = track.tags {
					for name in tags {
						crate::db::tag_manager::ensure_tag(
							conn, name, crate::db::tag_manager::TagKind::Tag,
						);
					}
				}
				if let Some(ref genres) = track.genres {
					for name in genres {
						crate::db::tag_manager::ensure_tag(
							conn, name, crate::db::tag_manager::TagKind::Genre,
						);
					}
				}

				if auto_enrich_albums {
					if let Some(ref albums) = track.albums {
						for entry in albums {
							if !entry.uid.is_empty() && !scanned_album_uids.contains(&entry.uid) {
								scanned_album_uids.push(entry.uid.clone());
							}
						}
					}
				}

				if auto_fetch_lyrics {
					if let (Some(ref client), Some(ref rt)) = (&http_client, &rt) {
						let title = track.title.as_deref().unwrap_or("").to_string();
						let artist = track.album_artist.as_ref().map(|a| a.name.clone())
							.or_else(|| track.artists.as_ref().and_then(|v| v.first()).map(|a| a.name.clone()))
							.unwrap_or_default();
						let album = track.albums.as_ref()
							.and_then(|v| v.first())
							.map(|e| e.name.clone());
						let duration_secs = track.duration_ms.map(|ms| (ms / 1000) as u64);

						if !title.is_empty() && !artist.is_empty() {
							if let Some(lyrics) =
								rt.block_on(crate::enrichment::lyrics::fetch_lyrics(
									client,
									&title,
									&artist,
									album.as_deref(),
									duration_secs,
								))
							{
								let track_id = crate::db::get_track_by_uid(conn, &track.uid)
									.ok()
									.flatten()
									.and_then(|t| t.id);
								if let Some(id) = track_id {
									let _ = crate::db::upsert_lyrics(
										conn,
										&crate::db::Lyrics {
											id: None,
											track_id: id,
											source: lyrics.source,
											plain: lyrics.plain,
											synced: lyrics.synced,
											instrumental: lyrics.instrumental,
										},
									);
								}
							}
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

	if auto_enrich_albums && !scanned_album_uids.is_empty() {
		if let (Some(ref settings), Some(ref client), Some(ref rt)) =
			(&enrich_settings, &http_client, &rt)
		{
			let albums = crate::db::get_all_albums(conn).unwrap_or_default();
			for album_uid in &scanned_album_uids {
				if let Some(album) = albums.iter().find(|a| &a.uid == album_uid) {
					let artist = album.album_artist.as_ref().map(|a| a.name.clone())
						.or_else(|| album.artists.as_ref().and_then(|v| v.first()).map(|a| a.name.clone()));

					if let Some(artist) = artist {
						let result = rt.block_on(crate::enrichment::enrich_album_async(
							client,
							&album.title,
							&artist,
							settings,
							None,
						));
						let update = AlbumUpdate {
							title: None,
							format: result.format,
							rating: None,
							artists: None,
							album_artist: None,
							release_date: result.release_date,
							tags: None,
							genres: result.genres,
							tracks: None,
							credits: result.description,
							label: result.label,
							artwork_blob: result.artwork,
							artwork_path: None,
							emulate_type: None,
						};
						let _ = crate::db::update_album_by_uid(conn, album_uid, &update);
					}
				}
			}
		}
	}

	let duplicates = crate::db::find_duplicates(conn).unwrap_or_default();
	if !duplicates.is_empty() {
		app.emit("duplicates:found", duplicates.len()).ok();
	}

	app.emit("scan:done", ()).ok();
}