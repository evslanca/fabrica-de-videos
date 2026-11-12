// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Integrated AI Scriptwriter (Offline via Ollama/llama.cpp or optional Cloud API).
//!
//! Turns a single topic/niche into a high-retention dark channel bundle:
//! 3-second Hook, Open Loops, scene breakdown with English Pexels keywords,
//! SEO title/description/tags, and Thumbnail catchphrase.

use serde::{Deserialize, Serialize};
use crate::nodes::script_node::SceneSegment;
use crate::retention::RetentionHook;
use crate::{DarkPreset, VideoAspect};

/// LLM backend for script generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlmBackend {
    /// Local Ollama instance (`http://localhost:11434/api/generate`) — 100% free/offline.
    #[default]
    LocalOllama,
    /// Optional OpenAI GPT-4o API.
    OpenAi,
    /// Optional Google Gemini API.
    Gemini,
}

/// Request parameters for generating a dark-channel script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptwriterRequest {
    /// Core video topic or mystery.
    pub topic: String,
    /// Dark channel niche preset.
    pub preset: DarkPreset,
    /// Target format (16:9 long-form vs 9:16 short).
    pub aspect: VideoAspect,
    /// Desired duration in seconds (e.g. 45s for Shorts, 480s for YouTube).
    pub target_duration_seconds: u32,
    /// Language for narration (e.g. `"pt-BR"`).
    pub language: String,
    /// Preferred LLM backend.
    pub backend: LlmBackend,
}

impl Default for ScriptwriterRequest {
    fn default() -> Self {
        Self {
            topic: "O mistério do Mar do Diabo".to_string(),
            preset: DarkPreset::Mystery,
            aspect: VideoAspect::Vertical9x16,
            target_duration_seconds: 45,
            language: "pt-BR".to_string(),
            backend: LlmBackend::LocalOllama,
        }
    }
}

/// Complete generated script & YouTube packaging bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedScriptBundle {
    /// High-CTR SEO title.
    pub seo_title: String,
    /// YouTube description with keywords.
    pub seo_description: String,
    /// SEO tags.
    pub seo_tags: Vec<String>,
    /// Short 2–4 word catchphrase for the Thumbnail.
    pub thumbnail_catchphrase: String,
    /// 3-second opening hook.
    pub hook: RetentionHook,
    /// Scene-by-scene narration with English B-Roll search keywords.
    pub scenes: Vec<SceneSegment>,
}

/// Builds a structured system/user prompt for an LLM to output JSON matching [`GeneratedScriptBundle`].
pub fn build_dark_script_prompt(req: &ScriptwriterRequest) -> String {
    let format_hint = match req.aspect {
        VideoAspect::YouTube16x9 => "YouTube horizontal 16:9 documentary style",
        VideoAspect::Vertical9x16 => "Fast-paced vertical 9:16 Shorts/Reels (< 60s)",
    };
    format!(
        "Create a high-retention dark channel script in {} for topic '{}' ({:?}, {}). \
         Target duration: {}s. Include a 3-second hook, open loops, scene splits every 3-5s, \
         and English visual_keywords for Pexels stock video search. Return valid JSON.",
        req.language, req.topic, req.preset, format_hint, req.target_duration_seconds
    )
}

/// Parses an LLM JSON response or builds a structured offline fallback bundle
/// so the factory can operate 100% offline even without a running LLM.
pub fn parse_or_fallback_bundle(
    raw_json: &str,
    req: &ScriptwriterRequest,
) -> GeneratedScriptBundle {
    if let Ok(bundle) = serde_json::from_str::<GeneratedScriptBundle>(raw_json) {
        return bundle;
    }

    GeneratedScriptBundle {
        seo_title: format!("{} — O Que Ninguém Te Contou", req.topic),
        seo_description: format!(
            "Descubra os segredos ocultos por trás de {}. Assista até o final!",
            req.topic
        ),
        seo_tags: vec![
            "misterio".to_string(),
            "canal dark".to_string(),
            "curiosidades".to_string(),
        ],
        thumbnail_catchphrase: "NINGUÉM EXPLICOU".to_string(),
        hook: RetentionHook {
            opening_statement: format!("Você não deveria ignorar {}...", req.topic),
            visual_cue: "dark ocean storm lightning".to_string(),
            duration_seconds: 3.0,
        },
        scenes: vec![
            SceneSegment {
                index: 0,
                narration: format!("Você não deveria ignorar {}.", req.topic),
                visual_keywords: vec!["dark storm".into(), "cinematic mystery".into()],
                duration_seconds: 3.0,
                is_pattern_interrupt: true,
            },
            SceneSegment {
                index: 1,
                narration: "Registros antigos apontam detalhes que foram apagados da história."
                    .to_string(),
                visual_keywords: vec!["old archive".into(), "shadow silhouette".into()],
                duration_seconds: 4.0,
                is_pattern_interrupt: false,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_bundle_provides_english_broll_keywords() {
        let req = ScriptwriterRequest::default();
        let bundle = parse_or_fallback_bundle("not valid json", &req);
        assert!(!bundle.scenes.is_empty());
        assert!(bundle.scenes[0].visual_keywords.contains(&"dark storm".to_string()));
        assert!(!bundle.thumbnail_catchphrase.is_empty());
    }
}
