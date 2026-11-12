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
    /// User opened the factory pane from the dock or menu.
    Open,
    /// User closed the factory pane.
    Close,
    /// User clicked a node card to select it.
    NodeSelected { id: String, kind: String },
    /// User pressed "run" on a single node card.
    TestNode { id: String },
    /// User toggled the 16:9 / 9:16 aspect ratio switch.
    AspectChanged(bool),
    /// User triggered the master "FABRICATE" button.
    Fabricate,
    /// User started the batch production queue.
    BatchStart,
    /// User stopped the batch production queue.
    BatchStop,
    /// A batch item finished (success or failure).
    BatchItemFinished { id: String, success: bool },
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
}

impl From<FactoryMsg> for Msg {
    fn from(msg: FactoryMsg) -> Self {
        Msg::Factory(msg)
    }
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
    /// Last reported progress per node (`0.0..=1.0`).
    pub node_progress: HashMap<String, f32>,
    /// Last status message.
    pub last_status: String,
    /// Current pipeline configuration.
    pub pipeline_config: FactoryPipelineConfig,
}

impl Default for FactoryPane {
    fn default() -> Self {
        Self {
            open: false,
            selected_node_id: None,
            aspect_16_9: true,
            fabricating: false,
            batch_running: false,
            node_progress: HashMap::new(),
            last_status: String::new(),
            pipeline_config: FactoryPipelineConfig::default(),
        }
    }
}

impl FactoryPane {
    /// Applies one message to the factory pane state.
    pub fn update(&mut self, msg: FactoryMsg) {
        match msg {
            FactoryMsg::Open => {
                self.open = true;
            }
            FactoryMsg::Close => {
                self.open = false;
                self.selected_node_id = None;
            }
            FactoryMsg::NodeSelected { id, .. } => {
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
            FactoryMsg::TestNode { id } => {
                log::info!("Factory: testing node '{}'", id);
                self.node_progress.insert(id.clone(), 0.25);
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
            FactoryMsg::BatchStart => {
                self.batch_running = true;
                log::info!("Factory: batch production started");
            }
            FactoryMsg::BatchStop => {
                self.batch_running = false;
                log::info!("Factory: batch production stopped");
            }
            FactoryMsg::BatchItemFinished { id, success } => {
                log::info!(
                    "Factory: batch item '{}' finished — {}",
                    id,
                    if success { "✓ success" } else { "✗ failed" }
                );
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
        }
    }
}
