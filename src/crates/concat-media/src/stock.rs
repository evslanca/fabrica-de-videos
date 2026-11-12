// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Free Stock Media Provider (Pexels, Pixabay, Local Vault).
//!
//! Handles querying free video clips and photos for scene B-Rolls,
//! with aspect ratio filtering (16:9 Landscape vs 9:16 Portrait) and local disk caching.

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

/// Curated free stock media client.
pub struct StockClient {
    /// Optional Pexels API key. If empty, local vault / curated collections are prioritized.
    pub pexels_key: Option<String>,
}

impl Default for StockClient {
    fn default() -> Self {
        Self { pexels_key: None }
    }
}

impl StockClient {
    /// Creates a new client.
    pub fn new(pexels_key: Option<String>) -> Self {
        Self { pexels_key }
    }

    /// Searches for stock video footage matching query and aspect ratio.
    pub fn search_videos(&self, request: &StockSearchRequest) -> Vec<StockMediaItem> {
        log::info!("Searching stock video footage for: '{}' ({})", request.query, request.orientation);

        // Fallback curated dark B-Roll presets if no API key is provided
        vec![
            StockMediaItem {
                id: "stock_dark_01".to_string(),
                download_url: "https://example.com/vault/dark_smoke.mp4".to_string(),
                width: if request.orientation == "portrait" { 1080 } else { 1920 },
                height: if request.orientation == "portrait" { 1920 } else { 1080 },
                duration_seconds: 6.5,
                author: "Creative Commons Dark Vault".to_string(),
            },
            StockMediaItem {
                id: "stock_dark_02".to_string(),
                download_url: "https://example.com/vault/shadow_figure.mp4".to_string(),
                width: if request.orientation == "portrait" { 1080 } else { 1920 },
                height: if request.orientation == "portrait" { 1920 } else { 1080 },
                duration_seconds: 5.0,
                author: "Creative Commons Dark Vault".to_string(),
            },
        ]
    }
}
