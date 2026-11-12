// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! High-CTR Thumbnail Generator: Composes high-contrast cover images
//! with dark atmospheric grading and bold typography.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// High CTR Thumbnail layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailConfig {
    /// Source frame extracted from video timeline.
    pub source_frame_path: Option<PathBuf>,
    /// High-impact catchphrase (e.g., "O SEGREDO REVELADO").
    pub overlay_text: String,
    /// Text color hex (e.g. "#FFEB3B" neon yellow, "#FFFFFF" crisp white).
    pub text_color_hex: String,
    /// Shadow and outline thickness.
    pub outline_thickness: f32,
    /// Dark edge vignette strength (0.0 to 1.0).
    pub vignette_intensity: f32,
    /// Output thumbnail image path.
    pub output_path: PathBuf,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self {
            source_frame_path: None,
            overlay_text: "NÃO ASSISTA SOZINHO".to_string(),
            text_color_hex: "#FFE500".to_string(),
            outline_thickness: 12.0,
            vignette_intensity: 0.75,
            output_path: PathBuf::from("export/thumb.jpg"),
        }
    }
}
