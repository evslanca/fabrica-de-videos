// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Dynamic Subtitles / Captions Node: Formats word-synchronized animated text (Hormozi / Dark style).

use serde::{Deserialize, Serialize};

/// Visual caption color palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptionTheme {
    /// Yellow highlight on active word, white on surrounding words.
    #[default]
    HighlighterYellow,
    /// Toxic neon green on active word, white on surrounding words.
    NeonGreen,
    /// Cyan blue on active word, pure white on surrounding words.
    ElectricCyan,
    /// Deep gold / amber highlight.
    DarkGold,
    /// Monochromatic crisp white with heavy black shadow.
    MonochromeBold,
}

/// Caption configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionNodeConfig {
    /// Visual style theme.
    pub theme: CaptionTheme,
    /// Font size (scaled for 1080p width).
    pub font_size: f32,
    /// Maximum words displayed simultaneously on screen (1 to 4).
    pub max_words_per_burst: usize,
    /// Vertical position offset (0.0 center, 0.35 bottom safe zone).
    pub vertical_offset: f32,
    /// Heavy outline width in pixels.
    pub outline_width: f32,
}

impl Default for CaptionNodeConfig {
    fn default() -> Self {
        Self {
            theme: CaptionTheme::HighlighterYellow,
            font_size: 64.0,
            max_words_per_burst: 3,
            vertical_offset: 0.25, // Safely within mobile Shorts UI zone
            outline_width: 8.0,
        }
    }
}
