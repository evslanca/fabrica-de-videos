// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Local AI Models Manager: Tracks on-device models (Whisper GGML, Chatterbox ONNX,
//! MusicGen ONNX, and local LLMs) and selects the optimal hardware accelerator.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Supported local AI model families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelKind {
    WhisperBase,
    WhisperSmall,
    ChatterboxTurboOnnx,
    MusicGenSmallOnnx,
    OllamaLlama3,
}

/// Hardware execution backend for local inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorBackend {
    Cpu,
    DirectMl,
    CoreMl,
}

/// Installation status of one local model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelStatus {
    /// Model identifier.
    pub kind: LocalModelKind,
    /// Human-readable display name.
    pub display_name: String,
    /// Approximate disk footprint in MB.
    pub size_mb: u32,
    /// Expected path on disk.
    pub local_path: PathBuf,
    /// Whether the model files exist on disk.
    pub installed: bool,
    /// Selected hardware accelerator.
    pub accelerator: AcceleratorBackend,
}

/// Returns the recommended hardware accelerator for the current OS.
pub const fn recommended_accelerator() -> AcceleratorBackend {
    if cfg!(target_os = "macos") {
        AcceleratorBackend::CoreMl
    } else if cfg!(target_os = "windows") {
        AcceleratorBackend::DirectMl
    } else {
        AcceleratorBackend::Cpu
    }
}

/// Scans `models_dir` and returns the catalog of local AI models with install status.
pub fn default_model_catalog(models_dir: &Path) -> Vec<LocalModelStatus> {
    let accel = recommended_accelerator();
    let specs = [
        (LocalModelKind::WhisperBase, "Whisper Base (Transcrição/Dubbing)", 142, "whisper-base.bin"),
        (LocalModelKind::WhisperSmall, "Whisper Small (Alta Precisão)", 466, "whisper-small.bin"),
        (LocalModelKind::ChatterboxTurboOnnx, "Chatterbox Turbo ONNX (Voice Cloning)", 980, "chatterbox-turbo"),
        (LocalModelKind::MusicGenSmallOnnx, "MusicGen Small ONNX (Músicas IA)", 1250, "musicgen-small-onnx"),
        (LocalModelKind::OllamaLlama3, "Ollama Llama 3 (Roteirista Offline)", 4700, "ollama-llama3"),
    ];

    specs
        .into_iter()
        .map(|(kind, name, size_mb, rel)| {
            let local_path = models_dir.join(rel);
            let installed = local_path.exists();
            LocalModelStatus {
                kind,
                display_name: name.to_string(),
                size_mb,
                local_path,
                installed,
                accelerator: accel,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_all_five_local_models() {
        let catalog = default_model_catalog(Path::new("models"));
        assert_eq!(catalog.len(), 5);
        assert!(catalog.iter().any(|m| m.kind == LocalModelKind::ChatterboxTurboOnnx));
        assert!(catalog.iter().any(|m| m.kind == LocalModelKind::MusicGenSmallOnnx));
    }
}
