// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Script and Storyline Node: Handles script ingestion, hook formulation,
//! and scene-by-scene timing segmentation.

use serde::{Deserialize, Serialize};
use crate::retention::RetentionHook;

/// Scene breakdown derived from the script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSegment {
    /// Sequential index of the scene.
    pub index: usize,
    /// Narration text for this scene.
    pub narration: String,
    /// Suggested keywords for finding B-roll footage.
    pub visual_keywords: Vec<String>,
    /// Estimated or actual duration in seconds.
    pub duration_seconds: f32,
    /// Whether this scene acts as a pattern interrupt (abrupt zoom, sfx, visual change).
    pub is_pattern_interrupt: bool,
}

/// Script node configuration and state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScriptNodeConfig {
    /// Full narrative text.
    pub raw_script: String,
    /// 3-second hook structure.
    pub hook: Option<RetentionHook>,
    /// Generated scene breakdown.
    pub scenes: Vec<SceneSegment>,
}

impl ScriptNodeConfig {
    /// Parses a raw script into structured scene segments based on sentence boundaries
    /// and retention intervals (target 3 to 5 seconds per visual change).
    pub fn parse_scenes(&mut self) {
        self.scenes.clear();
        let sentences: Vec<&str> = self
            .raw_script
            .split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for (idx, sentence) in sentences.into_iter().enumerate() {
            // Rough word count estimation (~2.5 words per second spoken rate)
            let word_count = sentence.split_whitespace().count();
            let duration = (word_count as f32 / 2.5).max(2.0);

            // Simple keyword extractor: takes words longer than 4 characters
            let keywords = sentence
                .split_whitespace()
                .filter(|w| w.len() > 4)
                .take(3)
                .map(|w| w.to_lowercase())
                .collect();

            self.scenes.push(SceneSegment {
                index: idx,
                narration: sentence.to_string(),
                visual_keywords: keywords,
                duration_seconds: duration,
                is_pattern_interrupt: idx % 3 == 0,
            });
        }
    }
}
