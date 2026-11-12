// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Voice Synthesis Node: Dispatches TTS requests to Edge-TTS, local Chatterbox
//! Voice Cloning (ONNX), OpenAI, or ElevenLabs, with Voice Design and
//! Multi-Narrator dialogue parsing.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Supported speech providers for the factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEngine {
    /// Microsoft Edge Speech (100% Free, neural, multilingual, word timestamps).
    #[default]
    EdgeTts,
    /// Local Chatterbox Turbo via ONNX Runtime — clones any 5–15s reference voice offline.
    ChatterboxClone,
    /// ElevenLabs Turbo / Multilingual v2 (Premium quality, voice cloning).
    ElevenLabs,
    /// OpenAI Audio (tts-1, tts-1-hd).
    OpenAi,
}

/// Voice Design expressive style preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceStyle {
    #[default]
    DarkNarrator,
    WhisperSuspense,
    EpicDocumentary,
    UrgentNews,
    Neutral,
}

/// Parametric Voice Design configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDesignParams {
    /// Expressive delivery style.
    pub style: VoiceStyle,
    /// Optional reference audio clip (5–15s WAV/MP3) for local Chatterbox voice cloning.
    pub reference_audio_path: Option<PathBuf>,
    /// Expressiveness / warmth (0.0 to 1.0).
    pub warmth: f32,
    /// Voice stability (0.0 to 1.0).
    pub stability: f32,
}

impl Default for VoiceDesignParams {
    fn default() -> Self {
        Self {
            style: VoiceStyle::DarkNarrator,
            reference_audio_path: None,
            warmth: 0.70,
            stability: 0.65,
        }
    }
}

/// A parsed line in a multi-narrator script (`[Speaker]: text`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogueLine {
    /// Speaker label (e.g. `"Narrador"`, `"Testemunha"`).
    pub speaker: String,
    /// Clean text to speak (inline tags stripped).
    pub text: String,
    /// Leading pause in seconds requested via `[pausa Xs]`.
    pub pause_before_seconds: f32,
}

/// Parses a script that may contain `[Speaker]: text` prefixes and `[pausa Xs]` tags.
pub fn parse_multi_speaker_script(raw: &str) -> Vec<DialogueLine> {
    let mut lines = Vec::new();
    for raw_line in raw.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (speaker, rest) = if let Some(after_bracket) = trimmed.strip_prefix('[') {
            if let Some((spk, body)) = after_bracket.split_once("]:") {
                (spk.trim().to_string(), body.trim())
            } else {
                ("Narrador".to_string(), trimmed)
            }
        } else {
            ("Narrador".to_string(), trimmed)
        };

        let mut pause = 0.0_f32;
        let mut clean = rest.to_string();
        if let Some(start) = clean.find("[pausa ") {
            if let Some(end_rel) = clean[start..].find(']') {
                let tag = &clean[start + 7..start + end_rel];
                let num_str = tag.trim().trim_end_matches('s');
                if let Ok(val) = num_str.parse::<f32>() {
                    pause = val.clamp(0.0, 10.0);
                }
                clean.replace_range(start..start + end_rel + 1, "");
            }
        }

        lines.push(DialogueLine {
            speaker,
            text: clean.trim().to_string(),
            pause_before_seconds: pause,
        });
    }
    lines
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
    /// Voice Design & cloning parameters.
    pub design: VoiceDesignParams,
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
            design: VoiceDesignParams::default(),
            output_audio: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multi_speaker_and_pause_tags() {
        let script = "[Narrador]: Tudo começou à meia-noite.\n[Testemunha]: [pausa 0.8s] Eu ouvi passos lá fora.";
        let parsed = parse_multi_speaker_script(script);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].speaker, "Narrador");
        assert_eq!(parsed[0].pause_before_seconds, 0.0);
        assert_eq!(parsed[1].speaker, "Testemunha");
        assert!((parsed[1].pause_before_seconds - 0.8).abs() < 1e-5);
        assert_eq!(parsed[1].text, "Eu ouvi passos lá fora.");
    }
}
