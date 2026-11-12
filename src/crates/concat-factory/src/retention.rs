// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! High-retention narrative heuristics: 3-second hook structures, open loops,
//! and pattern interrupt timing for YouTube Dark channels.

use serde::{Deserialize, Serialize};

/// 3-Second Hook formula.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionHook {
    /// Provocative question or bold claim (e.g. "What if I told you that...").
    pub opening_statement: String,
    /// Visual cue (e.g. extreme closeup, sudden glitch, burning document).
    pub visual_cue: String,
    /// Target duration in seconds (must be <= 3.5s).
    pub duration_seconds: f32,
}

impl Default for RetentionHook {
    fn default() -> Self {
        Self {
            opening_statement: "Você não deveria estar assistindo a isso...".to_string(),
            visual_cue: "dark glitch with rapid zoom".to_string(),
            duration_seconds: 3.0,
        }
    }
}

/// Pattern interrupt controller: calculates when to inject visual or acoustic changes
/// to reset viewer attention span.
pub struct PatternInterruptEngine;

impl PatternInterruptEngine {
    /// Recommends cut timestamps (in seconds) to keep pacing under 4.5 seconds per shot.
    pub fn calculate_interrupt_timestamps(total_duration_seconds: f32) -> Vec<f32> {
        let mut cuts = Vec::new();
        let mut current = 3.0; // Initial hook cut
        while current < total_duration_seconds {
            cuts.push(current);
            // Dynamic variance between 2.8s and 4.2s to prevent monotonous rhythm
            let interval = if cuts.len() % 2 == 0 { 3.2 } else { 4.0 };
            current += interval;
        }
        cuts
    }
}
