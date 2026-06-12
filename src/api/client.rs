use crate::api::models::*;
use crate::utils::{Quality, Result, YPlayerError};

const API_PATH: &str = "https://api.music.yandex.net:443/";
const CLIENT_ID: &str = "YandexMusicAndroid/24023621";

#[derive(Clone)]
pub struct YandexClient {
    client: reqwest::Client,
    user_id: Option<u64>,
}

impl YandexClient {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("OAuth {token}"))
                .map_err(|e| YPlayerError::Auth(format!("Invalid header: {e}")))?,
        );
        headers.insert(
            "X-Yandex-Music-Client",
            reqwest::header::HeaderValue::from_static(CLIENT_ID),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, user_id: None })
    }

    pub async fn init(&mut self) -> Result<()> {
        let uid = self.get_uid().await?;
        self.user_id = Some(uid);
        Ok(())
    }

    pub fn user_id(&self) -> Result<u64> {
        self.user_id.ok_or_else(|| YPlayerError::Auth("Not initialized".into()))
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(reqwest::Method::GET, path, None::<&str>).await
    }

    async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self, path: &str, body: Option<B>
    ) -> Result<T> {
        self.request(reqwest::Method::POST, path, body).await
    }

    async fn request<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self, method: reqwest::Method, path: &str, body: Option<B>
    ) -> Result<T> {
        let url = format!("{API_PATH}{path}");
        let mut req = self.client.request(method, &url);

        if let Some(b) = body {
            req = req.form(&b);
        }

        let resp = req.send().await?;
        let status = resp.status();

        let text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(YPlayerError::YandexMusicApi(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| YPlayerError::Parse(format!("JSON parse: {e}")))?;

        if let Some(error) = json["error"].as_object() {
            let msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
            return Err(YPlayerError::YandexMusicApi(msg.to_string()));
        }

        let result = json.get("result")
            .ok_or_else(|| YPlayerError::YandexMusicApi("Missing 'result' in response".into()))?;

        serde_json::from_value(result.clone())
            .map_err(|e| YPlayerError::Parse(format!("Result parse: {e}")))
    }

    async fn get_uid(&self) -> Result<u64> {
        let status: serde_json::Value = self.get("account/status").await?;
        let uid = status["account"]["uid"]
            .as_u64()
            .or_else(|| status["account"]["uid"].as_i64().map(|i| i as u64))
            .ok_or_else(|| YPlayerError::YandexMusicApi("UID not found".into()))?;
        Ok(uid)
    }

    // ── Liked Tracks ──

    /// Ids of all liked tracks, newest first. The full objects are heavy,
    /// so callers resolve them in chunks via get_tracks_batch.
    pub async fn get_liked_track_ids(&self) -> Result<Vec<String>> {
        let uid = self.user_id()?;
        let resp: serde_json::Value = self.get(&format!("users/{uid}/likes/tracks")).await?;

        Ok(resp["library"]["tracks"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|t| parse_id(&t["id"])).collect())
            .unwrap_or_default())
    }

    /// Resolve a chunk of liked-track ids into full tracks.
    pub async fn get_liked_tracks_chunk(&self, ids: &[String]) -> Result<Vec<YTrack>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let mut tracks = self.get_tracks_batch(ids).await?;
        for t in &mut tracks {
            t.liked = true;
        }
        Ok(tracks)
    }

    pub async fn get_tracks_batch(&self, ids: &[String]) -> Result<Vec<YTrack>> {
        let ids_str = ids.join(",");
        let tracks: Vec<serde_json::Value> = self.get(&format!("tracks?trackIds={ids_str}")).await?;
        Ok(tracks.iter().filter_map(|t| parse_track(t, false, None)).collect())
    }

    // ── Playlists ──

    pub async fn get_playlists(&self) -> Result<Vec<YPlaylist>> {
        let uid = self.user_id()?;
        let playlists: Vec<serde_json::Value> = self.get(
            &format!("users/{uid}/playlists/list")
        ).await?;

        Ok(playlists.into_iter().filter_map(|p| parse_playlist(&p)).collect())
    }

    pub async fn get_playlist_tracks(&self, kind: i64) -> Result<Vec<YTrack>> {
        let uid = self.user_id()?;
        let resp: serde_json::Value = self.get(
            &format!("users/{uid}/playlists/{kind}?richTracks=true")
        ).await?;

        let tracks = resp["tracks"]
            .as_array()
            .map(|arr| {
                arr.iter().filter_map(|t| {
                    let track = t.get("track").or_else(|| Some(t))?;
                    parse_track(track, false, t.get("index").and_then(|i| i.as_u64()))
                }).collect()
            })
            .unwrap_or_default();

        Ok(tracks)
    }

    // ── Albums ──

    pub async fn get_album(&self, album_id: &str) -> Result<YAlbum> {
        let resp: serde_json::Value = self.get(&format!("albums/{album_id}")).await?;

        let tracks = resp["volumes"]
            .as_array()
            .map(|volumes| {
                volumes.iter()
                    .filter_map(|v| v.as_array())
                    .flatten()
                    .filter_map(|t| parse_track(t, false, t.get("index").and_then(|i| i.as_u64())))
                    .collect()
            })
            .unwrap_or_default();

        Ok(YAlbum {
            id: parse_id(&resp["id"]).unwrap_or_default(),
            title: resp["title"].as_str().unwrap_or("").to_string(),
            artists: parse_artists(resp["artists"].as_array()),
            cover_uri: jfield(&resp, "coverUri", "cover_uri").as_str().map(String::from),
            track_count: jfield(&resp, "trackCount", "track_count").as_u64().unwrap_or(0) as u32,
            year: resp["year"].as_u64().map(|y| y as u32),
            genre: resp["genre"].as_str().map(String::from),
            liked: resp["liked"].as_bool().unwrap_or(false),
            tracks,
        })
    }

    // ── Artists ──

    pub async fn get_artist_tracks(&self, artist_id: &str) -> Result<Vec<YTrack>> {
        let resp: serde_json::Value = self.get(
            &format!("artists/{artist_id}/tracks?page=0&pageSize=50")
        ).await?;

        let tracks = resp["tracks"]
            .as_array()
            .map(|arr| parse_tracks(arr, false))
            .unwrap_or_default();

        Ok(tracks)
    }

    // ── Listening history ──

    /// Recently played tracks, newest first, up to `limit`.
    /// GET /music-history returns days (historyTabs) of play groups; full
    /// track objects are then fetched in one batch by their ids.
    pub async fn get_history(&self, limit: usize) -> Result<Vec<YTrack>> {
        let resp: serde_json::Value = self.get("music-history?fullModelsCount=0").await?;

        let empty = vec![];
        let mut ids: Vec<String> = Vec::new();
        'outer: for tab in jfield(&resp, "historyTabs", "history_tabs").as_array().unwrap_or(&empty) {
            for group in tab["items"].as_array().unwrap_or(&empty) {
                for item in group["tracks"].as_array().unwrap_or(&empty) {
                    let item_id = jfield(&item["data"], "itemId", "item_id");
                    if let Some(id) = parse_id(jfield(item_id, "trackId", "track_id")) {
                        if !ids.contains(&id) {
                            ids.push(id);
                            if ids.len() >= limit {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if ids.is_empty() {
            return Ok(vec![]);
        }
        self.get_tracks_batch(&ids).await
    }

    // ── Search ──

    pub async fn search(&self, query: &str, search_type: &str, page: u32) -> Result<YSearchResult> {
        let query_encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let resp: serde_json::Value = self.get(
            &format!("search?text={query_encoded}&page={page}&type={search_type}&nocorrect=false")
        ).await?;

        Ok(YSearchResult {
            tracks: resp["tracks"]["results"]
                .as_array().map(|a| parse_tracks(a, false)).unwrap_or_default(),
            artists: resp["artists"]["results"]
                .as_array().map(|a| a.iter().filter_map(parse_artist).collect()).unwrap_or_default(),
            albums: resp["albums"]["results"]
                .as_array().map(|a| a.iter().filter_map(|a| {
                    Some(YAlbum {
                        id: parse_id(&a["id"]).unwrap_or_default(),
                        title: a["title"].as_str()?.to_string(),
                        artists: parse_artists(a["artists"].as_array()),
                        cover_uri: jfield(a, "coverUri", "cover_uri").as_str().map(String::from),
                        track_count: jfield(a, "trackCount", "track_count").as_u64().unwrap_or(0) as u32,
                        year: a["year"].as_u64().map(|y| y as u32),
                        genre: a["genre"].as_str().map(String::from),
                        liked: false,
                        tracks: vec![],
                    })
                }).collect()).unwrap_or_default(),
            playlists: resp["playlists"]["results"]
                .as_array().map(|a| a.iter().filter_map(parse_playlist).collect()).unwrap_or_default(),
        })
    }

    // ── Track Download Info ──

    pub async fn get_track_download_info(&self, track_id: &str) -> Result<Vec<YTrackDownloadInfo>> {
        let infos: Vec<serde_json::Value> = self.get(
            &format!("tracks/{track_id}/download-info")
        ).await?;

        Ok(infos.iter().map(|i| YTrackDownloadInfo {
            url: i["downloadInfoUrl"].as_str().unwrap_or("").to_string(),
            codec: i["codec"].as_str().unwrap_or("").to_string(),
            bitrate: i["bitrateInKbps"].as_u64().unwrap_or(0) as u32,
            gain: i["gain"].as_bool().unwrap_or(false),
        }).collect())
    }

    pub async fn get_best_download_url(&self, track_id: &str, quality: &Quality) -> Result<String> {
        let infos = self.get_track_download_info(track_id).await?;

        if infos.is_empty() {
            return Err(YPlayerError::NotFound("No download info".into()));
        }

        let target_priority = quality.priority();

        let best = infos.iter()
            .filter(|i| {
                let q = match i.codec.as_str() {
                    "flac" => Quality::Flac,
                    "mp3" => Quality::Mp3_320,
                    "aac" => Quality::Aac,
                    _ => Quality::Any,
                };
                q.priority() <= target_priority
            })
            .max_by_key(|i| i.bitrate);

        let chosen = best.unwrap_or_else(|| {
            infos.iter().max_by_key(|i| i.bitrate).unwrap()
        });

        self.resolve_download_url(&chosen.url).await
    }

    async fn resolve_download_url(&self, info_url: &str) -> Result<String> {
        let resp = self.client.get(info_url).send().await?;
        let body = resp.text().await.unwrap_or_default();

        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_str(&body);
        let mut host = String::new();
        let mut path = String::new();
        let mut sig = String::new();
        let mut ts = String::new();
        let mut buf = Vec::new();
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                }
                Ok(Event::Text(ref t)) => {
                    let val = t.unescape().unwrap_or_default().to_string();
                    match current_tag.as_str() {
                        "host" => host = val,
                        "path" => path = val,
                        "s" => sig = val,
                        "ts" => ts = val,
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        if sig.is_empty() || ts.is_empty() || host.is_empty() || path.is_empty() {
            return Err(YPlayerError::NotFound(
                "Invalid download info XML: missing fields".into(),
            ));
        }

        use md5::{Digest, Md5};

        const SIGN_SALT: &str = "XGRlBW9FXlekgbPrRHuSiA";
        let hash = Md5::digest(format!("{}{}{}", SIGN_SALT, &path[1..], sig).as_bytes());
        let hex = format!("{:x}", hash);

        Ok(format!("https://{host}/get-mp3/{hex}/{ts}{path}"))
    }

    /// Download raw audio bytes from a resolved download URL.
    /// Used by the cache layer to avoid passing URLs directly to mpv.
    pub async fn download_audio(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(YPlayerError::YandexMusicApi(format!(
                "download failed: HTTP {}", resp.status()
            )));
        }
        let bytes = resp.bytes().await.map_err(|e|
            YPlayerError::YandexMusicApi(format!("read error: {e}"))
        )?;
        Ok(bytes.to_vec())
    }

    // ── Lyrics ──

    pub async fn get_lyrics(&self, track_id: &str) -> Result<String> {
        let resp: serde_json::Value = self.get(&format!("tracks/{track_id}/lyrics")).await?;
        resp["lyrics"].as_str()
            .map(String::from)
            .ok_or_else(|| YPlayerError::NotFound("Lyrics not found".into()))
    }

    // ── Like / Unlike ──

    pub async fn like_track(&self, track_id: &str) -> Result<()> {
        let uid = self.user_id()?;
        self.post::<_, serde_json::Value>(
            &format!("users/{uid}/likes/tracks/add-multiple"),
            Some([("track-ids", track_id)])
        ).await?;
        Ok(())
    }

    pub async fn unlike_track(&self, track_id: &str) -> Result<()> {
        let uid = self.user_id()?;
        self.post::<_, serde_json::Value>(
            &format!("users/{uid}/likes/tracks/remove-multiple"),
            Some([("track-ids", track_id)])
        ).await?;
        Ok(())
    }
}

/// The API returns camelCase keys ("durationMs"); some older payloads use
/// snake_case. Prefer camelCase, fall back to snake_case.
fn jfield<'a>(v: &'a serde_json::Value, camel: &str, snake: &str) -> &'a serde_json::Value {
    let c = &v[camel];
    if c.is_null() { &v[snake] } else { c }
}

/// Yandex ids arrive either as numbers or strings depending on endpoint.
fn parse_id(v: &serde_json::Value) -> Option<String> {
    v.as_i64()
        .map(|i| i.to_string())
        .or_else(|| v.as_str().map(String::from))
}

fn parse_playlist(p: &serde_json::Value) -> Option<YPlaylist> {
    let id = jfield(p, "playlistUuid", "playlist_id").as_str()
        .map(String::from)
        .or_else(|| p["kind"].as_i64().map(|k| k.to_string()))
        .unwrap_or_default();
    Some(YPlaylist {
        id,
        kind: p["kind"].as_i64().unwrap_or(0),
        title: p["title"].as_str()?.to_string(),
        owner_name: p["owner"]["name"].as_str()
            .or_else(|| p["owner"]["login"].as_str())
            .unwrap_or("")
            .to_string(),
        track_count: jfield(p, "trackCount", "track_count").as_u64().unwrap_or(0) as u32,
        cover_uri: p["cover"]["uri"].as_str()
            .or_else(|| jfield(p, "ogImage", "og_image").as_str())
            .map(String::from),
        description: p["description"].as_str().map(String::from),
        duration_ms: jfield(p, "durationMs", "duration_ms").as_u64().unwrap_or(0) as u32,
        liked: false,
        tracks: vec![],
    })
}

fn parse_track(t: &serde_json::Value, liked: bool, position: Option<u64>) -> Option<YTrack> {
    // A track without an id can't be downloaded or cached — skip it.
    let id = parse_id(&t["id"])?;
    let lyrics_info = jfield(t, "lyricsInfo", "lyrics_info");
    Some(YTrack {
        id,
        title: t["title"].as_str()?.to_string(),
        artists: parse_artists(t["artists"].as_array()),
        albums: t["albums"].as_array().map(|a| a.iter().filter_map(parse_album).collect()).unwrap_or_default(),
        duration_ms: jfield(t, "durationMs", "duration_ms").as_u64().unwrap_or(0) as u32,
        cover_uri: jfield(t, "coverUri", "cover_uri").as_str()
            .or_else(|| t["albums"].as_array().and_then(|a| a.first()).and_then(|a| jfield(a, "coverUri", "cover_uri").as_str()))
            .map(String::from),
        lyrics_available: jfield(t, "lyricsAvailable", "lyrics_available").as_bool().unwrap_or(false)
            || jfield(lyrics_info, "hasAvailableTextLyrics", "has_visible_lyrics").as_bool().unwrap_or(false)
            || jfield(lyrics_info, "hasAvailableSyncLyrics", "has_lyrics").as_bool().unwrap_or(false),
        explicit: t["explicit"].as_bool().unwrap_or(false),
        track_position: position.or_else(|| jfield(t, "trackPosition", "track_position").as_u64()).or_else(|| t["index"].as_u64()).map(|i| i as u32),
        liked,
    })
}

fn parse_tracks(arr: &[serde_json::Value], liked: bool) -> Vec<YTrack> {
    arr.iter().filter_map(|t| parse_track(t, liked, None)).collect()
}

fn parse_artists(arr: Option<&Vec<serde_json::Value>>) -> Vec<YArtist> {
    arr.map(|a| a.iter().filter_map(parse_artist).collect()).unwrap_or_default()
}

fn parse_artist(a: &serde_json::Value) -> Option<YArtist> {
    Some(YArtist {
        id: parse_id(&a["id"]).unwrap_or_default(),
        name: a["name"].as_str()?.to_string(),
        cover_uri: a["cover"]["uri"].as_str().map(String::from),
        genres: a["genres"].as_array().map(|g| {
            g.iter().filter_map(|g| g.as_str().map(String::from)).collect()
        }).unwrap_or_default(),
        tracks_count: a["counts"]["tracks"].as_u64()
            .or_else(|| jfield(a, "tracksCount", "tracks_count").as_u64())
            .map(|u| u as u32),
        albums_count: a["counts"]["directAlbums"].as_u64()
            .or_else(|| jfield(a, "albumsCount", "albums_count").as_u64())
            .map(|u| u as u32),
        liked: false,
    })
}

fn parse_album(a: &serde_json::Value) -> Option<YAlbum> {
    Some(YAlbum {
        id: parse_id(&a["id"]).unwrap_or_default(),
        title: a["title"].as_str()?.to_string(),
        artists: parse_artists(a["artists"].as_array()),
        cover_uri: jfield(a, "coverUri", "cover_uri").as_str().map(String::from),
        track_count: jfield(a, "trackCount", "track_count").as_u64().unwrap_or(0) as u32,
        year: a["year"].as_u64().map(|y| y as u32),
        genre: a["genre"].as_str().map(String::from),
        liked: false,
        tracks: vec![],
    })
}