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

/// Computes a visual contrast + colorfulness score for an RGBA8 frame so the
/// thumbnail generator automatically picks the sharpest, highest-impact frame.
pub fn score_frame_contrast_rgba(pixels: &[u8]) -> f32 {
    if pixels.len() < 4 {
        return 0.0;
    }
    let count = (pixels.len() / 4) as f64;
    let mut sum_luma = 0.0_f64;
    let mut sum_luma_sq = 0.0_f64;
    let mut sum_sat = 0.0_f64;

    for px in pixels.chunks_exact(4) {
        let r = px[0] as f64 / 255.0;
        let g = px[1] as f64 / 255.0;
        let b = px[2] as f64 / 255.0;
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        sum_luma += luma;
        sum_luma_sq += luma * luma;
        sum_sat += max_c - min_c;
    }

    let mean = sum_luma / count;
    let variance = (sum_luma_sq / count - mean * mean).max(0.0);
    let std_dev = variance.sqrt();
    let mean_sat = sum_sat / count;
    (std_dev * 0.8 + mean_sat * 0.2) as f32
}

/// Selects the index of the highest-contrast frame from `frames`.
pub fn select_best_thumbnail_frame(frames: &[&[u8]]) -> Option<usize> {
    frames
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            score_frame_contrast_rgba(a).total_cmp(&score_frame_contrast_rgba(b))
        })
        .map(|(idx, _)| idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_highest_contrast_frame_for_thumbnail() {
        let flat_gray = vec![128u8, 128, 128, 255, 128, 128, 128, 255];
        let high_contrast = vec![10u8, 10, 10, 255, 250, 220, 20, 255];
        let best = select_best_thumbnail_frame(&[&flat_gray, &high_contrast]);
        assert_eq!(best, Some(1));
    }
}
