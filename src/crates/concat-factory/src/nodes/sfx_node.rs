// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Sound Effects (SFX) and Atmosphere Node: Controls procedural sound injection
//! (whooshes, risers, hits) and background music auto-ducking.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Classification of sound effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SfxType {
    /// Fast transition whoosh for scene cuts.
    TransitionWhoosh,
    /// Sub-bass thud or cinematic hit for dramatic emphasis.
    ImpactHit,
    /// Rising pitch tension drone before a reveal.
    TensionRiser,
    /// Short pop/tick on caption keyword highlights.
    CaptionPop,
}

/// Schedulable SFX placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfxCue {
    /// Timestamp in milliseconds.
    pub time_ms: u64,
    /// Kind of effect.
    pub kind: SfxType,
    /// Specific audio file path.
    pub audio_path: PathBuf,
    /// Volume multiplier (0.0 to 1.0).
    pub volume: f32,
}

/// SFX and BGM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfxNodeConfig {
    /// Path to background music track.
    pub bgm_track: Option<PathBuf>,
    /// Base background music volume (e.g. -20 dB or 0.15 linear).
    pub bgm_volume: f32,
    /// How many decibels to attenuate during speech (-18 dB default).
    pub ducking_attenuation_db: f32,
    /// Whether to generate procedural whooshes on all scene transitions.
    pub auto_transition_whooshes: bool,
    /// Generated SFX cues for the project.
    pub cues: Vec<SfxCue>,
}

impl Default for SfxNodeConfig {
    fn default() -> Self {
        Self {
            bgm_track: None,
            bgm_volume: 0.18,
            ducking_attenuation_db: -18.0,
            auto_transition_whooshes: true,
            cues: Vec::new(),
        }
    }
}
