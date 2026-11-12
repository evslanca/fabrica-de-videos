// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Voice Synthesis Node: Dispatches TTS requests to Edge-TTS, OpenAI, or ElevenLabs.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Supported speech providers for the factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEngine {
    /// Microsoft Edge Speech (100% Free, neural, multilingual, word timestamps).
    #[default]
    EdgeTts,
    /// ElevenLabs Turbo / Multilingual v2 (Premium quality, voice cloning).
    ElevenLabs,
    /// OpenAI Audio (tts-1, tts-1-hd).
    OpenAi,
}

/// Voice node configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceNodeConfig {
    /// Selected engine.
    pub engine: VoiceEngine,
    /// Voice identifier (e.g., "pt-BR-AntonioNeural", "pt-BR-FranciscaNeural").
    pub voice_id: String,
    /// Speech rate modifier (-50% to +100%, 0.0 is normal).
    pub speed_rate: f32,
    /// Pitch modifier (-50Hz to +50Hz, 0.0 is normal).
    pub pitch_offset: f32,
    /// Cached path to the generated voice audio file.
    pub output_audio: Option<PathBuf>,
}

impl Default for VoiceNodeConfig {
    fn default() -> Self {
        Self {
            engine: VoiceEngine::EdgeTts,
            voice_id: "pt-BR-AntonioNeural".to_string(),
            speed_rate: 0.0,
            pitch_offset: 0.0,
            output_audio: None,
        }
    }
}
