// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Video Dubbing & Multi-Language Channel Scaling (`1 Video -> 5 Languages`).
//!
//! Coordinates vocal/instrumental stem isolation, speech transcription,
//! segment translation, and time-stretched TTS re-synthesis while preserving
//! all original B-Roll cuts, SFX cues, and background music.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Supported dubbing target languages with default Edge-TTS neural voices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetLanguage {
    PortugueseBr,
    EnglishUs,
    Spanish,
    French,
    German,
    Italian,
}

impl TargetLanguage {
    /// ISO language code.
    pub fn code(self) -> &'static str {
        match self {
            Self::PortugueseBr => "pt-BR",
            Self::EnglishUs => "en-US",
            Self::Spanish => "es-ES",
            Self::French => "fr-FR",
            Self::German => "de-DE",
            Self::Italian => "it-IT",
        }
    }

    /// Recommended free neural voice for dark narration in this language.
    pub fn default_dark_voice(self) -> &'static str {
        match self {
            Self::PortugueseBr => "pt-BR-AntonioNeural",
            Self::EnglishUs => "en-US-ChristopherNeural",
            Self::Spanish => "es-ES-AlvaroNeural",
            Self::French => "fr-FR-HenriNeural",
            Self::German => "de-DE-ConradNeural",
            Self::Italian => "it-IT-DiegoNeural",
        }
    }
}

/// A single dubbed speech segment aligned to the original video timecodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingSegment {
    /// Segment sequence index.
    pub index: usize,
    /// Original start time in milliseconds.
    pub start_ms: u64,
    /// Original end time in milliseconds.
    pub end_ms: u64,
    /// Transcribed original text.
    pub original_text: String,
    /// Translated text in the target language.
    pub translated_text: String,
    /// Computed TTS rate percentage adjustment (`-40..=+60`) to fit window.
    pub required_rate_percent: i32,
}

/// Video dubbing job configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingJobConfig {
    /// Source video or project audio path.
    pub source_video: PathBuf,
    /// Target languages to export in parallel.
    pub target_languages: Vec<TargetLanguage>,
    /// Whether to isolate vocals and preserve original BGM + SFX stems.
    pub preserve_bgm_stems: bool,
    /// Whether to generate translated Hormozi captions.
    pub generate_translated_captions: bool,
}

impl Default for DubbingJobConfig {
    fn default() -> Self {
        Self {
            source_video: PathBuf::from("input.mp4"),
            target_languages: vec![TargetLanguage::EnglishUs, TargetLanguage::Spanish],
            preserve_bgm_stems: true,
            generate_translated_captions: true,
        }
    }
}

/// Computes the TTS speech rate percentage (`-40..=+60`) needed so that
/// `estimated_tts_ms` fits inside `original_ms`.
pub fn compute_time_stretch_rate(original_ms: u64, estimated_tts_ms: u64) -> i32 {
    if original_ms == 0 || estimated_tts_ms == 0 {
        return 0;
    }
    let ratio = estimated_tts_ms as f32 / original_ms as f32;
    let delta_pct = ((ratio - 1.0) * 100.0).round() as i32;
    delta_pct.clamp(-40, 60)
}

/// Expands a single [`crate::pipeline::FactoryPipelineConfig`] into a multi-language
/// batch (`1 Video -> N Languages`), assigning each language's default neural voice
/// and a language-suffixed `project_name`.
pub fn build_multilang_batch_jobs(
    base_cfg: &crate::pipeline::FactoryPipelineConfig,
    languages: &[TargetLanguage],
) -> Vec<(TargetLanguage, crate::pipeline::FactoryPipelineConfig)> {
    languages
        .iter()
        .map(|&lang| {
            let mut cfg = base_cfg.clone();
            cfg.voice.voice_id = lang.default_dark_voice().to_string();
            cfg.export.project_name = format!(
                "{}_{}",
                base_cfg.export.project_name,
                lang.code().to_lowercase()
            );
            (lang, cfg)
        })
        .collect()
}

/// Execution plan for automated video dubbing (`1 Video -> N Languages`)
/// coordinating vocal/instrumental isolation, Whisper transcription, and TTS alignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbingPipelinePlan {
    /// Original source video/audio path.
    pub source_media: PathBuf,
    /// Target dubbing language.
    pub target_language: TargetLanguage,
    /// Separated vocal stem path (for Whisper transcription & voice cloning reference).
    pub vocals_stem_path: PathBuf,
    /// Separated instrumental/BGM stem path (preserved in final mix).
    pub instrumental_stem_path: PathBuf,
    /// Output dubbed speech audio path.
    pub dubbed_voice_path: PathBuf,
}

/// Builds a [`DubbingPipelinePlan`] under `work_dir` for the given source media and language.
pub fn prepare_dubbing_plan(
    source_media: &std::path::Path,
    target_language: TargetLanguage,
    work_dir: &std::path::Path,
) -> DubbingPipelinePlan {
    let lang_dir = work_dir.join(target_language.code().to_lowercase());
    DubbingPipelinePlan {
        source_media: source_media.to_path_buf(),
        target_language,
        vocals_stem_path: work_dir.join("stems").join("vocals.wav"),
        instrumental_stem_path: work_dir.join("stems").join("instrumental.wav"),
        dubbed_voice_path: lang_dir.join("dubbed_voice.wav"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_stretch_rate_clamps_within_natural_bounds() {
        // Dubbed text 25% longer -> speed up +25%
        assert_eq!(compute_time_stretch_rate(4000, 5000), 25);
        // Dubbed text much shorter -> slow down clamped at -40%
        assert_eq!(compute_time_stretch_rate(10000, 2000), -40);
        // Dubbed text way longer -> speed up clamped at +60%
        assert_eq!(compute_time_stretch_rate(2000, 8000), 60);
    }

    #[test]
    fn target_languages_map_to_neural_voices() {
        assert_eq!(TargetLanguage::EnglishUs.default_dark_voice(), "en-US-ChristopherNeural");
        assert_eq!(TargetLanguage::Spanish.code(), "es-ES");
    }

    #[test]
    fn builds_multilang_batch_jobs_for_scaling() {
        let base = crate::pipeline::FactoryPipelineConfig::default();
        let jobs = build_multilang_batch_jobs(
            &base,
            &[TargetLanguage::EnglishUs, TargetLanguage::Spanish, TargetLanguage::French],
        );
        assert_eq!(jobs.len(), 3);
        assert_eq!(jobs[0].1.voice.voice_id, "en-US-ChristopherNeural");
        assert!(jobs[0].1.export.project_name.ends_with("_en-us"));
        assert_eq!(jobs[2].1.voice.voice_id, "fr-FR-HenriNeural");
    }
}
