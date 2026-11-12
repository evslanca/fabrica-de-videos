// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Factory Timeline Assembler.
//!
//! Takes all processed factory node outputs (voice audio, B-roll clips, word
//! boundaries, SFX cues, caption bursts, ducking keyframes) and constructs a
//! fully-timed Concat project `Document` ready for GPU rendering.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::auto_ducking::{DuckingParams, GainKeyframe, build_ducking_keyframes};

/// A single visual scene unit ready for timeline placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledScene {
    /// Scene sequence number.
    pub index: usize,
    /// Start timecode in seconds.
    pub start_seconds: f32,
    /// Duration in seconds.
    pub duration_seconds: f32,
    /// B-roll clip path.
    pub video_path: PathBuf,
    /// Narration audio for this scene.
    pub audio_path: Option<PathBuf>,
    /// Whether the clip is horizontally mirrored.
    pub mirror_horizontal: bool,
    /// Ken Burns motion applied (string tag, resolved at render time).
    pub ken_burns_motion: String,
}

/// Caption burst: a group of 1-3 words displayed simultaneously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionBurst {
    /// Words in this burst.
    pub words: Vec<String>,
    /// Burst start in seconds.
    pub start_seconds: f32,
    /// Burst end in seconds.
    pub end_seconds: f32,
    /// Index of the word currently being spoken (highlighted).
    pub active_word_index: usize,
}

/// Complete assembled factory project, ready for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledProject {
    /// Total video duration in seconds.
    pub total_duration_seconds: f32,
    /// Target resolution width.
    pub width: u32,
    /// Target resolution height.
    pub height: u32,
    /// Assembled video scenes in order.
    pub scenes: Vec<AssembledScene>,
    /// Caption bursts across the full video.
    pub caption_bursts: Vec<CaptionBurst>,
    /// BGM gain automation curve.
    pub ducking_keyframes: Vec<GainKeyframe>,
    /// Path to the BGM audio track.
    pub bgm_path: Option<PathBuf>,
    /// Path to thumbnail image.
    pub thumbnail_path: Option<PathBuf>,
}

impl AssembledProject {
    /// Assembles a complete project from factory node outputs.
    pub fn assemble(
        scenes: Vec<AssembledScene>,
        bgm_path: Option<PathBuf>,
        caption_bursts: Vec<CaptionBurst>,
        width: u32,
        height: u32,
        ducking_params: &DuckingParams,
    ) -> Self {
        let total_duration = scenes
            .last()
            .map(|s| s.start_seconds + s.duration_seconds)
            .unwrap_or(0.0);

        // Collect voice activity windows from scenes that have audio
        let voice_windows: Vec<(f32, f32)> = scenes
            .iter()
            .filter(|s| s.audio_path.is_some())
            .map(|s| (s.start_seconds, s.start_seconds + s.duration_seconds))
            .collect();

        let ducking_keyframes =
            build_ducking_keyframes(&voice_windows, total_duration, ducking_params);

        log::info!(
            "Factory Assembler: {} scenes, {:.1}s total, {} caption bursts, {} ducking keyframes",
            scenes.len(),
            total_duration,
            caption_bursts.len(),
            ducking_keyframes.len()
        );

        Self {
            total_duration_seconds: total_duration,
            width,
            height,
            scenes,
            caption_bursts,
            ducking_keyframes,
            bgm_path,
            thumbnail_path: None,
        }
    }
}
