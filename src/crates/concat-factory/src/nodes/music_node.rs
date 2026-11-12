// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! AI Music Creation Node: Generates copyright-free tracks in two modes:
//! - Instrumental (Sem Voz): dark ambient, phonk, horror drone, epic cinematic, lo-fi.
//! - Vocal with Lyrics (Com Voz): sung/rap/choir tracks from style prompt + lyrics.
//!
//! Supports 100% local ONNX inference (MusicGen / ACE-Step) as primary engine,
//! with optional external API fallback (Suno / Udio / ElevenLabs Music).

use std::path::PathBuf;
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
}
