// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! SFX Asset Library: procedural assignment of whooshes, risers and impact hits
//! based on scene cut positions, pattern interrupts, and narrative tension markers.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Catalogue of built-in dark SFX assets embedded in the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SfxAsset {
    WhooshFast,
    WhooshEcho,
    SubBassDrop,
    CinematicBraam,
    TensionRiser,
    HeartbeatLoop,
    CaptionPop,
    GlassBreak,
}

impl SfxAsset {
    /// Relative path under the app's bundled `assets/sfx/` directory.
    pub fn asset_path(self) -> PathBuf {
        let name = match self {
            Self::WhooshFast      => "transitions/whoosh_fast.wav",
            Self::WhooshEcho      => "transitions/whoosh_echo.wav",
            Self::SubBassDrop     => "impacts/sub_bass_drop.wav",
            Self::CinematicBraam  => "impacts/cinematic_braam.wav",
            Self::TensionRiser    => "tension/riser_8s.wav",
            Self::HeartbeatLoop   => "tension/heartbeat_loop.wav",
            Self::CaptionPop      => "ui/caption_pop.wav",
            Self::GlassBreak      => "impacts/glass_break.wav",
        };
        PathBuf::from("assets/sfx").join(name)
    }

    /// Default volume (linear gain 0.0–1.0) for this asset.
    pub fn default_gain(self) -> f32 {
        match self {
            Self::WhooshFast | Self::WhooshEcho => 0.60,
            Self::SubBassDrop | Self::CinematicBraam => 0.75,
            Self::TensionRiser => 0.45,
            Self::HeartbeatLoop => 0.30,
            Self::CaptionPop => 0.35,
            Self::GlassBreak => 0.70,
        }
    }
}

/// A placed SFX event on the final timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfxEvent {
    /// Asset to play.
    pub asset: SfxAsset,
    /// Trigger time in seconds from video start.
    pub time_seconds: f32,
    /// Gain override (uses `asset.default_gain()` when None).
    pub gain_override: Option<f32>,
}

/// Procedurally assigns SFX to a list of scene cut timestamps.
pub fn assign_scene_sfx(scene_cut_times: &[f32], riser_before_last_cut: bool) -> Vec<SfxEvent> {
    let mut events = Vec::new();

    for (i, &cut_time) in scene_cut_times.iter().enumerate() {
        // Alternate between fast whoosh and echo whoosh for variety
        let asset = if i % 2 == 0 { SfxAsset::WhooshFast } else { SfxAsset::WhooshEcho };
        events.push(SfxEvent {
            asset,
            // Trigger slightly before the cut so the whoosh peaks on the new frame
            time_seconds: (cut_time - 0.08).max(0.0),
            gain_override: None,
        });

        // Every 3rd cut, add a cinematic impact
        if i % 3 == 2 {
            events.push(SfxEvent {
                asset: SfxAsset::SubBassDrop,
                time_seconds: cut_time,
                gain_override: Some(0.55),
            });
        }
    }

    // Riser leading into the last cut / reveal
    if riser_before_last_cut {
        if let Some(&last_cut) = scene_cut_times.last() {
            events.push(SfxEvent {
                asset: SfxAsset::TensionRiser,
                time_seconds: (last_cut - 8.0).max(0.0),
                gain_override: None,
            });
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_whooshes_subbass_and_riser() {
        let cuts = vec![3.0, 6.5, 10.0, 14.0];
        let events = assign_scene_sfx(&cuts, true);

        assert!(events.iter().any(|e| e.asset == SfxAsset::WhooshFast));
        assert!(events.iter().any(|e| e.asset == SfxAsset::WhooshEcho));
        assert!(events.iter().any(|e| e.asset == SfxAsset::SubBassDrop));
        assert!(events.iter().any(|e| e.asset == SfxAsset::TensionRiser));
    }
}
