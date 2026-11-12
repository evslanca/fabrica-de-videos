// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Procedural Ken Burns, Film Grain, and Vignette Effects for Dark Videos.
//!
//! Applies dynamic camera movement (pan & slow zoom) to static footage/images,
//! and overlays 35mm film grain + dark vignettes to defeat "reused content" flags
//! while establishing cinematic tension.

use serde::{Deserialize, Serialize};

/// Ken Burns camera motion trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KenBurnsMotion {
    /// Slow zoom into the center of the frame (1.0x -> 1.15x).
    #[default]
    ZoomInCenter,
    /// Slow zoom out revealing surrounding context (1.15x -> 1.0x).
    ZoomOutCenter,
    /// Pan smoothly from left to right with slight 1.08x zoom.
    PanLeftToRight,
    /// Pan smoothly from right to left with slight 1.08x zoom.
    PanRightToLeft,
}

/// Parameters for anti-reused content visual filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiReusedFilterParams {
    /// Ken Burns camera trajectory.
    pub motion: KenBurnsMotion,
    /// Intensity of 35mm film grain (0.0 = none, 0.15 = subtle cinematic).
    pub film_grain_intensity: f32,
    /// Edge vignette darkness (0.0 = none, 0.45 = deep dramatic corners).
    pub vignette_darkness: f32,
    /// Micro speed variation factor (e.g. 1.015x to change digital audio/video hash).
    pub speed_variation: f32,
    /// Whether horizontal mirroring is permitted.
    pub allow_horizontal_flip: bool,
}

impl Default for AntiReusedFilterParams {
    fn default() -> Self {
        Self {
            motion: KenBurnsMotion::ZoomInCenter,
            film_grain_intensity: 0.12,
            vignette_darkness: 0.40,
            speed_variation: 1.012,
            allow_horizontal_flip: true,
        }
    }
}
