// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Factory pane: the visual workflow canvas controller.
//!
//! Follows the same pattern as the other panes in this module:
//! owns its state, every change arrives as a [`FactoryMsg`], and
//! [`FactoryPane::update`] is the only code that mutates it.

use std::collections::HashMap;
use concat_factory::VideoAspect;
use concat_factory::pipeline::{FactoryPipelineConfig, run_pipeline};

use crate::host::{on_ui, spawn};
use crate::panes::Msg;

/// Everything that can happen to the factory pane.
#[derive(Clone, Debug)]
pub enum FactoryMsg {
    /// User clicked a node card to select it.
    NodeSelected { id: String },
    /// User pressed "run" on a single node card.
    TestNode { id: String },
    /// User chose a preset/resource for a node in the double-click modal.
    ApplyNodePreset { id: String, preset: String },
    /// User toggled the 16:9 / 9:16 aspect ratio switch.
    AspectChanged(bool),
    /// User triggered the master "FABRICATE" button.
    Fabricate,
    /// User started the batch production queue.
    BatchStart,
    /// User stopped the batch production queue.
    BatchStop,
    /// User added a topic to the batch production queue.
    BatchAdd(String),
    /// User cleared the batch production queue.
    BatchClear,
    /// Progress or completion of a batch item.
    BatchItemProgress {
        index: usize,
        progress: f32,
        phase: String,
        finished: bool,
        failed: bool,
    },
    /// Progress update from an active pipeline stage.
    StageProgress {
        node_id: String,
        progress: f32,
        phase: String,
    },
    /// End-to-end pipeline run completed.
    PipelineFinished { success: bool, summary: String },
    /// Files dropped onto the Factory canvas from the OS file manager.
    FilesDropped(Vec<std::path::PathBuf>),
    /// User clicked an input or output port on a node card to connect/disconnect wires.
    PortClicked { node_id: String, is_output: bool },
}

impl From<FactoryMsg> for Msg {
    fn from(msg: FactoryMsg) -> Self {
        Msg::Factory(msg)
    }
}

/// One queued item in the batch production list.
#[derive(Clone, Debug)]
pub struct BatchJobItem {
    /// Unique identifier for the queue row.
    pub id: String,
    /// Topic or prompt for this video.
    pub topic: String,
    /// Current phase label.
    pub phase: String,
    /// Progress in `0.0..=1.0`.
    pub progress: f32,
    /// Whether this item has finished.
    pub done: bool,
    /// Whether this item failed.
    pub failed: bool,
}

/// State for the factory workflow canvas.
pub struct FactoryPane {
    /// Whether the pane is currently shown.
    pub open: bool,
    /// ID of the currently selected node.
    pub selected_node_id: Option<String>,
    /// Selected output aspect ratio (`true` = 16:9 YouTube, `false` = 9:16 Shorts).
    pub aspect_16_9: bool,
    /// Whether a single pipeline run is currently active.
    pub fabricating: bool,
    /// Whether the batch queue is running.
    pub batch_running: bool,
    /// Queued batch items.
    pub batch_items: Vec<BatchJobItem>,
    /// Last reported progress per node (`0.0..=1.0`).
    pub node_progress: HashMap<String, f32>,
    /// Last status message.
    pub last_status: String,
    /// Current pipeline configuration.
    pub pipeline_config: FactoryPipelineConfig,
    /// Source node ID awaiting target input port click for interactive wiring.
    pub pending_wire_from: Option<String>,
    /// Active custom `(from, to)` port connections toggled by the user.
    pub custom_wires: Vec<(String, String)>,
}

impl Default for FactoryPane {
    fn default() -> Self {
        Self {
            open: false,
            selected_node_id: None,
            aspect_16_9: true,
            fabricating: false,
            batch_running: false,
            batch_items: Vec::new(),
            node_progress: HashMap::new(),
            last_status: String::new(),
            pipeline_config: FactoryPipelineConfig::default(),
            pending_wire_from: None,
            custom_wires: vec![
                ("script".into(), "voice".into()),
                ("script".into(), "media".into()),
                ("script".into(), "music".into()),
                ("voice".into(), "sfx".into()),
                ("music".into(), "sfx".into()),
                ("voice".into(), "caption".into()),
                ("media".into(), "export".into()),
                ("sfx".into(), "export".into()),
                ("caption".into(), "export".into()),
            ],
        }
    }
}

impl FactoryPane {
    /// Applies one message to the factory pane state.
    pub fn update(&mut self, msg: FactoryMsg) {
        match msg {
            FactoryMsg::NodeSelected { id } => {
                self.selected_node_id = Some(id);
            }
            FactoryMsg::AspectChanged(is_16_9) => {
                self.aspect_16_9 = is_16_9;
                self.pipeline_config.export.aspect = if is_16_9 {
                    VideoAspect::YouTube16x9
                } else {
                    VideoAspect::Vertical9x16
                };
                log::info!(
                    "Factory: aspect set to {}",
                    if is_16_9 { "16:9 YouTube" } else { "9:16 Shorts" }
                );
            }
            FactoryMsg::ApplyNodePreset { id, preset } => {
                match id.as_str() {
                    "script" => {
                        self.pipeline_config.pro.editing_style =
                            concat_factory::presets::EditingStyle::from_str_loose(&preset);
                    }
                    "caption" => {
                        let pal = concat_factory::presets::ColorPalette::from_str_loose(&preset);
                        self.pipeline_config.pro.color_palette = pal;
                        let (h, e, c) = pal.colors();
                        self.pipeline_config.pro.highlight_color = h.to_string();
                        self.pipeline_config.pro.effects_color = e.to_string();
                        self.pipeline_config.pro.caption_color = c.to_string();
                    }
                    "voice" => {
                        use concat_factory::nodes::voice_node::VoiceStyle;
                        self.pipeline_config.voice.design.style =
                            if preset.contains("Clonagem") || preset.contains("Sussurro") {
                                VoiceStyle::WhisperSuspense
                            } else if preset.contains("Épico") {
                                VoiceStyle::EpicDocumentary
                            } else {
                                VoiceStyle::DarkNarrator
                            };
                    }
                    "music" => {
                        use concat_factory::nodes::music_node::MusicGenre;
                        self.pipeline_config.music.genre = if preset.contains("Phonk") {
                            MusicGenre::Phonk
                        } else if preset.contains("Horror") {
                            MusicGenre::HorrorDrone
                        } else if preset.contains("Synthwave") {
                            MusicGenre::SynthwaveNoir
                        } else {
                            MusicGenre::DarkAmbient
                        };
                    }
                    "sfx" => {
                        if let Some((key, val)) = preset.split_once(':') {
                            use concat_factory::presets::*;
                            match key {
                                "motion_rhythm" => {
                                    self.pipeline_config.pro.motion_rhythm = match val {
                                        "suave" => MotionRhythm::Suave,
                                        "agressivo" => MotionRhythm::Agressivo,
                                        _ => MotionRhythm::Medio,
                                    };
                                }
                                "caption_mode" => {
                                    self.pipeline_config.pro.caption_mode = match val {
                                        "frase_curta" => CaptionMode::FraseCurta,
                                        "so_palavras_chave" => CaptionMode::SoPalavrasChave,
                                        "sem_legenda" => CaptionMode::SemLegenda,
                                        _ => CaptionMode::PalavraPorPalavra,
                                    };
                                }
                                "transition_style" => {
                                    self.pipeline_config.pro.transition_style = match val {
                                        "corte_seco" => TransitionStyle::CorteSeco,
                                        "zoom_morph" => TransitionStyle::ZoomMorph,
                                        "glitch_flash" => TransitionStyle::GlitchFlash,
                                        _ => TransitionStyle::WhooshBlur,
                                    };
                                }
                                "color_grade" => {
                                    self.pipeline_config.pro.color_grade = match val {
                                        "dark_cinema" => ColorGradeLut::DarkCinema,
                                        "teal_and_orange" => ColorGradeLut::TealAndOrange,
                                        "crisp_tech" => ColorGradeLut::CrispTech,
                                        _ => ColorGradeLut::Natural,
                                    };
                                }
                                "auto_ducking" => {
                                    self.pipeline_config.pro.auto_ducking = val == "true";
                                }
                                "safe_zones" => {
                                    self.pipeline_config.pro.safe_zones = val == "true";
                                }
                                _ => {}
                            }
                        }
                    }
                    "export" => {
                        if preset.contains('_') {
                            let plat = concat_factory::presets::PlatformPreset::from_str_loose(&preset);
                            self.pipeline_config.pro.apply_platform_preset(plat);
                            self.aspect_16_9 = !plat.is_vertical();
                        } else {
                            self.aspect_16_9 = !preset.contains("9:16");
                        }
                        self.pipeline_config.export.aspect = if self.aspect_16_9 {
                            concat_factory::VideoAspect::YouTube16x9
                        } else {
                            concat_factory::VideoAspect::Vertical9x16
                        };
                    }
                    _ => {}
                }
                self.last_status = format!("Nó '{id}' atualizado: {preset}");
            }
            FactoryMsg::TestNode { id } => {
                if id.is_empty() {
                    return;
                }
                log::info!("Factory: testing node '{id}'");
                self.node_progress.insert(id.clone(), 0.25);
                self.last_status = format!("Testando nó '{id}'...");
                let mut cfg = self.pipeline_config.clone();
                if cfg.script.raw_script.trim().is_empty() {
                    cfg.script.raw_script =
                        "Você não deveria estar vendo isso. O oceano esconde segredos antigos."
                            .to_string();
                }
                let node_id = id.clone();
                spawn(
                    move || -> Result<String, String> {
                        match node_id.as_str() {
                            "script" => {
                                cfg.script.parse_scenes();
                                Ok(format!("Roteiro OK: {} cenas detectadas", cfg.script.scenes.len()))
                            }
                            "media" => {
                                let stock = concat_media::stock::StockClient::new(cfg.pexels_key.clone())
                                    .with_pixabay_key(cfg.pixabay_key.clone());
                                let req = concat_media::stock::StockSearchRequest {
                                    query: "dark ocean mystery".into(),
                                    orientation: if cfg.export.aspect == VideoAspect::YouTube16x9 {
                                        "landscape".into()
                                    } else {
                                        "portrait".into()
                                    },
                                    per_page: 2,
                                };
                                let items = stock.search_videos(&req);
                                Ok(format!("B-Roll OK: {} clipes encontrados", items.len()))
                            }
                            "voice" => {
                                let out = cfg.export.output_dir.join("audio").join("test_voice.wav");
                                let mut tts = concat_speech::edge_tts::EdgeTtsClient::new(cfg.voice.voice_id.clone());
                                tts.rate_percent = cfg.voice.speed_rate as i32;
                                tts.pitch_percent = cfg.voice.pitch_offset as i32;
                                let res = tts
                                    .synthesize_to_file("Teste de narração.", &out)
                                    .map_err(|e| format!("Falha TTS: {e}"))?;
                                Ok(format!("Voz OK: {:.1}s sintetizados", res.duration_seconds))
                            }
                            "music" => {
                                let bgm = cfg.export.output_dir.join("audio").join("test_bgm.wav");
                                cfg.music.duration_seconds = 3.0;
                                cfg.music
                                    .synthesize_ambient_wav(&bgm)
                                    .map_err(|e| format!("Falha BGM: {e}"))?;
                                Ok("Música IA OK: 3.0s BGM gerados".into())
                            }
                            "sfx" => Ok(format!(
                                "SFX OK: auto-ducking {:.1} dB",
                                cfg.sfx.ducking_attenuation_db
                            )),
                            "caption" => {
                                let bursts = concat_project::captions_factory::build_caption_bursts(
                                    &[],
                                    &concat_project::captions_factory::BurstConfig::default(),
                                );
                                Ok(format!("Legendas OK: {} blocos", bursts.len()))
                            }
                            "export" => Ok(format!(
                                "Exportar OK: {} ({})",
                                cfg.export.project_name,
                                if cfg.export.aspect == VideoAspect::YouTube16x9 { "16:9" } else { "9:16" }
                            )),
                            _ => Ok(format!("Nó '{node_id}' pronto")),
                        }
                    },
                    move |state, _app, _models, result| {
                        let (progress, phase) = match result {
                            Ok(msg) => (1.0, msg),
                            Err(err) => (0.0, err),
                        };
                        state.handle(Msg::Factory(FactoryMsg::StageProgress {
                            node_id: id,
                            progress,
                            phase,
                        }));
                    },
                );
            }
            FactoryMsg::Fabricate => {
                if self.fabricating {
                    return;
                }
                self.fabricating = true;
                self.node_progress.clear();
                let mut cfg = self.pipeline_config.clone();
                if cfg.script.raw_script.trim().is_empty() {
                    cfg.script.raw_script =
                        "Você não deveria estar vendo isso. O oceano esconde segredos antigos."
                            .to_string();
                }
                log::info!("Factory: fabricating video — dispatching background pipeline");
                spawn(
                    move || {
                        run_pipeline(&mut cfg, |node_id, progress, phase| {
                            let nid = node_id.to_string();
                            let ph = phase.to_string();
                            on_ui(move |state, _app, _models| {
                                state.handle(Msg::Factory(FactoryMsg::StageProgress {
                                    node_id: nid,
                                    progress,
                                    phase: ph,
                                }));
                            });
                        })
                    },
                    |state, _app, _models, result| {
                        let msg = match result {
                            Ok(out) => FactoryMsg::PipelineFinished {
                                success: true,
                                summary: format!(
                                    "{} cenas montadas ({:.1}s)",
                                    out.assembled.scenes.len(),
                                    out.assembled.total_duration_seconds
                                ),
                            },
                            Err(err) => FactoryMsg::PipelineFinished {
                                success: false,
                                summary: err.to_string(),
                            },
                        };
                        state.handle(Msg::Factory(msg));
                    },
                );
            }
            FactoryMsg::BatchAdd(topic) => {
                let trimmed = topic.trim();
                if trimmed.is_empty() {
                    return;
                }
                let id = format!("job_{}", self.batch_items.len() + 1);
                self.batch_items.push(BatchJobItem {
                    id,
                    topic: trimmed.to_string(),
                    phase: "Na fila".to_string(),
                    progress: 0.0,
                    done: false,
                    failed: false,
                });
                self.last_status = format!("Adicionado ao lote ({} itens)", self.batch_items.len());
            }
            FactoryMsg::BatchClear => {
                if self.batch_running {
                    self.batch_items.retain(|item| !item.done);
                    self.last_status = "Itens concluídos removidos do lote".to_string();
                } else {
                    self.batch_items.clear();
                    self.last_status = "Fila de produção limpa".to_string();
                }
            }
            FactoryMsg::BatchStart => {
                if self.batch_running {
                    return;
                }
                if self.batch_items.iter().all(|item| item.done) {
                    self.last_status = "Fila vazia — digite um tema e clique em +".to_string();
                    return;
                }
                self.batch_running = true;
                self.last_status = "Lote iniciado".to_string();
                log::info!("Factory: batch production started");
                self.dispatch_next_batch_item();
            }
            FactoryMsg::BatchStop => {
                self.batch_running = false;
                self.last_status = "Lote interrompido".to_string();
                log::info!("Factory: batch production stopped");
            }
            FactoryMsg::BatchItemProgress {
                index,
                progress,
                phase,
                finished,
                failed,
            } => {
                if let Some(item) = self.batch_items.get_mut(index) {
                    item.progress = progress;
                    item.phase = phase.clone();
                    if finished {
                        item.done = true;
                        item.failed = failed;
                    }
                }
                self.last_status = phase;
                if finished && self.batch_running {
                    self.dispatch_next_batch_item();
                }
            }
            FactoryMsg::StageProgress {
                node_id,
                progress,
                phase,
            } => {
                self.node_progress.insert(node_id.clone(), progress);
                self.last_status = phase;
            }
            FactoryMsg::PipelineFinished { success, summary } => {
                self.fabricating = false;
                self.last_status = summary.clone();
                if success {
                    log::info!("Factory: pipeline finished — {}", summary);
                } else {
                    log::error!("Factory: pipeline failed — {}", summary);
                }
            }
            FactoryMsg::FilesDropped(paths) => {
                for path in paths {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_ascii_lowercase())
                        .unwrap_or_default();
                    match ext.as_str() {
                        "txt" | "md" => {
                            if let Ok(text) = std::fs::read_to_string(&path)
                                && !text.trim().is_empty()
                            {
                                self.pipeline_config.script.raw_script = text;
                                self.node_progress.insert("script".to_string(), 1.0);
                                self.last_status = format!(
                                    "Roteiro importado: {}",
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("script")
                                );
                            }
                        }
                        "mp3" | "wav" | "ogg" | "flac" | "m4a" => {
                            if self.selected_node_id.as_deref() == Some("voice") {
                                self.pipeline_config.voice.design.reference_audio_path =
                                    Some(path.clone());
                                self.pipeline_config.voice.engine =
                                    concat_factory::nodes::voice_node::VoiceEngine::ChatterboxClone;
                                self.node_progress.insert("voice".to_string(), 1.0);
                                self.last_status = format!(
                                    "Referência Voice Cloning: {}",
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("ref")
                                );
                            } else {
                                self.pipeline_config.sfx.bgm_track = Some(path.clone());
                                self.node_progress.insert("sfx".to_string(), 1.0);
                                self.last_status = format!(
                                    "Áudio/BGM importado: {}",
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("audio")
                                );
                            }
                        }
                        "mp4" | "mov" | "mkv" | "webm" | "png" | "jpg" | "jpeg" => {
                            self.pipeline_config.media.custom_broll_paths.push(path.clone());
                            self.node_progress.insert("media".to_string(), 1.0);
                            self.last_status = format!(
                                "B-Roll adicionado ({} arquivos)",
                                self.pipeline_config.media.custom_broll_paths.len()
                            );
                        }
                        _ => {}
                    }
                }
            }
            FactoryMsg::PortClicked { node_id, is_output } => {
                if is_output {
                    self.pending_wire_from = Some(node_id.clone());
                    self.last_status = format!("Conectando saída de '{node_id}' — clique em uma porta de entrada");
                } else if let Some(from) = self.pending_wire_from.take() {
                    if from != node_id {
                        let pair = (from.clone(), node_id.clone());
                        if let Some(pos) = self.custom_wires.iter().position(|w| w == &pair) {
                            self.custom_wires.remove(pos);
                            self.last_status = format!("Conexão removida: {from} ✕ {node_id}");
                        } else {
                            self.custom_wires.push(pair);
                            self.last_status = format!("Conectado: {from} ➔ {node_id}");
                        }
                    }
                }
            }
        }
    }

    fn dispatch_next_batch_item(&mut self) {
        let Some(index) = self.batch_items.iter().position(|item| !item.done) else {
            self.batch_running = false;
            self.last_status = "Lote concluído".to_string();
            return;
        };

        let topic = self.batch_items[index].topic.clone();
        self.batch_items[index].phase = "Iniciando...".to_string();
        self.batch_items[index].progress = 0.05;

        let mut cfg = self.pipeline_config.clone();
        cfg.script.raw_script = topic.clone();
        cfg.export.project_name = format!("batch_{}", index + 1);

        spawn(
            move || {
                run_pipeline(&mut cfg, |_node_id, progress, phase| {
                    let ph = phase.to_string();
                    on_ui(move |state, _app, _models| {
                        state.handle(Msg::Factory(FactoryMsg::BatchItemProgress {
                            index,
                            progress: progress.clamp(0.05, 0.95),
                            phase: ph,
                            finished: false,
                            failed: false,
                        }));
                    });
                })
            },
            move |state, _app, _models, result| {
                let (progress, phase, failed) = match result {
                    Ok(out) => (
                        1.0,
                        format!("Concluído ({:.1}s)", out.assembled.total_duration_seconds),
                        false,
                    ),
                    Err(err) => (0.0, format!("Erro: {err}"), true),
                };
                state.handle(Msg::Factory(FactoryMsg::BatchItemProgress {
                    index,
                    progress,
                    phase,
                    finished: true,
                    failed,
                }));
            },
        );
    }
}
