//! Factory History, Operational Feedback/Confirmations, Diagnostics, and
//! Automatic YouTube Thumbnail Scene Selector ("Print no Vídeo").

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Status of a Factory robot/agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunState {
    #[default]
    Livre,
    Ocupado,
    Pausado,
    Erro,
}

/// Actionable confirmation or error feedback for the Factory UI & MCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FactoryFeedback {
    ConfirmOverwrite { path: String, message: String },
    ConfirmRename { old_name: String, new_name: String, message: String },
    ConfirmStopAgent { agent_name: String, message: String },
    FileAlreadyExists { path: String, message: String },
    AudioDefective { reason: String },
    DiskFull { path: String, message: String },
    ClonedFile { source: String, cloned: String },
}

/// Checks if a target file already exists before saving.
pub fn check_file_before_save(path: &Path) -> Option<FactoryFeedback> {
    if path.exists() {
        Some(FactoryFeedback::ConfirmOverwrite {
            path: path.display().to_string(),
            message: format!(
                "O arquivo '{}' já existe. Tem certeza que deseja salvar em cima?",
                path.display()
            ),
        })
    } else {
        None
    }
}

/// Clones a file safely, appending `_copia` if needed.
pub fn clone_file_safe(source: &Path) -> Result<FactoryFeedback, FactoryFeedback> {
    let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("arquivo");
    let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("mp4");
    let target = source.with_file_name(format!("{stem}_copia.{ext}"));
    std::fs::copy(source, &target).map_err(|e| {
        if e.to_string().to_lowercase().contains("space") || e.raw_os_error() == Some(112) {
            FactoryFeedback::DiskFull {
                path: target.display().to_string(),
                message: "Não é possível salvar porque o disco está cheio.".into(),
            }
        } else {
            FactoryFeedback::AudioDefective {
                reason: format!("Falha ao clonar arquivo: {e}"),
            }
        }
    })?;
    Ok(FactoryFeedback::ClonedFile {
        source: source.display().to_string(),
        cloned: target.display().to_string(),
    })
}

/// Single entry in the Factory execution history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryHistoryEntry {
    pub project_name: String,
    pub success: bool,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
    pub timestamp_utc: String,
}

/// Persistent history of Factory runs (successes and errors).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactoryHistory {
    pub entries: Vec<FactoryHistoryEntry>,
}

impl FactoryHistory {
    pub fn push_success(&mut self, project_name: &str, output_path: &str, timestamp_utc: &str) {
        self.entries.push(FactoryHistoryEntry {
            project_name: project_name.to_string(),
            success: true,
            output_path: Some(output_path.to_string()),
            error_message: None,
            timestamp_utc: timestamp_utc.to_string(),
        });
    }

    pub fn push_error(&mut self, project_name: &str, error: &str, timestamp_utc: &str) {
        self.entries.push(FactoryHistoryEntry {
            project_name: project_name.to_string(),
            success: false,
            output_path: None,
            error_message: Some(error.to_string()),
            timestamp_utc: timestamp_utc.to_string(),
        });
    }
}

/// Best scene candidate selected automatically for a YouTube Thumbnail ("Print no Vídeo").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailCandidate {
    pub scene_index: usize,
    pub timestamp_seconds: f64,
    pub headline: String,
    pub ctr_score: u8,
}

/// Reads the video scenes and automatically selects the highest-impact moment for a YouTube Thumbnail.
pub fn select_best_thumbnail_scene(scenes: &[&str], total_duration_s: f64) -> ThumbnailCandidate {
    if scenes.is_empty() {
        return ThumbnailCandidate {
            scene_index: 0,
            timestamp_seconds: (total_duration_s * 0.25).max(1.0),
            headline: "DESCUBRA O SEGREDO".into(),
            ctr_score: 75,
        };
    }
    let mut best_idx = 0usize;
    let mut best_score = 0u8;
    for (i, text) in scenes.iter().enumerate() {
        let lower = text.to_lowercase();
        let mut s = 60u8;
        if lower.contains('!') || lower.contains('?') {
            s = s.saturating_add(15);
        }
        if lower.chars().any(|c| c.is_ascii_digit()) {
            s = s.saturating_add(15);
        }
        if s > best_score {
            best_score = s;
            best_idx = i;
        }
    }
    let ts = ((best_idx as f64 + 0.5) / scenes.len() as f64) * total_duration_s.max(1.0);
    let headline = scenes[best_idx]
        .split_whitespace()
        .take(5)
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase();
    ThumbnailCandidate {
        scene_index: best_idx,
        timestamp_seconds: ts,
        headline,
        ctr_score: best_score.min(99),
    }
}
