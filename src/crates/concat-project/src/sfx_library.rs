// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! SFX Asset Library: procedural assignment of whooshes, risers and impact hits
//! based on scene cut positions, pattern interrupts, and narrative tension markers.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Catalogue of built-in dark SFX assets embedded in the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SfxAsset {
    /// Quick transition whoosh for fast cuts.
    WhooshFast,
    /// Reverberant whoosh for dramatic scene shifts.
    WhooshEcho,
    /// Low-frequency sub-bass drop for hook or reveal hits.
    SubBassDrop,
    /// Heavy cinematic brass/braam impact.
    CinematicBraam,
    /// Rising tension swell leading into a climax.
    TensionRiser,
    /// Rhythmic heartbeat loop for suspenseful segments.
    HeartbeatLoop,
    /// Subtle UI pop for caption bursts.
    CaptionPop,
    /// Sharp glass shatter accent for shock moments.
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

/// All built-in SFX assets for batch materialization.
pub const ALL_SFX_ASSETS: [SfxAsset; 8] = [
    SfxAsset::WhooshFast,
    SfxAsset::WhooshEcho,
    SfxAsset::SubBassDrop,
    SfxAsset::CinematicBraam,
    SfxAsset::TensionRiser,
    SfxAsset::HeartbeatLoop,
    SfxAsset::CaptionPop,
    SfxAsset::GlassBreak,
];

/// Synthesizes 44.1 kHz 16-bit mono PCM samples for `asset` in memory.
pub fn synthesize_sfx_samples(asset: SfxAsset) -> Vec<i16> {
    let sample_rate = 44_100u32;
    let duration_secs: f32 = match asset {
        SfxAsset::WhooshFast => 0.45,
        SfxAsset::WhooshEcho => 0.85,
        SfxAsset::SubBassDrop => 1.2,
        SfxAsset::CinematicBraam => 1.8,
        SfxAsset::TensionRiser => 2.5,
        SfxAsset::HeartbeatLoop => 1.2,
        SfxAsset::CaptionPop => 0.12,
        SfxAsset::GlassBreak => 0.55,
    };
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut pcm = Vec::with_capacity(num_samples);
    let mut rng_state: u32 = 0x1357_2468;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let u = t / duration_secs;
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((rng_state >> 16) as f32 / 32768.0) - 1.0;

        let sample = match asset {
            SfxAsset::WhooshFast | SfxAsset::WhooshEcho => {
                let env = (std::f32::consts::PI * u).sin().powf(1.6);
                let sweep = (2.0 * std::f32::consts::PI * (180.0 + 900.0 * u) * t).sin();
                (noise * 0.55 + sweep * 0.35) * env
            }
            SfxAsset::SubBassDrop => {
                let freq = 115.0 * (1.0 - 0.72 * u);
                let env = (1.0 - u).powf(1.8);
                (2.0 * std::f32::consts::PI * freq * t).sin() * env * 0.9
            }
            SfxAsset::CinematicBraam => {
                let env = (u * 8.0).min(1.0) * (1.0 - u).powf(1.2);
                let f1 = (2.0 * std::f32::consts::PI * 55.0 * t).sin();
                let f2 = (2.0 * std::f32::consts::PI * 110.0 * t).sin() * 0.5;
                (f1 + f2 + noise * 0.12) * env * 0.75
            }
            SfxAsset::TensionRiser => {
                let freq = 90.0 + 480.0 * u * u;
                let env = u.powf(1.5) * (1.0 - (u - 0.92).max(0.0) / 0.08).max(0.0);
                ((2.0 * std::f32::consts::PI * freq * t).sin() * 0.6 + noise * 0.25) * env
            }
            SfxAsset::HeartbeatLoop => {
                let beat1 = (-((t - 0.15) * 28.0).powi(2)).exp();
                let beat2 = (-((t - 0.42) * 32.0).powi(2)).exp() * 0.75;
                (2.0 * std::f32::consts::PI * 52.0 * t).sin() * (beat1 + beat2)
            }
            SfxAsset::CaptionPop => {
                let env = (1.0 - u).powf(3.0);
                (2.0 * std::f32::consts::PI * (650.0 - 300.0 * u) * t).sin() * env * 0.8
            }
            SfxAsset::GlassBreak => {
                let env = (1.0 - u).powf(2.5);
                let ring = (2.0 * std::f32::consts::PI * 3200.0 * t).sin() * 0.4;
                (noise * 0.7 + ring) * env
            }
        };
        let clamped = (sample.clamp(-0.98, 0.98) * i16::MAX as f32) as i16;
        pcm.push(clamped);
    }
    pcm
}

/// Procedurally synthesizes a 44.1 kHz 16-bit mono PCM `.wav` file for `asset`
/// if it does not already exist at `output_path`.
pub fn synthesize_sfx_wav(asset: SfxAsset, output_path: &Path) -> io::Result<PathBuf> {
    if output_path.exists() {
        return Ok(output_path.to_path_buf());
    }
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let pcm = synthesize_sfx_samples(asset);
    write_mono_wav(output_path, 44_100, &pcm)?;
    Ok(output_path.to_path_buf())
}

/// Materializes the entire built-in SFX library under `base_dir` (e.g. project root).
pub fn ensure_sfx_library(base_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::with_capacity(ALL_SFX_ASSETS.len());
    for asset in ALL_SFX_ASSETS {
        let target = base_dir.join(asset.asset_path());
        paths.push(synthesize_sfx_wav(asset, &target)?);
    }
    Ok(paths)
}

fn write_mono_wav(path: &Path, sample_rate: u32, samples: &[i16]) -> io::Result<()> {
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;
    let byte_rate = sample_rate * 2;
    let mut f = File::create(path)?;
    f.write_all(b"RIFF")?;
    f.write_all(&file_len.to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&byte_rate.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for &s in samples {
        f.write_all(&s.to_le_bytes())?;
    }
    Ok(())
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

    #[test]
    fn synthesizes_valid_sfx_samples_for_all_assets() {
        for asset in ALL_SFX_ASSETS {
            let pcm = synthesize_sfx_samples(asset);
            assert!(pcm.len() > 4000);
            assert!(pcm.iter().any(|&s| s != 0));
        }
    }
}
