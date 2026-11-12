// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! End-to-End Factory Pipeline Runner.
//!
//! Executes the Dark Channel production graph (or a single isolated node for
//! rapid previewing), coordinating `concat-speech`, `concat-media`,
//! `concat-effects`, and `concat-project` into a final [`AssembledProject`].

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use concat_effects::dark_cinema::AntiReusedFilterParams;
use concat_media::stock::{StockClient, StockMediaItem, StockSearchRequest};
use concat_project::auto_ducking::DuckingParams;
use concat_project::captions_factory::{BurstConfig, build_caption_bursts};
use concat_project::factory_assembler::{AssembledProject, AssembledScene};
use concat_project::sfx_library::{SfxEvent, assign_scene_sfx};
use concat_speech::edge_tts::{EdgeTtsClient, SynthesisResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::nodes::caption_node::CaptionNodeConfig;
use crate::nodes::export_node::ExportNodeConfig;
use crate::nodes::media_node::MediaNodeConfig;
use crate::nodes::music_node::MusicNodeConfig;
use crate::nodes::script_node::ScriptNodeConfig;
use crate::nodes::sfx_node::SfxNodeConfig;
use crate::nodes::voice_node::VoiceNodeConfig;
use crate::{DarkPreset, VideoAspect};

/// Errors during pipeline execution.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// Script text is empty.
    #[error("script cannot be empty")]
    EmptyScript,
    /// Speech synthesis I/O error.
    #[error("voice synthesis failed: {0}")]
    VoiceIo(#[from] std::io::Error),
    /// Final MP4 export error.
    #[error("mp4 export failed: {0}")]
    ExportFailed(String),
}

/// Complete configuration for a single video factory run.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactoryPipelineConfig {
    /// Target dark channel preset.
    pub preset: DarkPreset,
    /// Script configuration.
    pub script: ScriptNodeConfig,
    /// Voice TTS configuration.
    pub voice: VoiceNodeConfig,
    /// Stock media & anti-reused content configuration.
    pub media: MediaNodeConfig,
    /// SFX & BGM ducking configuration.
    pub sfx: SfxNodeConfig,
    /// Procedural / AI music generation configuration.
    #[serde(default)]
    pub music: MusicNodeConfig,
    /// Dynamic caption configuration.
    pub caption: CaptionNodeConfig,
    /// Export resolution and target folder.
    pub export: ExportNodeConfig,
    /// Optional Pexels key (falls back to Pixabay/local vault when None).
    pub pexels_key: Option<String>,
    /// Optional Pixabay key (falls back to local vault when None).
    pub pixabay_key: Option<String>,
}

/// Output artifacts produced by a pipeline run.
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    /// Assembled timeline ready for GPU rendering.
    pub assembled: AssembledProject,
    /// Voice synthesis result with word boundaries.
    pub voice_result: SynthesisResult,
    /// Placed SFX events on the timeline.
    pub sfx_events: Vec<SfxEvent>,
    /// Anti-reused content filter settings applied to B-Rolls.
    pub anti_reused_filters: AntiReusedFilterParams,
    /// Rendered MP4 path when `cfg.export.render_mp4` is enabled.
    pub exported_mp4: Option<PathBuf>,
}

/// Runs the full dark video production pipeline synchronously on the worker thread.
pub fn run_pipeline(
    cfg: &mut FactoryPipelineConfig,
    mut on_progress: impl FnMut(&str, f32, &str),
) -> Result<PipelineOutput, PipelineError> {
    if cfg.script.raw_script.trim().is_empty() {
        return Err(PipelineError::EmptyScript);
    }

    // Stage 1: Script parsing & retention scene breakdown
    on_progress("script", 0.1, "Segmentando roteiro e hooks...");
    cfg.script.parse_scenes();
    on_progress("script", 1.0, "Roteiro segmentado");

    // Stage 2: Voice synthesis via Edge-TTS
    on_progress("voice", 0.15, "Sintetizando narração neural...");
    let audio_dir = cfg.export.output_dir.join("audio");
    let voice_path = audio_dir.join(format!("{}_voice.wav", cfg.export.project_name));
    let mut tts = EdgeTtsClient::new(cfg.voice.voice_id.clone());
    tts.rate_percent = cfg.voice.speed_rate as i32;
    tts.pitch_percent = cfg.voice.pitch_offset as i32;
    let voice_result = tts.synthesize_to_file(&cfg.script.raw_script, &voice_path)?;
    cfg.voice.output_audio = Some(voice_result.audio_path.clone());
    on_progress("voice", 1.0, "Narração concluída");

    // Stage 3: Stock B-Roll curation
    on_progress("media", 0.2, "Buscando B-Rolls livres de direitos...");
    let orientation = match cfg.export.aspect {
        VideoAspect::YouTube16x9 => "landscape",
        VideoAspect::Vertical9x16 => "portrait",
    };
    let stock = StockClient::new(cfg.pexels_key.clone()).with_pixabay_key(cfg.pixabay_key.clone());
    let mut selected_items: Vec<StockMediaItem> = Vec::with_capacity(cfg.script.scenes.len());

    for seg in &cfg.script.scenes {
        let query = if seg.visual_keywords.is_empty() {
            cfg.media.default_query.clone()
        } else {
            seg.visual_keywords.join(" ")
        };
        let candidates = stock.search_videos(&StockSearchRequest {
            query,
            orientation: orientation.to_string(),
            per_page: 2,
        });
        if let Some(first) = candidates.into_iter().next() {
            selected_items.push(first);
        }
    }

    let stock_cache_dir = cfg.export.output_dir.join("cache").join("stock_media");
    let downloaded_paths = StockClient::download_batch_parallel(&selected_items, &stock_cache_dir);

    let mut assembled_scenes = Vec::with_capacity(cfg.script.scenes.len());
    let mut cursor = 0.0_f32;
    let mut cut_times = Vec::new();

    for (idx, seg) in cfg.script.scenes.iter().enumerate() {
        let clip_path = if !cfg.media.custom_broll_paths.is_empty() {
            cfg.media.custom_broll_paths[idx % cfg.media.custom_broll_paths.len()].clone()
        } else {
            downloaded_paths
                .get(idx)
                .and_then(|res| res.as_ref().ok())
                .cloned()
                .or_else(|| {
                    selected_items
                        .get(idx)
                        .map(|c| PathBuf::from(&c.download_url))
                })
                .unwrap_or_else(|| PathBuf::from("assets/vault/dark_default.mp4"))
        };

        if idx > 0 {
            cut_times.push(cursor);
        }

        assembled_scenes.push(AssembledScene {
            index: idx,
            start_seconds: cursor,
            duration_seconds: seg.duration_seconds,
            video_path: clip_path,
            audio_path: if idx == 0 {
                Some(voice_result.audio_path.clone())
            } else {
                None
            },
            mirror_horizontal: cfg.media.enable_anti_reused_filter && (idx % 2 == 1),
            ken_burns_motion: if idx % 2 == 0 {
                "zoom_in_center".to_string()
            } else {
                "pan_left_to_right".to_string()
            },
        });
        cursor += seg.duration_seconds;
    }
    on_progress("media", 1.0, "B-Rolls selecionados");

    // Stage 4: Procedural SFX & Auto-Ducking
    on_progress("sfx", 0.5, "Injetando SFX e auto-ducking...");
    let sfx_events = if cfg.sfx.auto_transition_whooshes {
        assign_scene_sfx(&cut_times, true)
    } else {
        Vec::new()
    };
    let ducking_params = DuckingParams {
        bgm_base_gain: cfg.sfx.bgm_volume,
        ..Default::default()
    };
    on_progress("sfx", 1.0, "Mixagem de áudio pronta");

    // Stage 5: Hormozi Dynamic Captions
    on_progress("caption", 0.5, "Sincronizando legendas palavra por palavra...");
    let words: Vec<(String, u64, u64)> = voice_result
        .word_boundaries
        .iter()
        .map(|w| {
            (
                w.text.clone(),
                w.audio_offset_ms,
                w.audio_offset_ms + w.duration_ms,
            )
        })
        .collect();
    let caption_bursts = build_caption_bursts(
        &words,
        &BurstConfig {
            max_words: cfg.caption.max_words_per_burst,
            min_hold_ms: 350,
        },
    );
    on_progress("caption", 1.0, "Legendas sincronizadas");

    // Stage 6: Timeline Assembly (auto-synthesize dark BGM if none provided)
    on_progress("export", 0.4, "Montando timeline final...");
    let bgm_path = match cfg.sfx.bgm_track.clone() {
        Some(path) => Some(path),
        None => {
            let mut music_cfg = cfg.music.clone();
            music_cfg.duration_seconds = cursor.max(5.0);
            let bgm_out = audio_dir.join(format!("{}_bgm.wav", cfg.export.project_name));
            music_cfg.synthesize_ambient_wav(&bgm_out).ok()
        }
    };
    let assembled = AssembledProject::assemble(
        assembled_scenes,
        bgm_path,
        caption_bursts,
        cfg.export.aspect.width(),
        cfg.export.aspect.height(),
        &ducking_params,
    );

    let exported_mp4 = if cfg.export.render_mp4 {
        on_progress("export", 0.6, "Renderizando MP4 final...");
        let mp4_path = cfg
            .export
            .output_dir
            .join(format!("{}.mp4", cfg.export.project_name));
        let cancel = AtomicBool::new(false);
        let opts = crate::exporter::FactoryExportOptions {
            output_path: mp4_path,
            ..Default::default()
        };
        let out = crate::exporter::export_assembled_project(&assembled, &opts, &cancel, |f, tot, stage| {
            let ratio = if tot > 0 {
                0.6 + 0.35 * (f as f32 / tot as f32).clamp(0.0, 1.0)
            } else {
                0.75
            };
            on_progress("export", ratio, stage);
        })
        .map_err(PipelineError::ExportFailed)?;
        Some(out)
    } else {
        None
    };

    on_progress("export", 1.0, "Projeto montado com sucesso");

    Ok(PipelineOutput {
        assembled,
        voice_result,
        sfx_events,
        anti_reused_filters: AntiReusedFilterParams::default(),
        exported_mp4,
    })
}

/// Benchmarks timeline assembly + `to_concat_document()` conversion for `scene_count` scenes (~60s at 5s/scene).
pub fn benchmark_pipeline_assembly(scene_count: usize) -> std::time::Duration {
    let start = std::time::Instant::now();
    let scenes: Vec<AssembledScene> = (0..scene_count)
        .map(|i| AssembledScene {
            index: i,
            start_seconds: i as f32 * 5.0,
            duration_seconds: 5.0,
            video_path: PathBuf::from("assets/vault/dark_smoke.mp4"),
            audio_path: if i == 0 {
                Some(PathBuf::from("audio/narration.wav"))
            } else {
                None
            },
            mirror_horizontal: i % 2 == 1,
            ken_burns_motion: "zoom_in_center".to_string(),
        })
        .collect();
    let assembled = AssembledProject::assemble(
        scenes,
        Some(PathBuf::from("audio/bgm.wav")),
        Vec::new(),
        1920,
        1080,
        &DuckingParams::default(),
    );
    let _doc = assembled.to_concat_document();
    start.elapsed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_pipeline_assembles_shorts_project() {
        let tmp_dir = std::env::temp_dir().join("concat_factory_e2e_test");
        let mut cfg = FactoryPipelineConfig::default();
        cfg.script.raw_script =
            "Você não deveria estar vendo isso. O oceano esconde segredos antigos. Descubra agora!"
                .to_string();
        cfg.export.aspect = VideoAspect::Vertical9x16;
        cfg.export.output_dir = tmp_dir.clone();
        cfg.export.project_name = "short_test".to_string();

        let mut stages_seen = Vec::new();
        let output = run_pipeline(&mut cfg, |node, progress, _| {
            if (progress - 1.0).abs() < f32::EPSILON {
                stages_seen.push(node.to_string());
            }
        })
        .expect("pipeline should succeed");

        assert_eq!(output.assembled.width, 1080);
        assert_eq!(output.assembled.height, 1920);
        assert_eq!(output.assembled.scenes.len(), 3);
        assert!(!output.assembled.caption_bursts.is_empty());
        assert!(!output.sfx_events.is_empty());
        assert_eq!(
            stages_seen,
            vec!["script", "voice", "media", "sfx", "caption", "export"]
        );

        let _ = std::fs::remove_dir_all(tmp_dir);
    }

    #[test]
    fn empty_script_returns_error() {
        let mut cfg = FactoryPipelineConfig::default();
        let err = run_pipeline(&mut cfg, |_, _, _| {}).unwrap_err();
        assert!(matches!(err, PipelineError::EmptyScript));
    }
}
