use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

const MB_BASE: &str = "https://musicbrainz.org/ws/2";
const AUDIODB_BASE: &str = "https://www.theaudiodb.com/api/v1/json";
const USER_AGENT: &str = "Sonodex/1.0 (music library app)";

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

pub fn make_client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())
}

async fn search_musicbrainz(client: &Client, title: &str, artist: &str) -> Option<EnrichedMetadata> {
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

    let data: MbSearchResponse = resp.json().await.ok()?;
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

    let release_album_artist = release
        .and_then(|r| r.artist_credit.as_deref())
        .and_then(|ac| ac.first())
        .map(|ac| ac.artist.name.clone());

    let album_artist = release_album_artist.or_else(|| artists.first().cloned());

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
        serde_json::to_string(&vec![serde_json::json!({ "name": name, "track_number": null })])
            .unwrap_or_else(|_| "[]".to_string())
    });

    Some(EnrichedMetadata {
        title,
        artists: artists_json,
        album_artist,
        albums: albums_json,
        year,
        genres,
        bpm: None,
        key: None,
        artwork: None,
        mbid: Some(mbid),
    })
}

async fn search_audiodb(
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
    let artists = track.str_artist.as_deref().map(|a| {
        serde_json::to_string(&vec![a.to_string()]).unwrap_or_else(|_| "[]".to_string())
    });
    let album_artist = track.str_artist.clone();
    let album_name = track.str_album_stripped.or(track.str_album);
    let albums = album_name.map(|name| {
        serde_json::to_string(&vec![serde_json::json!({ "name": name, "track_number": null })])
            .unwrap_or_else(|_| "[]".to_string())
    });
    let year = track.int_year_released;
    let genres = track.str_genre.as_deref().map(|g| {
        serde_json::to_string(&vec![g.to_string()]).unwrap_or_else(|_| "[]".to_string())
    });
    let mbid = track.str_music_brainz_id;
    let bpm = track.int_track_bpm.as_deref().and_then(|s| s.parse::<f32>().ok());

    let artwork_url = track.str_track_thumb.or(track.str_artist_thumb);
    let artwork = if let Some(url) = artwork_url {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                resp.bytes().await.ok().map(|b| b.to_vec())
            } else {
                None
            }
        } else {
            None
        }
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
        key: None,
        artwork,
        mbid,
    })
}

fn apply_string(api: Option<String>, local: Option<String>, priority: &str) -> Option<String> {
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

fn apply_f32(api: Option<f32>, local: Option<f32>, priority: &str) -> Option<f32> {
    match priority {
        "api" => api.or(local),
        _ => local.or(api),
    }
}

fn apply_bytes(api: Option<Vec<u8>>, local: Option<Vec<u8>>, priority: &str) -> Option<Vec<u8>> {
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

    let enriched = match settings.primary_api.as_str() {
        "audiodb" => {
            let result = search_audiodb(client, &title, &artist, &settings.audiodb_key).await;
            if result.is_none() {
                search_musicbrainz(client, &title, &artist).await
            } else {
                result
            }
        }
        _ => {
            let result = search_musicbrainz(client, &title, &artist).await;
            if result.is_none() {
                search_audiodb(client, &title, &artist, &settings.audiodb_key).await
            } else {
                result
            }
        }
    }
    .ok_or_else(|| "Track not found in any API".to_string())?;

    Ok(EnrichResult {
        title: apply_string(enriched.title, track.title.clone(), &settings.priority_title),
        artists: apply_string(enriched.artists, track.artists.clone(), &settings.priority_artists),
        album_artist: apply_string(enriched.album_artist, track.album_artist.clone(), &settings.priority_album_artist),
        albums: apply_string(enriched.albums, track.albums.clone(), &settings.priority_album),
        year: apply_string(enriched.year, track.year.clone(), &settings.priority_year),
        genres: apply_string(enriched.genres, track.genres.clone(), &settings.priority_genres),
        bpm: apply_f32(enriched.bpm, track.bpm, &settings.priority_bpm),
        key: apply_string(enriched.key, track.key.clone(), &settings.priority_key),
        artwork: apply_bytes(enriched.artwork, track.existing_artwork.clone(), &settings.priority_artwork),
    })
}
