// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Automatic audio ducking and BGM volume envelope.
//!
//! Analyses the voice narration amplitude envelope and generates a gain automation
//! curve that attenuates the background music track during speech passages,
//! creating professional-sounding dark narration with no manual mixing.

use serde::{Deserialize, Serialize};

/// A single automation keyframe (time, gain).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GainKeyframe {
    /// Timestamp in seconds.
    pub time_seconds: f32,
    /// Linear gain (1.0 = full volume, 0.0 = silent).
    pub gain: f32,
}

/// Parameters for the auto-ducking algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckingParams {
    /// Full BGM gain when no voice is active (e.g. 0.18 = -15 dB).
    pub bgm_base_gain: f32,
    /// Reduced BGM gain during voice passages (e.g. 0.05 = -26 dB).
    pub bgm_ducked_gain: f32,
    /// Fade-down time in seconds when voice starts.
    pub attack_seconds: f32,
    /// Fade-up time in seconds after voice ends.
    pub release_seconds: f32,
    /// Silence threshold below which audio is considered "no voice" (0.0–1.0).
    pub silence_threshold: f32,
}

impl Default for DuckingParams {
    fn default() -> Self {
        Self {
            bgm_base_gain: 0.18,
            bgm_ducked_gain: 0.05,
            attack_seconds: 0.12,
            release_seconds: 0.4,
            silence_threshold: 0.02,
        }
    }
}

/// Builds a gain automation curve for the BGM track from voice activity windows.
///
/// `voice_windows` is a list of `(start_seconds, end_seconds)` segments
/// where the narrator is actively speaking. The result is a keyframe sequence
/// suitable for passing to the timeline assembler.
pub fn build_ducking_keyframes(
    voice_windows: &[(f32, f32)],
    total_duration: f32,
    params: &DuckingParams,
) -> Vec<GainKeyframe> {
    let mut kf = Vec::new();

    if voice_windows.is_empty() {
        kf.push(GainKeyframe { time_seconds: 0.0, gain: params.bgm_base_gain });
        kf.push(GainKeyframe { time_seconds: total_duration, gain: params.bgm_base_gain });
        return kf;
    }

    let mut cursor = 0.0_f32;

    for &(start, end) in voice_windows {
        // BGM at full level up to just before the voice attack begins
        if start > cursor {
            kf.push(GainKeyframe { time_seconds: cursor, gain: params.bgm_base_gain });
        }

        // Attack: quick fade-down just before speech starts
        let attack_start = (start - params.attack_seconds).max(cursor);
        kf.push(GainKeyframe { time_seconds: attack_start, gain: params.bgm_base_gain });
        kf.push(GainKeyframe { time_seconds: start, gain: params.bgm_ducked_gain });

        // Hold through the voice window
        kf.push(GainKeyframe { time_seconds: end, gain: params.bgm_ducked_gain });

        // Release: smooth fade-up after speech ends
        let release_end = end + params.release_seconds;
        kf.push(GainKeyframe { time_seconds: release_end, gain: params.bgm_base_gain });

        cursor = release_end;
    }

    // Final keyframe at full level to end of track
    if cursor < total_duration {
        kf.push(GainKeyframe { time_seconds: total_duration, gain: params.bgm_base_gain });
    }

    kf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ducking_attenuates_during_voice_and_restores_after() {
        let params = DuckingParams::default();
        let keyframes = build_ducking_keyframes(&[(2.0, 5.0)], 10.0, &params);

        // Should start at base gain, drop to ducked gain at 2.0, hold until 5.0, restore after release
        assert!((keyframes.first().unwrap().gain - params.bgm_base_gain).abs() < 1e-6);
        assert!(keyframes.iter().any(|k| (k.time_seconds - 2.0).abs() < 1e-6 && (k.gain - params.bgm_ducked_gain).abs() < 1e-6));
        assert!(keyframes.iter().any(|k| (k.time_seconds - 5.0).abs() < 1e-6 && (k.gain - params.bgm_ducked_gain).abs() < 1e-6));
        assert!((keyframes.last().unwrap().gain - params.bgm_base_gain).abs() < 1e-6);
    }

    #[test]
    fn empty_voice_windows_keeps_base_gain() {
        let params = DuckingParams::default();
        let keyframes = build_ducking_keyframes(&[], 8.0, &params);
        assert_eq!(keyframes.len(), 2);
        assert!((keyframes[0].gain - params.bgm_base_gain).abs() < 1e-6);
        assert!((keyframes[1].gain - params.bgm_base_gain).abs() < 1e-6);
    }
}
