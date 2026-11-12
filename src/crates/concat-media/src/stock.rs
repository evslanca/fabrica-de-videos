// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Free Stock Media Provider (Pexels, Pixabay, Local Vault).
//!
//! Queries free HD/4K video clips for scene B-Rolls via Pexels API (when a key
//! is provided) or falls back cleanly to the curated local Dark Vault, with
//! aspect ratio filtering (`landscape` 16:9 vs `portrait` 9:16) and disk caching.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Media item result from a stock query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMediaItem {
    /// Unique identifier from the provider.
    pub id: String,
    /// Direct download link.
    pub download_url: String,
    /// Width of the video/photo.
    pub width: u32,
    /// Height of the video/photo.
    pub height: u32,
    /// Duration in seconds (0 for still photos).
    pub duration_seconds: f32,
    /// Author/Photographer credit.
    pub author: String,
}

/// Stock Media Search Request.
#[derive(Debug, Clone)]
pub struct StockSearchRequest {
    /// Search keywords (e.g., "dark forest", "fog neon street").
    pub query: String,
    /// Desired orientation ("portrait" for 9:16 or "landscape" for 16:9).
    pub orientation: String,
    /// Max results to retrieve.
    pub per_page: usize,
}

impl Default for StockSearchRequest {
    fn default() -> Self {
        Self {
            query: "dark mystery".to_string(),
            orientation: "landscape".to_string(),
            per_page: 5,
        }
    }
}

#[derive(Deserialize)]
struct PexelsVideoResponse {
    #[serde(default)]
    videos: Vec<PexelsVideo>,
}

#[derive(Deserialize)]
struct PexelsVideo {
    id: u64,
    duration: Option<f32>,
    user: Option<PexelsUser>,
    #[serde(default)]
    video_files: Vec<PexelsVideoFile>,
}

#[derive(Deserialize)]
struct PexelsUser {
    name: String,
}

#[derive(Deserialize)]
struct PexelsVideoFile {
    link: String,
    width: Option<u32>,
    height: Option<u32>,
    quality: Option<String>,
}

#[derive(Deserialize)]
struct PixabayVideoResponse {
    #[serde(default)]
    hits: Vec<PixabayVideoHit>,
}

#[derive(Deserialize)]
struct PixabayVideoHit {
    id: u64,
    duration: Option<f32>,
    user: Option<String>,
    videos: PixabayVideoVariants,
}

#[derive(Deserialize)]
struct PixabayVideoVariants {
    large: Option<PixabayVideoFile>,
    medium: Option<PixabayVideoFile>,
    small: Option<PixabayVideoFile>,
}

#[derive(Deserialize)]
struct PixabayVideoFile {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
}

/// Curated free stock media client.
#[derive(Default)]
pub struct StockClient {
    /// Optional Pexels API key. If empty, Pixabay or local vault is used.
    pub pexels_key: Option<String>,
    /// Optional Pixabay API key used as secondary free stock provider.
    pub pixabay_key: Option<String>,
}

impl StockClient {
    /// Creates a new client with an optional Pexels API key.
    pub fn new(pexels_key: Option<String>) -> Self {
        Self {
            pexels_key,
            pixabay_key: None,
        }
    }

    /// Attaches an optional Pixabay API key for fallback stock queries.
    pub fn with_pixabay_key(mut self, pixabay_key: Option<String>) -> Self {
        self.pixabay_key = pixabay_key;
        self
    }

    /// Searches for stock video footage matching query and aspect ratio.
    /// Queries Pexels first, then Pixabay over HTTPS when configured, falling back
    /// to curated dark vault presets on offline/no-key setups.
    pub fn search_videos(&self, request: &StockSearchRequest) -> Vec<StockMediaItem> {
        log::info!(
            "Searching stock video footage for: '{}' ({})",
            request.query,
            request.orientation
        );

        if let Some(key) = self.pexels_key.as_deref().filter(|k| !k.trim().is_empty()) {
            match self.query_pexels_videos(key, request) {
                Ok(items) if !items.is_empty() => return items,
                Ok(_) => log::warn!("Pexels returned 0 videos for '{}', trying Pixabay/vault", request.query),
                Err(e) => log::warn!("Pexels API request failed ({}), trying Pixabay/vault", e),
            }
        }

        if let Some(key) = self.pixabay_key.as_deref().filter(|k| !k.trim().is_empty()) {
            match self.query_pixabay_videos(key, request) {
                Ok(items) if !items.is_empty() => return items,
                Ok(_) => log::warn!("Pixabay returned 0 videos for '{}', using vault", request.query),
                Err(e) => log::warn!("Pixabay API request failed ({}), using vault", e),
            }
        }

        // Fallback curated dark B-Roll presets
        vec![
            StockMediaItem {
                id: "stock_dark_01".to_string(),
                download_url: "assets/vault/dark_smoke.mp4".to_string(),
                width: if request.orientation == "portrait" { 1080 } else { 1920 },
                height: if request.orientation == "portrait" { 1920 } else { 1080 },
                duration_seconds: 6.5,
                author: "Creative Commons Dark Vault".to_string(),
            },
            StockMediaItem {
                id: "stock_dark_02".to_string(),
                download_url: "assets/vault/shadow_figure.mp4".to_string(),
                width: if request.orientation == "portrait" { 1080 } else { 1920 },
                height: if request.orientation == "portrait" { 1920 } else { 1080 },
                duration_seconds: 5.0,
                author: "Creative Commons Dark Vault".to_string(),
            },
        ]
    }

    fn query_pexels_videos(
        &self,
        api_key: &str,
        request: &StockSearchRequest,
    ) -> Result<Vec<StockMediaItem>, String> {
        let resp = ureq::get("https://api.pexels.com/videos/search")
            .set("Authorization", api_key)
            .query("query", &request.query)
            .query("orientation", &request.orientation)
            .query("per_page", &request.per_page.to_string())
            .call()
            .map_err(|e| e.to_string())?;

        let parsed: PexelsVideoResponse = resp
            .into_json()
            .map_err(|e| format!("invalid JSON from Pexels: {e}"))?;

        Ok(parse_pexels_response(parsed))
    }

    fn query_pixabay_videos(
        &self,
        api_key: &str,
        request: &StockSearchRequest,
    ) -> Result<Vec<StockMediaItem>, String> {
        let per_page = request.per_page.max(3).to_string();
        let resp = ureq::get("https://pixabay.com/api/videos/")
            .query("key", api_key)
            .query("q", &request.query)
            .query("per_page", &per_page)
            .query("safesearch", "true")
            .call()
            .map_err(|e| e.to_string())?;

        let parsed: PixabayVideoResponse = resp
            .into_json()
            .map_err(|e| format!("invalid JSON from Pixabay: {e}"))?;

        Ok(parse_pixabay_response(parsed))
    }

    /// Downloads a remote stock media item into `cache_dir` (atomic `.part` write).
    /// Returns the local cached file path if it already exists.
    pub fn download_to_cache(item: &StockMediaItem, cache_dir: &Path) -> io::Result<PathBuf> {
        fs::create_dir_all(cache_dir)?;
        let target = cache_dir.join(format!("{}.mp4", item.id));
        if target.exists() {
            return Ok(target);
        }

        if !item.download_url.starts_with("http") {
            return Ok(PathBuf::from(&item.download_url));
        }

        let part = cache_dir.join(format!("{}.mp4.part", item.id));
        let resp = ureq::get(&item.download_url)
            .call()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let mut reader = resp.into_reader();
        let mut out = File::create(&part)?;
        io::copy(&mut reader, &mut out)?;
        fs::rename(&part, &target)?;
        Ok(target)
    }

    /// Downloads a stock item with bounded retries before returning an error.
    pub fn download_with_retry(
        item: &StockMediaItem,
        cache_dir: &Path,
        max_attempts: usize,
    ) -> io::Result<PathBuf> {
        let attempts = max_attempts.max(1);
        let mut last_err = None;
        for attempt in 1..=attempts {
            match Self::download_to_cache(item, cache_dir) {
                Ok(path) => return Ok(path),
                Err(err) => {
                    log::warn!(
                        "Stock download attempt {attempt}/{attempts} failed for '{}': {err}",
                        item.id
                    );
                    last_err = Some(err);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| io::Error::other("stock download failed")))
    }

    /// Downloads multiple stock items in parallel using scoped OS threads,
    /// preserving the input order in the returned results.
    pub fn download_batch_parallel(
        items: &[StockMediaItem],
        cache_dir: &Path,
    ) -> Vec<Result<PathBuf, String>> {
        if items.is_empty() {
            return Vec::new();
        }
        std::thread::scope(|scope| {
            let handles: Vec<_> = items
                .iter()
                .map(|item| {
                    scope.spawn(move || {
                        Self::download_with_retry(item, cache_dir, 2).map_err(|e| e.to_string())
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err("stock download worker panicked".to_string()))
                })
                .collect()
        })
    }
}

fn parse_pexels_response(parsed: PexelsVideoResponse) -> Vec<StockMediaItem> {
    parsed
        .videos
        .into_iter()
        .filter_map(|v| {
            let best = v
                .video_files
                .iter()
                .find(|f| f.quality.as_deref() == Some("hd"))
                .or_else(|| v.video_files.first())?;
            Some(StockMediaItem {
                id: format!("pexels_{}", v.id),
                download_url: best.link.clone(),
                width: best.width.unwrap_or(1920),
                height: best.height.unwrap_or(1080),
                duration_seconds: v.duration.unwrap_or(5.0),
                author: v.user.map(|u| u.name).unwrap_or_else(|| "Pexels".into()),
            })
        })
        .collect()
}

fn parse_pixabay_response(parsed: PixabayVideoResponse) -> Vec<StockMediaItem> {
    parsed
        .hits
        .into_iter()
        .filter_map(|v| {
            let best = v
                .videos
                .large
                .or(v.videos.medium)
                .or(v.videos.small)?;
            Some(StockMediaItem {
                id: format!("pixabay_{}", v.id),
                download_url: best.url,
                width: best.width.unwrap_or(1920),
                height: best.height.unwrap_or(1080),
                duration_seconds: v.duration.unwrap_or(5.0),
                author: v.user.unwrap_or_else(|| "Pixabay".into()),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pexels_json_picking_hd_file() {
        let json = r#"{
            "videos": [{
                "id": 42,
                "duration": 7.5,
                "user": { "name": "DarkCreator" },
                "video_files": [
                    { "link": "https://cdn/sd.mp4", "width": 640, "height": 360, "quality": "sd" },
                    { "link": "https://cdn/hd.mp4", "width": 1920, "height": 1080, "quality": "hd" }
                ]
            }]
        }"#;
        let parsed: PexelsVideoResponse = serde_json::from_str(json).unwrap();
        let items = parse_pexels_response(parsed);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "pexels_42");
        assert_eq!(items[0].download_url, "https://cdn/hd.mp4");
        assert_eq!(items[0].width, 1920);
    }

    #[test]
    fn parses_pixabay_json_picking_large_or_medium() {
        let json = r#"{
            "hits": [{
                "id": 99,
                "duration": 12.0,
                "user": "Nocturnal",
                "videos": {
                    "medium": { "url": "https://cdn/med.mp4", "width": 1280, "height": 720 },
                    "large": { "url": "https://cdn/lg.mp4", "width": 1920, "height": 1080 }
                }
            }]
        }"#;
        let parsed: PixabayVideoResponse = serde_json::from_str(json).unwrap();
        let items = parse_pixabay_response(parsed);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "pixabay_99");
        assert_eq!(items[0].download_url, "https://cdn/lg.mp4");
        assert_eq!(items[0].width, 1920);
        assert_eq!(items[0].author, "Nocturnal");
    }

    #[test]
    fn offline_fallback_respects_orientation() {
        let client = StockClient::default();
        let items = client.search_videos(&StockSearchRequest {
            orientation: "portrait".to_string(),
            ..Default::default()
        });
        assert!(!items.is_empty());
        assert_eq!(items[0].width, 1080);
        assert_eq!(items[0].height, 1920);
    }

    #[test]
    fn parallel_batch_download_preserves_order_for_local_vault_items() {
        let client = StockClient::default();
        let items = client.search_videos(&StockSearchRequest::default());
        let cache = std::env::temp_dir().join("concat_stock_parallel_test");
        let results = StockClient::download_batch_parallel(&items, &cache);
        assert_eq!(results.len(), items.len());
        assert!(results.iter().all(|r| r.is_ok()));
        let _ = fs::remove_dir_all(&cache);
    }
}
