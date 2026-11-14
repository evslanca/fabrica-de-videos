//! Advanced AI enhancements inspired by top video editors (OpusClip, Descript, Submagic, CapCut).

use serde::{Deserialize, Serialize};

/// Configuration for advanced AI post-production features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEnhancementsConfig {
    /// Analyze segments and compute a 0..=100 Viral Score for clipping.
    pub viral_score_clipping: bool,
    /// Track active speaker face when reframing 16:9 to 9:16.
    pub auto_reframe_speaker: bool,
    /// Enable transcript-based editing & automatic filler-word removal.
    pub text_based_editing: bool,
    /// Apply AI gaze / eye-contact correction toward the camera lens.
    pub eye_contact_correction: bool,
    /// Insert contextual B-Roll & emojis synced to spoken keywords.
    pub contextual_broll_emojis: bool,
    /// Apply AI Studio Sound (denoise + vocal EQ + clarity).
    pub ai_studio_sound: bool,
}

impl Default for AiEnhancementsConfig {
    fn default() -> Self {
        Self {
            viral_score_clipping: true,
            auto_reframe_speaker: true,
            text_based_editing: true,
            eye_contact_correction: false,
            contextual_broll_emojis: true,
            ai_studio_sound: true,
        }
    }
}

/// Viral potential score for a script or video segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViralSegmentScore {
    pub score: u8,
    pub hook_strength: u8,
    pub pacing_score: u8,
    pub reason: String,
}

/// Computes a 0..=100 Viral Score from script text and target duration.
pub fn score_segment(text: &str, duration_s: f32) -> ViralSegmentScore {
    let lower = text.to_lowercase();
    let mut hook: u8 = 55;
    if lower.contains('?') || lower.contains('!') {
        hook = hook.saturating_add(15);
    }
    if lower.chars().any(|c| c.is_ascii_digit()) {
        hook = hook.saturating_add(10);
    }
    for kw in ["segredo", "erro", "nunca", "como", "verdade", "absurdo", "secret", "mistake"] {
        if lower.contains(kw) {
            hook = hook.saturating_add(10);
            break;
        }
    }
    let pacing: u8 = if (15.0..=60.0).contains(&duration_s) { 90 } else { 72 };
    let score = ((hook as u16 + pacing as u16) / 2).min(100) as u8;
    ViralSegmentScore {
        score,
        hook_strength: hook.min(100),
        pacing_score: pacing,
        reason: format!("Hook {}/100 · Ritmo {}/100", hook.min(100), pacing),
    }
}

/// Removes common PT-BR and EN filler words from a transcript.
pub fn remove_filler_words(transcript: &str) -> String {
    let fillers = ["ééé", "éé", "tipo assim", "né,", "né ", "uh,", "um,", "uh ", "um "];
    let mut out = transcript.to_string();
    for f in fillers {
        out = out.replace(f, " ");
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Contextual B-Roll & emoji cue extracted from spoken keywords.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordCue {
    pub keyword: String,
    pub emoji: String,
    pub broll_query: String,
}

/// Extracts contextual emoji & B-Roll search cues from narration text.
pub fn extract_keyword_cues(transcript: &str) -> Vec<KeywordCue> {
    let lower = transcript.to_lowercase();
    let map = [
        ("dinheiro", "💰", "money cash finance"),
        ("lucro", "📈", "stock market chart growth"),
        ("tempo", "⏳", "hourglass clock time"),
        ("cérebro", "🧠", "brain neural network"),
        ("fogo", "🔥", "fire flames dark"),
        ("alerta", "🚨", "warning red light"),
        ("vídeo", "🎬", "cinema camera studio"),
        ("áudio", "🎙️", "microphone studio recording"),
        ("ia", "🤖", "futuristic robot ai"),
        ("segredo", "🤫", "mystery shadow dark"),
        ("erro", "❌", "glitch error screen"),
        ("sucesso", "🏆", "trophy celebration"),
    ];
    map.iter()
        .filter(|(k, _, _)| lower.contains(*k))
        .map(|(k, e, q)| KeywordCue {
            keyword: (*k).to_string(),
            emoji: (*e).to_string(),
            broll_query: (*q).to_string(),
        })
        .collect()
}
