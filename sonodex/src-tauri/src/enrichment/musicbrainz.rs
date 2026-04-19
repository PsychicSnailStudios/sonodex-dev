use super::EnrichedMetadata;
use reqwest::Client;
use serde::Deserialize;
use tokio::time::{sleep, Duration};

const MB_BASE: &str = "https://musicbrainz.org/ws/2";

// ─────────────────────────────────────────────
// RESPONSE TYPES
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct MbSearchResponse {
    recordings: Vec<MbRecording>,
}

#[derive(Debug, Deserialize)]
struct MbRecording {
    id: String,
    title: Option<String>,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<MbArtistCredit>>,
    releases: Option<Vec<MbRelease>>,
    tags: Option<Vec<MbTag>>,
}

#[derive(Debug, Deserialize)]
struct MbArtistCredit {
    artist: MbArtist,
}

#[derive(Debug, Deserialize)]
struct MbArtist {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MbRelease {
    title: String,
    date: Option<String>,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<MbArtistCredit>>,
}

#[derive(Debug, Deserialize)]
struct MbTag {
    name: String,
    count: u32,
}

// ─────────────────────────────────────────────
// SEARCH
// ─────────────────────────────────────────────

pub async fn search(client: &Client, title: &str, artist: &str) -> Option<EnrichedMetadata> {
    // MusicBrainz requires at least 1 second between requests.
    sleep(Duration::from_millis(1100)).await;

    let query = format!("recording:\"{}\" AND artist:\"{}\"", title, artist);
    let url = format!(
        "{}/recording?query={}&limit=1&inc=artist-credits+releases+tags&fmt=json",
        MB_BASE,
        urlencoding::encode(&query)
    );

    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let body = resp.text().await.ok()?;
    let data: MbSearchResponse = serde_json::from_str(&body).ok()?;
    let recording = data.recordings.into_iter().next()?;

    let mbid = recording.id.clone();
    let title = recording.title.clone();

    let artists: Vec<String> = recording
        .artist_credit
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|ac| ac.artist.name.clone())
        .collect();

    let release = recording.releases.as_deref().and_then(|r| r.first());
    let album_name = release.map(|r| r.title.clone());
    let year = release
        .and_then(|r| r.date.as_deref())
        .and_then(|d| d.split('-').next().map(|s| s.to_string()))
        .filter(|y| y.len() == 4);

    let album_artist = release
        .and_then(|r| r.artist_credit.as_deref())
        .and_then(|ac| ac.first())
        .map(|ac| ac.artist.name.clone())
        .or_else(|| artists.first().cloned());

    let mut top_tags: Vec<MbTag> = recording.tags.unwrap_or_default();
    top_tags.sort_by(|a, b| b.count.cmp(&a.count));
    let genres = if top_tags.is_empty() {
        None
    } else {
        let genre_vec: Vec<String> = top_tags.into_iter().take(3).map(|t| t.name).collect();
        serde_json::to_string(&genre_vec).ok()
    };

    let artists_json = if artists.is_empty() {
        None
    } else {
        serde_json::to_string(&artists).ok()
    };

    let albums_json = album_name.map(|name| {
        serde_json::to_string(&vec![
            serde_json::json!({ "name": name, "track_number": null }),
        ])
        .unwrap_or_else(|_| "[]".to_string())
    });

    Some(EnrichedMetadata {
        title,
        artists: artists_json,
        album_artist,
        albums: albums_json,
        year,
        genres,
        mbid: Some(mbid),
        ..Default::default()
    })
}

pub async fn fetch_cover_art(client: &Client, mbid: &str) -> Option<Vec<u8>> {
	let url = format!("https://coverartarchive.org/release-group/{}/front", mbid);
	let res = client.get(&url).send().await.ok()?;
	if res.status().is_success() {
		res.bytes().await.ok().map(|b| b.to_vec())
	} else {
		let url2 = format!("https://coverartarchive.org/release/{}/front", mbid);
		let res2 = client.get(&url2).send().await.ok()?;
		if res2.status().is_success() {
			res2.bytes().await.ok().map(|b| b.to_vec())
		} else {
			None
		}
	}
}

pub async fn search_album(client: &Client, album: &str, artist: &str) -> Option<String> {
	sleep(Duration::from_millis(1100)).await;

	let query = format!("release:\"{}\" AND artist:\"{}\"", album, artist);
	let url = format!(
		"{}/release?query={}&limit=1&fmt=json",
		MB_BASE,
		urlencoding::encode(&query)
	);

	#[derive(Deserialize)]
	struct MbReleaseSearch {
		releases: Vec<MbReleaseResult>,
	}
	#[derive(Deserialize)]
	struct MbReleaseResult {
		id: String,
	}

	let resp = client.get(&url).send().await.ok()?;
	if !resp.status().is_success() {
		return None;
	}
	let body = resp.text().await.ok()?;
	let data: MbReleaseSearch = serde_json::from_str(&body).ok()?;
	data.releases.into_iter().next().map(|r| r.id)
}