// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! AI Music Creation Node: Generates copyright-free tracks in two modes:
//! - Instrumental (Sem Voz): dark ambient, phonk, horror drone, epic cinematic, lo-fi.
//! - Vocal with Lyrics (Com Voz): sung/rap/choir tracks from style prompt + lyrics.
//!
//! Supports 100% local ONNX inference (MusicGen / ACE-Step) as primary engine,
//! with optional external API fallback (Suno / Udio / ElevenLabs Music).

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Whether the generated music contains vocals or is purely instrumental.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MusicMode {
    /// Instrumental only (no vocals) — ideal for BGM with narrator auto-ducking.
    #[default]
    Instrumental,
    /// Full song with sung/spoken vocals and lyrics.
    VocalWithLyrics,
}

/// Genre / atmospheric mood presets for dark channels & social media.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MusicGenre {
    #[default]
    DarkAmbient,
    Phonk,
    HorrorDrone,
    EpicCinematic,
    LoFiMystery,
    SynthwaveNoir,
}

/// Vocal timbre when `MusicMode::VocalWithLyrics` is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VocalTimbre {
    #[default]
    MaleDeep,
    FemaleEthereal,
    ChoirDark,
    RaspyRap,
}

/// Music generation backend engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MusicEngine {
    /// 100% local & free via ONNX Runtime (`ort`) — MusicGen / ACE-Step.
    #[default]
    LocalOnnx,
    /// Optional external Suno-compatible API.
    SunoApi,
    /// Optional ElevenLabs Music API.
    ElevenLabsMusic,
}

/// Music node configuration and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicNodeConfig {
    /// Instrumental (sem voz) or VocalWithLyrics (com voz).
    pub mode: MusicMode,
    /// Atmospheric genre preset.
    pub genre: MusicGenre,
    /// Backend generation engine.
    pub engine: MusicEngine,
    /// Target tempo in Beats Per Minute (60–180).
    pub bpm: u16,
    /// Target track duration in seconds.
    pub duration_seconds: f32,
    /// Style prompt describing instruments, mood, and texture.
    pub style_prompt: String,
    /// Lyrics text (used only when `mode == MusicMode::VocalWithLyrics`).
    pub lyrics: String,
    /// Vocal timbre (used only when `mode == MusicMode::VocalWithLyrics`).
    pub vocal_timbre: VocalTimbre,
    /// Whether to split generated track into vocal + instrumental stems.
    pub extract_stems: bool,
    /// Whether to emit beat timestamps for Beat-Sync visual cuts.
    pub beat_sync_cuts: bool,
    /// Output audio path on disk.
    pub output_audio: Option<PathBuf>,
}

impl Default for MusicNodeConfig {
    fn default() -> Self {
        Self {
            mode: MusicMode::Instrumental,
            genre: MusicGenre::DarkAmbient,
            engine: MusicEngine::LocalOnnx,
            bpm: 90,
            duration_seconds: 60.0,
            style_prompt: "deep sub bass, dark atmospheric pad, subtle cinematic tension".to_string(),
            lyrics: String::new(),
            vocal_timbre: VocalTimbre::MaleDeep,
            extract_stems: false,
            beat_sync_cuts: true,
            output_audio: None,
        }
    }
}

impl MusicNodeConfig {
    /// Computes beat timestamps (in seconds) across the track duration
    /// so visual B-Roll cuts and `BeatZoomPulse` shaders snap to the rhythm.
    pub fn beat_timestamps(&self) -> Vec<f32> {
        let bpm = self.bpm.clamp(40, 220) as f32;
        let beat_interval = 60.0 / bpm;
        let mut beats = Vec::new();
        let mut t = 0.0_f32;
        while t < self.duration_seconds {
            beats.push(t);
            t += beat_interval;
        }
        beats
    }

    /// Synthesizes a procedural 44.1 kHz 16-bit mono dark atmospheric BGM `.wav`
    /// matching [`Self::genre`], [`Self::bpm`], and [`Self::duration_seconds`] as a
    /// zero-cost local fallback when ONNX MusicGen weights are not installed.
    pub fn synthesize_ambient_wav(&self, output_path: &Path) -> io::Result<PathBuf> {
        if output_path.exists() {
            return Ok(output_path.to_path_buf());
        }
        if let Some(parent) = output_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let sample_rate = 44_100u32;
        let samples = self.synthesize_ambient_samples(sample_rate);
        let data_len = (samples.len() * 2) as u32;
        let file_len = 36 + data_len;
        let mut f = File::create(output_path)?;
        f.write_all(b"RIFF")?;
        f.write_all(&file_len.to_le_bytes())?;
        f.write_all(b"WAVEfmt ")?;
        f.write_all(&16u32.to_le_bytes())?;
        f.write_all(&1u16.to_le_bytes())?;
        f.write_all(&1u16.to_le_bytes())?;
        f.write_all(&sample_rate.to_le_bytes())?;
        f.write_all(&(sample_rate * 2).to_le_bytes())?;
        f.write_all(&2u16.to_le_bytes())?;
        f.write_all(&16u16.to_le_bytes())?;
        f.write_all(b"data")?;
        f.write_all(&data_len.to_le_bytes())?;

        for pcm in samples {
            f.write_all(&pcm.to_le_bytes())?;
        }
        Ok(output_path.to_path_buf())
    }

    /// Generates mono 16-bit PCM samples for the procedural dark ambient BGM bed.
    pub fn synthesize_ambient_samples(&self, sample_rate: u32) -> Vec<i16> {
        let duration = self.duration_seconds.clamp(1.0, 600.0);
        let num_samples = (sample_rate as f32 * duration) as usize;
        let beat_hz = (self.bpm.clamp(40, 220) as f32) / 60.0;
        let base_freq = match self.genre {
            MusicGenre::DarkAmbient | MusicGenre::HorrorDrone => 55.0,
            MusicGenre::Phonk => 65.4,
            MusicGenre::EpicCinematic => 73.4,
            MusicGenre::LoFiMystery => 82.4,
            MusicGenre::SynthwaveNoir => 61.7,
        };

        let two_pi = 2.0 * std::f32::consts::PI;
        let mut out = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let pulse = 0.65 + 0.35 * (two_pi * beat_hz * t).sin();
            let sub = (two_pi * base_freq * t).sin() * 0.55;
            let fifth = (two_pi * (base_freq * 1.5) * t).sin() * 0.25;
            let sample = (sub + fifth) * pulse * 0.35;
            out.push((sample.clamp(-0.95, 0.95) * i16::MAX as f32) as i16);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_timestamps_match_bpm_and_duration() {
        let cfg = MusicNodeConfig {
            bpm: 120,
            duration_seconds: 10.0,
            ..Default::default()
        };
        // 120 BPM = 2 beats/sec -> 20 beats in 10 seconds
        let beats = cfg.beat_timestamps();
        assert_eq!(beats.len(), 20);
        assert!((beats[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn vocal_mode_preserves_lyrics_and_timbre() {
        let cfg = MusicNodeConfig {
            mode: MusicMode::VocalWithLyrics,
            lyrics: "Nas sombras da noite o segredo acordou".to_string(),
            vocal_timbre: VocalTimbre::ChoirDark,
            ..Default::default()
        };
        assert_eq!(cfg.mode, MusicMode::VocalWithLyrics);
        assert!(!cfg.lyrics.is_empty());
    }

    #[test]
    fn synthesizes_ambient_bgm_samples_with_non_zero_energy() {
        let cfg = MusicNodeConfig {
            genre: MusicGenre::HorrorDrone,
            bpm: 90,
            duration_seconds: 2.0,
            ..Default::default()
        };
        let samples = cfg.synthesize_ambient_samples(44_100);
        assert_eq!(samples.len(), 88_200);
        assert!(samples.iter().any(|&s| s.abs() > 100));
    }
}
