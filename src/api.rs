use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const INNERTUBE_URL: &str = "https://music.youtube.com/youtubei/v1";
const API_KEY: &str = "AIzaSyC9XL3ZjWddXya6X74dJoCTL-WEYFDNX30";
const CLIENT_NAME: &str = "WEB_REMIX";
const CLIENT_VERSION: &str = "1.20241023.01.00";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_text: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_explicit: bool,
}

impl Track {
    pub fn youtube_url(&self) -> String {
        format!("https://music.youtube.com/watch?v={}", self.video_id)
    }

    #[allow(dead_code)]
    pub fn display_title(&self, max_width: usize) -> String {
        if self.title.len() > max_width {
            format!("{}...", &self.title[..max_width.saturating_sub(3)])
        } else {
            self.title.clone()
        }
    }
}

#[derive(Clone)]
pub struct YtMusicClient {
    http: reqwest::Client,
}

impl YtMusicClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .build()
                .expect("failed to create http client"),
        }
    }

    fn context_body(&self) -> Value {
        serde_json::json!({
            "context": {
                "client": {
                    "clientName": CLIENT_NAME,
                    "clientVersion": CLIENT_VERSION,
                    "hl": "en",
                    "gl": "US",
                }
            }
        })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Track>> {
        let mut body = self.context_body();
        body["query"] = Value::String(query.to_string());
        body["params"] = Value::String("EgWKAQIIAQ%3D%3D".to_string());

        let resp = self
            .http
            .post(format!(
                "{}/search?key={}&prettyPrint=false",
                INNERTUBE_URL, API_KEY
            ))
            .header("Content-Type", "application/json")
            .header("Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/")
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(parse_search_results(&resp))
    }

    #[allow(dead_code)]
    pub async fn get_suggestions(&self, query: &str) -> Result<Vec<String>> {
        let mut body = self.context_body();
        body["input"] = Value::String(query.to_string());

        let resp = self
            .http
            .post(format!(
                "{}/music/get_search_suggestions?key={}&prettyPrint=false",
                INNERTUBE_URL, API_KEY
            ))
            .header("Content-Type", "application/json")
            .header("Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/")
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(parse_suggestions(&resp))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LyricsResponse {
    #[serde(rename = "plainLyrics")]
    pub plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
    pub instrumental: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LyricLine {
    pub time_ms: u64,
    pub text: String,
}

impl YtMusicClient {
    pub async fn fetch_lyrics(
        &self,
        track: &str,
        artist: &str,
        duration_secs: Option<u64>,
    ) -> Result<Option<LyricsResponse>> {
        let mut url = reqwest::Url::parse("https://lrclib.net/api/get")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("track_name", track);
            q.append_pair("artist_name", artist);
            if let Some(d) = duration_secs {
                q.append_pair("duration", &d.to_string());
            }
        }
        let resp = self
            .http
            .get(url)
            .header("User-Agent", "ytmusic-tui/0.1.0")
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        Ok(Some(resp.json::<LyricsResponse>().await?))
    }
}

pub fn parse_synced_lyrics(synced: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    for line in synced.lines() {
        let line = line.trim();
        if line.len() < 10 || !line.starts_with('[') {
            continue;
        }
        if let Some(bracket_end) = line.find(']') {
            let timestamp = &line[1..bracket_end];
            let text = line[bracket_end + 1..].trim().to_string();
            if let Some(ms) = parse_lrc_timestamp(timestamp) {
                lines.push(LyricLine { time_ms: ms, text });
            }
        }
    }
    lines
}

fn parse_lrc_timestamp(ts: &str) -> Option<u64> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let mins: f64 = parts[0].parse().ok()?;
    let secs: f64 = parts[1].parse().ok()?;
    Some((mins * 60_000.0 + secs * 1000.0) as u64)
}

pub fn duration_text_to_secs(text: &str) -> Option<u64> {
    let parts: Vec<&str> = text.split(':').collect();
    match parts.len() {
        2 => {
            let m: u64 = parts[0].parse().ok()?;
            let s: u64 = parts[1].parse().ok()?;
            Some(m * 60 + s)
        }
        3 => {
            let h: u64 = parts[0].parse().ok()?;
            let m: u64 = parts[1].parse().ok()?;
            let s: u64 = parts[2].parse().ok()?;
            Some(h * 3600 + m * 60 + s)
        }
        _ => None,
    }
}

fn parse_search_results(data: &Value) -> Vec<Track> {
    let mut tracks = Vec::new();

    let contents = traverse(data, &["contents", "tabbedSearchResultsRenderer", "tabs"]);

    let tabs = match contents.and_then(|v| v.as_array()) {
        Some(t) => t,
        None => return tracks,
    };

    for tab in tabs {
        let sections = traverse(
            tab,
            &["tabRenderer", "content", "sectionListRenderer", "contents"],
        );

        let sections = match sections.and_then(|v| v.as_array()) {
            Some(s) => s,
            None => continue,
        };

        for section in sections {
            let shelf_contents = section
                .get("musicShelfRenderer")
                .and_then(|s| s.get("contents"))
                .and_then(|c| c.as_array());

            let items = match shelf_contents {
                Some(i) => i,
                None => continue,
            };

            for item in items {
                if let Some(track) = parse_track_item(item) {
                    tracks.push(track);
                }
            }
        }
    }

    tracks
}

fn parse_track_item(item: &Value) -> Option<Track> {
    let renderer = item.get("musicResponsiveListItemRenderer")?;

    let video_id = renderer
        .get("playlistItemData")
        .and_then(|p| p.get("videoId"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            traverse(
                renderer,
                &[
                    "overlay",
                    "musicItemThumbnailOverlayRenderer",
                    "content",
                    "musicPlayButtonRenderer",
                    "playNavigationEndpoint",
                    "watchEndpoint",
                    "videoId",
                ],
            )
            .and_then(|v| v.as_str())
        })?;

    let columns = renderer.get("flexColumns")?.as_array()?;

    let title = columns
        .first()
        .and_then(|c| {
            traverse(
                c,
                &["musicResponsiveListItemFlexColumnRenderer", "text", "runs"],
            )
        })
        .and_then(extract_runs_text)
        .unwrap_or_default();

    if title.is_empty() {
        return None;
    }

    let subtitle_runs = columns.get(1).and_then(|c| {
        traverse(
            c,
            &["musicResponsiveListItemFlexColumnRenderer", "text", "runs"],
        )
    });

    let (artist, album, duration_from_subtitle) = parse_subtitle_runs(subtitle_runs);

    let duration_from_fixed = renderer
        .get("fixedColumns")
        .and_then(|fc| fc.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| traverse(c, &["musicResponsiveListItemFixedColumnRenderer", "text"]))
        .and_then(|text| {
            text.get("simpleText")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .or_else(|| extract_runs_text(text))
        })
        .filter(|t| is_duration_str(t));

    let duration_text = duration_from_fixed.or(duration_from_subtitle);

    let thumbnail_url = traverse(
        renderer,
        &[
            "thumbnail",
            "musicThumbnailRenderer",
            "thumbnail",
            "thumbnails",
        ],
    )
    .and_then(|t| t.as_array())
    .and_then(|arr| arr.last())
    .and_then(|t| t.get("url"))
    .and_then(|u| u.as_str())
    .map(|s| s.to_string());

    let is_explicit = renderer
        .get("badges")
        .and_then(|b| b.as_array())
        .map(|badges| {
            badges.iter().any(|b| {
                b.get("musicInlineBadgeRenderer")
                    .and_then(|r| r.get("icon"))
                    .and_then(|i| i.get("iconType"))
                    .and_then(|t| t.as_str())
                    == Some("MUSIC_EXPLICIT_BADGE")
            })
        })
        .unwrap_or(false);

    Some(Track {
        video_id: video_id.to_string(),
        title,
        artist,
        album,
        duration_text,
        thumbnail_url,
        is_explicit,
    })
}

fn is_duration_str(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let parts: Vec<&str> = s.split(':').collect();
    parts.len() >= 2 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

fn parse_subtitle_runs(runs_val: Option<&Value>) -> (String, Option<String>, Option<String>) {
    let runs = match runs_val.and_then(|v| v.as_array()) {
        Some(r) => r,
        None => return ("Unknown".to_string(), None, None),
    };

    let text_parts: Vec<&str> = runs
        .iter()
        .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
        .collect();

    let full_text = text_parts.join("");
    let segments: Vec<&str> = full_text.split(" \u{2022} ").collect();

    let duration = segments
        .iter()
        .rev()
        .find(|s| is_duration_str(s))
        .map(|s| s.to_string());

    let non_meta: Vec<&&str> = segments
        .iter()
        .filter(|s| !["Song", "Video", "EP", "Single", "Album"].contains(s) && !is_duration_str(s))
        .collect();

    let artist = non_meta
        .first()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let album = non_meta
        .iter()
        .rev()
        .find(|s| ***s != artist)
        .map(|s| s.to_string());

    (artist, album, duration)
}

fn parse_suggestions(data: &Value) -> Vec<String> {
    let contents = data
        .get("contents")
        .and_then(|c| c.as_array())
        .unwrap_or(&Vec::new())
        .clone();

    contents
        .iter()
        .filter_map(|item| {
            item.get("searchSuggestionsSectionRenderer")
                .and_then(|s| s.get("contents"))
                .and_then(|c| c.as_array())
        })
        .flatten()
        .filter_map(|suggestion| {
            let renderer = suggestion
                .get("searchSuggestionRenderer")
                .or_else(|| suggestion.get("musicResponsiveListItemRenderer"))?;

            let query_val = traverse(renderer, &["navigationEndpoint", "searchEndpoint", "query"]);
            traverse(renderer, &["suggestion", "runs"])
                .or(query_val)
                .and_then(|v| {
                    if v.is_string() {
                        v.as_str().map(|s| s.to_string())
                    } else {
                        extract_runs_text(v)
                    }
                })
        })
        .collect()
}

fn traverse<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for &key in path {
        current = current.get(key)?;
    }
    Some(current)
}

fn extract_runs_text(runs: &Value) -> Option<String> {
    let arr = runs.as_array()?;
    let text: String = arr
        .iter()
        .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
        .collect();

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
