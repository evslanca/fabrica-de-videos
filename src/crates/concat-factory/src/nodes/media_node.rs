// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Media and B-Roll Curation Node: Fetches free video/photo footage from Pexels/Pixabay
//! and organizes local dark texture assets.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Stock media provider source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StockSource {
    /// Pexels API (Free stock videos and photos).
    #[default]
    Pexels,
    /// Pixabay API (Secondary free fallback).
    Pixabay,
    /// Local files and asset vault on disk.
    LocalVault,
}

/// A selected media clip for a scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMediaClip {
    /// Associated scene index.
    pub scene_index: usize,
    /// Local path to downloaded/cached media file.
    pub local_path: PathBuf,
    /// Original URL if downloaded from stock.
    pub source_url: Option<String>,
    /// Whether to apply dynamic Ken Burns zoom/pan.
    pub apply_ken_burns: bool,
    /// Whether to mirror horizontally (anti-reused content filter).
    pub mirror_horizontal: bool,
}

/// Media node configuration and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaNodeConfig {
    /// Selected stock provider.
    pub source: StockSource,
    /// Search queries per scene or global.
    pub default_query: String,
    /// Whether to enforce anti-reused content visual filtering.
    pub enable_anti_reused_filter: bool,
    /// Downloaded clips mapped to scene indices.
    pub scene_clips: Vec<SceneMediaClip>,
}

impl Default for MediaNodeConfig {
    fn default() -> Self {
        Self {
            source: StockSource::Pexels,
            default_query: "dark mystery cinematic".to_string(),
            enable_anti_reused_filter: true,
            scene_clips: Vec::new(),
        }
    }
}
