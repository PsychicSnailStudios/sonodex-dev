pub mod audiodb;
pub mod discogs;
pub mod lastfm;
pub mod musicbrainz;
pub mod lyrics;

use reqwest::Client;
use std::time::Duration;

pub const USER_AGENT: &str = "Sonodex/1.0 (music library app)";

// ─────────────────────────────────────────────
// SHARED TYPES
// ─────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct EnrichedMetadata {
	pub title: Option<String>,
	pub artists: Option<String>,
	pub album_artist: Option<String>,
	pub albums: Option<String>,
	pub year: Option<String>,
	pub genres: Option<String>,
	pub bpm: Option<f32>,
	pub key: Option<String>,
	pub artwork: Option<Vec<u8>>,
	pub label: Option<String>,
	pub format: Option<String>,
	pub mbid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TrackInput {
	pub id: i64,
	pub title: Option<String>,
	pub artists: Option<String>,
	pub album_artist: Option<String>,
	pub albums: Option<String>,
	pub year: Option<String>,
	pub genres: Option<String>,
	pub bpm: Option<f32>,
	pub key: Option<String>,
	pub existing_artwork: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct EnrichSettings {
	pub primary_api: String,
	pub lastfm_key: String,
	pub discogs_key: String,
	pub audiodb_key: String,
	pub priority_title: String,
	pub priority_artists: String,
	pub priority_album_artist: String,
	pub priority_album: String,
	pub priority_year: String,
	pub priority_genres: String,
	pub priority_bpm: String,
	pub priority_key: String,
	pub priority_artwork: String,
}

pub struct EnrichResult {
	pub title: Option<String>,
	pub artists: Option<String>,
	pub album_artist: Option<String>,
	pub albums: Option<String>,
	pub year: Option<String>,
	pub genres: Option<String>,
	pub bpm: Option<f32>,
	pub key: Option<String>,
	pub artwork: Option<Vec<u8>>,
}


// ─────────────────────────────────────────────
// CLIENT
// ─────────────────────────────────────────────

pub fn make_client() -> Result<Client, String> {
	Client::builder()
		.user_agent(USER_AGENT)
		.timeout(Duration::from_secs(10))
		.build()
		.map_err(|e| e.to_string())
}


// ─────────────────────────────────────────────
// PRIORITY MERGING
// ─────────────────────────────────────────────

pub fn apply_string(api: Option<String>, local: Option<String>, priority: &str) -> Option<String> {
	match priority {
		"api" => api.or(local),
		_ => {
			if local.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
				local
			} else {
				api
			}
		}
	}
}

pub fn apply_f32(api: Option<f32>, local: Option<f32>, priority: &str) -> Option<f32> {
	match priority {
		"api" => api.or(local),
		_ => local.or(api),
	}
}

pub fn apply_bytes(api: Option<Vec<u8>>, local: Option<Vec<u8>>, priority: &str) -> Option<Vec<u8>> {
	match priority {
		"api" => api.or(local),
		_ => {
			if local.as_ref().map(|v| !v.is_empty()).unwrap_or(false) {
				local
			} else {
				api
			}
		}
	}
}


// ─────────────────────────────────────────────
// ORCHESTRATION
// ─────────────────────────────────────────────

/// Runs all APIs in priority order, merging results as each one fills gaps.
/// The primary API is tried first; the rest follow in a fixed fallback order.
pub async fn enrich_track_async(
	client: &Client,
	track: &TrackInput,
	settings: &EnrichSettings,
) -> Result<EnrichResult, String> {
	let title = track.title.clone().unwrap_or_default();
	let artist = track
		.album_artist
		.clone()
		.or_else(|| {
			track.artists.as_deref().and_then(|a| {
				serde_json::from_str::<Vec<String>>(a)
					.ok()
					.and_then(|v| v.into_iter().next())
			})
		})
		.unwrap_or_default();

	if title.is_empty() || artist.is_empty() {
		return Err("Track missing title or artist".to_string());
	}

	// Build the ordered list of APIs to try, primary first.
	let all_apis = ["musicbrainz", "audiodb", "lastfm", "discogs"];
	let ordered: Vec<&str> = std::iter::once(settings.primary_api.as_str())
		.chain(all_apis.iter().copied().filter(|&a| a != settings.primary_api.as_str()))
		.collect();

	let mut merged = EnrichedMetadata::default();

	for api in ordered {
		// Stop early if we have everything we care about.
		if merged.title.is_some()
			&& merged.artists.is_some()
			&& merged.albums.is_some()
			&& merged.year.is_some()
			&& merged.genres.is_some()
			&& merged.artwork.is_some()
		{
			break;
		}

		let result = match api {
			"musicbrainz" => musicbrainz::search(client, &title, &artist).await,
			"audiodb" => audiodb::search(client, &title, &artist, &settings.audiodb_key).await,
			"lastfm" => lastfm::search(client, &title, &artist, &settings.lastfm_key).await,
			"discogs" => discogs::search(client, &title, &artist, &settings.discogs_key).await,
			_ => None,
		};

		if let Some(r) = result {
			// Fill gaps: only overwrite if the current merged field is empty.
			if merged.title.is_none() { merged.title = r.title; }
			if merged.artists.is_none() { merged.artists = r.artists; }
			if merged.album_artist.is_none() { merged.album_artist = r.album_artist; }
			if merged.albums.is_none() { merged.albums = r.albums; }
			if merged.year.is_none() { merged.year = r.year; }
			if merged.genres.is_none() { merged.genres = r.genres; }
			if merged.bpm.is_none() { merged.bpm = r.bpm; }
			if merged.key.is_none() { merged.key = r.key; }
			if merged.artwork.is_none() { merged.artwork = r.artwork; }
			if merged.label.is_none() { merged.label = r.label; }
			if merged.format.is_none() { merged.format = r.format; }
			if merged.mbid.is_none() { merged.mbid = r.mbid; }
		}
	}

	if merged.title.is_none() && merged.artists.is_none() {
		return Err("Track not found in any API".to_string());
	}

	Ok(EnrichResult {
		title: apply_string(merged.title, track.title.clone(), &settings.priority_title),
		artists: apply_string(merged.artists, track.artists.clone(), &settings.priority_artists),
		album_artist: apply_string(merged.album_artist, track.album_artist.clone(), &settings.priority_album_artist),
		albums: apply_string(merged.albums, track.albums.clone(), &settings.priority_album),
		year: apply_string(merged.year, track.year.clone(), &settings.priority_year),
		genres: apply_string(merged.genres, track.genres.clone(), &settings.priority_genres),
		bpm: apply_f32(merged.bpm, track.bpm, &settings.priority_bpm),
		key: apply_string(merged.key, track.key.clone(), &settings.priority_key),
		artwork: apply_bytes(merged.artwork, track.existing_artwork.clone(), &settings.priority_artwork),
	})
}
