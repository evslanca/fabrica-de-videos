// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Voice FX Effects & Studio Mastering Chain (DSP Parameters).
//!
//! Configures broadcast-grade vocal mastering (-14 LUFS target, HPF 80Hz,
//! presence EQ, de-esser, limiter -1 dBTP) as well as creative dark-channel
//! voice FX presets (Cavernous Reverb, Vintage Radio, Dark Pitch Down, etc.).

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
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

/// Approximates integrated loudness (LUFS) from 16-bit PCM samples using RMS energy.
pub fn measure_pcm16_lufs(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return -70.0;
    }
    let sum_sq: f64 = samples
        .iter()
        .map(|&s| {
            let norm = s as f64 / i16::MAX as f64;
            norm * norm
        })
        .sum();
    let rms = (sum_sq / samples.len() as f64).sqrt();
    if rms <= 1e-9 {
        return -70.0;
    }
    (20.0 * rms.log10() as f32).clamp(-70.0, 0.0)
}

/// Normalizes 16-bit PCM samples in-place toward `target_lufs` while enforcing
/// a true-peak ceiling (`ceiling_db`, e.g. `-1.0` dBTP) to prevent clipping.
/// Returns the applied gain in dB.
pub fn normalize_pcm16_samples(samples: &mut [i16], target_lufs: f32, ceiling_db: f32) -> f32 {
    let measured = measure_pcm16_lufs(samples);
    let gain_db = estimate_lufs_gain_db(measured, target_lufs);
    let linear_gain = 10.0_f32.powf(gain_db / 20.0);
    let peak_limit = 10.0_f32.powf(ceiling_db / 20.0) * i16::MAX as f32;
    for s in samples.iter_mut() {
        *s = (*s as f32 * linear_gain).clamp(-peak_limit, peak_limit) as i16;
    }
    gain_db
}

/// Normalizes a 16-bit PCM WAV file toward `target_lufs` (e.g. `-14.0` for YouTube)
/// while enforcing a true-peak ceiling (`ceiling_db`, e.g. `-1.0` dBTP) to prevent clipping.
/// Returns the applied gain in dB.
pub fn normalize_pcm16_wav_to_lufs(
    input_wav: &Path,
    output_wav: &Path,
    target_lufs: f32,
    ceiling_db: f32,
) -> io::Result<f32> {
    let bytes = fs::read(input_wav)?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported or invalid WAV header",
        ));
    }

    let header = &bytes[..44];
    let payload = &bytes[44..];
    let mut samples: Vec<i16> = payload
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();

    let gain_db = normalize_pcm16_samples(&mut samples, target_lufs, ceiling_db);

    if let Some(parent) = output_wav.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let mut out = File::create(output_wav)?;
    out.write_all(header)?;
    for s in samples {
        out.write_all(&s.to_le_bytes())?;
    }
    Ok(gain_db)
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

    #[test]
    fn normalizes_pcm16_samples_without_clipping() {
        let mut samples = vec![1500i16, -1500i16, 1500, -1500, 1500, -1500];
        let applied_db = normalize_pcm16_samples(&mut samples, -14.0, -1.0);
        assert!(applied_db > 5.0);
        let peak_limit = (10.0_f32.powf(-1.0 / 20.0) * i16::MAX as f32) as i16 + 1;
        assert!(samples.iter().all(|&s| s.abs() <= peak_limit));
    }
}
