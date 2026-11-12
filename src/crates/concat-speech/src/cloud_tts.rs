// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Optional Cloud TTS Providers: OpenAI Audio (`tts-1`, `tts-1-hd`) and ElevenLabs.
//!
//! These are strictly optional second-tier providers; the factory runs 100%
//! free/offline-capable using [`crate::edge_tts::EdgeTtsClient`] by default.

use serde::{Deserialize, Serialize};

/// OpenAI TTS model variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum OpenAiModel {
    #[default]
    Tts1,
    Tts1Hd,
}

impl OpenAiModel {
    /// API model string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tts1 => "tts-1",
            Self::Tts1Hd => "tts-1-hd",
        }
    }
}

/// OpenAI TTS voice options.
pub const OPENAI_VOICES: &[&str] = &["alloy", "echo", "fable", "onyx", "nova", "shimmer"];

/// Request configuration for OpenAI Audio Speech API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiTtsConfig {
    /// Model (`tts-1` or `tts-1-hd`).
    pub model: OpenAiModel,
    /// Voice name (`onyx` is deep/cinematic, great for dark channels).
    pub voice: String,
    /// Speed multiplier (0.25 to 4.0).
    pub speed: f32,
}

impl Default for OpenAiTtsConfig {
    fn default() -> Self {
        Self {
            model: OpenAiModel::Tts1Hd,
            voice: "onyx".to_string(),
            speed: 1.0,
        }
    }
}

impl OpenAiTtsConfig {
    /// Endpoint URL for OpenAI speech synthesis.
    pub const ENDPOINT: &'static str = "https://api.openai.com/v1/audio/speech";

    /// Builds the JSON payload for the OpenAI speech API.
    pub fn build_payload(&self, input_text: &str) -> serde_json::Value {
        serde_json::json!({
            "model": self.model.as_str(),
            "input": input_text,
            "voice": self.voice,
            "speed": self.speed.clamp(0.25, 4.0),
            "response_format": "wav"
        })
    }
}

/// Request configuration for ElevenLabs Text-to-Speech API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevenLabsTtsConfig {
    /// ElevenLabs Voice ID.
    pub voice_id: String,
    /// Model ID (`eleven_multilingual_v2` by default).
    pub model_id: String,
    /// Voice stability (0.0 to 1.0).
    pub stability: f32,
    /// Similarity boost (0.0 to 1.0).
    pub similarity_boost: f32,
}

impl Default for ElevenLabsTtsConfig {
    fn default() -> Self {
        Self {
            voice_id: "pNInz6obpgDQGcFmaJgB".to_string(), // Adam (deep narrator)
            model_id: "eleven_multilingual_v2".to_string(),
            stability: 0.55,
            similarity_boost: 0.75,
        }
    }
}

impl ElevenLabsTtsConfig {
    /// Builds the ElevenLabs TTS endpoint URL for the configured voice.
    pub fn endpoint_url(&self) -> String {
        format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}",
            self.voice_id
        )
    }

    /// Builds the JSON payload for ElevenLabs TTS.
    pub fn build_payload(&self, text: &str) -> serde_json::Value {
        serde_json::json!({
            "text": text,
            "model_id": self.model_id,
            "voice_settings": {
                "stability": self.stability.clamp(0.0, 1.0),
                "similarity_boost": self.similarity_boost.clamp(0.0, 1.0)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_payload_clamps_speed_and_sets_wav() {
        let cfg = OpenAiTtsConfig {
            model: OpenAiModel::Tts1Hd,
            voice: "onyx".to_string(),
            speed: 10.0,
        };
        let payload = cfg.build_payload("Segredo oculto");
        assert_eq!(payload["model"], "tts-1-hd");
        assert_eq!(payload["voice"], "onyx");
        assert_eq!(payload["speed"], 4.0);
        assert_eq!(payload["response_format"], "wav");
    }

    #[test]
    fn elevenlabs_endpoint_and_payload_valid() {
        let cfg = ElevenLabsTtsConfig::default();
        assert!(cfg.endpoint_url().ends_with(&cfg.voice_id));
        let payload = cfg.build_payload("História sombria");
        assert_eq!(payload["model_id"], "eleven_multilingual_v2");
        assert_eq!(payload["text"], "História sombria");
    }
}
