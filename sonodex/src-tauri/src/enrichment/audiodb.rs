use super::EnrichedMetadata;
use reqwest::Client;
use serde::Deserialize;

const AUDIODB_BASE: &str = "https://www.theaudiodb.com/api/v1/json";

// ─────────────────────────────────────────────
// RESPONSE TYPES
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AudioDbSearchResponse {
    track: Option<Vec<AudioDbTrack>>,
}

#[derive(Debug, Deserialize)]
struct AudioDbTrack {
    #[serde(rename = "strTrack")]
    str_track: Option<String>,
    #[serde(rename = "strArtist")]
    str_artist: Option<String>,
    #[serde(rename = "strAlbum")]
    str_album: Option<String>,
    #[serde(rename = "strAlbumStripped")]
    str_album_stripped: Option<String>,
    #[serde(rename = "intYearReleased")]
    int_year_released: Option<String>,
    #[serde(rename = "strGenre")]
    str_genre: Option<String>,
    #[serde(rename = "strMusicBrainzID")]
    str_music_brainz_id: Option<String>,
    #[serde(rename = "strTrackThumb")]
    str_track_thumb: Option<String>,
    #[serde(rename = "strArtistThumb")]
    str_artist_thumb: Option<String>,
    #[serde(rename = "intTrackBPM")]
    int_track_bpm: Option<String>,
}

// ─────────────────────────────────────────────
// SEARCH
// ─────────────────────────────────────────────

pub async fn search(
    client: &Client,
    title: &str,
    artist: &str,
    api_key: &str,
) -> Option<EnrichedMetadata> {
    let key = if api_key.is_empty() { "2" } else { api_key };
    let url = format!(
        "{}/{}/searchtrack.php?s={}&t={}",
        AUDIODB_BASE,
        key,
        urlencoding::encode(artist),
        urlencoding::encode(title)
    );

    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let data: AudioDbSearchResponse = resp.json().await.ok()?;
    let track = data.track?.into_iter().next()?;

    let title = track.str_track.clone();
    let artists = track
        .str_artist
        .as_deref()
        .map(|a| serde_json::to_string(&vec![a.to_string()]).unwrap_or_else(|_| "[]".to_string()));
    let album_artist = track.str_artist.clone();
    let album_name = track.str_album_stripped.or(track.str_album);
    let albums = album_name.map(|name| {
        serde_json::to_string(&vec![
            serde_json::json!({ "name": name, "track_number": null }),
        ])
        .unwrap_or_else(|_| "[]".to_string())
    });
    let year = track.int_year_released;
    let genres = track
        .str_genre
        .as_deref()
        .map(|g| serde_json::to_string(&vec![g.to_string()]).unwrap_or_else(|_| "[]".to_string()));
    let mbid = track.str_music_brainz_id;
    let bpm = track
        .int_track_bpm
        .as_deref()
        .and_then(|s| s.parse::<f32>().ok());

    let artwork_url = track.str_track_thumb.or(track.str_artist_thumb);
    let artwork = if let Some(url) = artwork_url {
        client
            .get(&url)
            .send()
            .await
            .ok()
            .filter(|r| r.status().is_success())
            .and_then(|r| futures::executor::block_on(r.bytes()).ok())
            .map(|b| b.to_vec())
    } else {
        None
    };

    Some(EnrichedMetadata {
        title,
        artists,
        album_artist,
        albums,
        year,
        genres,
        bpm,
        artwork,
        mbid,
        ..Default::default()
    })
}
