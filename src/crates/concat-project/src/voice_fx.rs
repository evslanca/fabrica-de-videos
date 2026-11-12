// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Voice FX Effects & Studio Mastering Chain (DSP Parameters).
//!
//! Configures broadcast-grade vocal mastering (-14 LUFS target, HPF 80Hz,
//! presence EQ, de-esser, limiter -1 dBTP) as well as creative dark-channel
//! voice FX presets (Cavernous Reverb, Vintage Radio, Dark Pitch Down, etc.).

use serde::{Deserialize, Serialize};

/// Creative voice FX preset applied on top of narration or vocals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceFxPreset {
    /// Clean studio broadcast voice (YouTube / documentary standard).
    #[default]
    StudioBroadcast,
    /// Deep cavernous reverb for horror/supernatural stories.
    CavernousReverb,
    /// Band-limited vintage radio / phone call effect.
    VintageRadio,
    /// Subtle pitch-down (-2 semitones) for ominous dark narration.
    DarkPitchDown,
    /// Spectral delay echo for dramatic reveals.
    SpectralEcho,
    /// Warm analog tape saturation.
    AnalogSaturation,
}

/// Studio vocal mastering chain configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteringChainConfig {
    /// High-pass filter cutoff in Hz (removes rumble below 80 Hz).
    pub hpf_hz: f32,
    /// Presence boost around 4 kHz in dB (+2.5 dB default).
    pub presence_boost_db: f32,
    /// De-esser threshold in dB (-18 dB default).
    pub deesser_threshold_db: f32,
    /// True-peak limiter ceiling in dBTP (-1.0 dBTP default).
    pub limiter_ceiling_db: f32,
    /// Target integrated loudness in LUFS (-14.0 LUFS for YouTube).
    pub target_lufs: f32,
    /// Creative FX preset.
    pub fx_preset: VoiceFxPreset,
}

impl Default for MasteringChainConfig {
    fn default() -> Self {
        Self {
            hpf_hz: 80.0,
            presence_boost_db: 2.5,
            deesser_threshold_db: -18.0,
            limiter_ceiling_db: -1.0,
            target_lufs: -14.0,
            fx_preset: VoiceFxPreset::StudioBroadcast,
        }
    }
}

/// Computes the gain adjustment in dB required to normalize measured audio to `target_lufs`,
/// clamped to a safe range `[-18.0, +18.0]` dB.
pub fn estimate_lufs_gain_db(measured_lufs: f32, target_lufs: f32) -> f32 {
    (target_lufs - measured_lufs).clamp(-18.0, 18.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lufs_gain_calculation_and_clamping() {
        assert!((estimate_lufs_gain_db(-20.0, -14.0) - 6.0).abs() < 1e-5);
        assert!((estimate_lufs_gain_db(-10.0, -14.0) - (-4.0)).abs() < 1e-5);
        assert!((estimate_lufs_gain_db(-50.0, -14.0) - 18.0).abs() < 1e-5);
    }
}
