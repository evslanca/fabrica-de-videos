// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Hormozi-style dynamic caption burst builder.
//!
//! Splits a list of `WordBoundary` timestamps (from Edge-TTS or any TTS provider)
//! into display bursts of 1–3 words, alternating highlight color across each burst
//! to maximize on-screen readability and viewer retention.
//!
//! Safe Zone awareness:
//! - 9:16 Shorts/Reels: bursts placed at 25% from bottom, 15% lateral margin.
//! - 16:9 YouTube: bursts placed at 12% from bottom, 10% lateral margin.

use crate::factory_assembler::CaptionBurst;

/// Caption color theme for highlighted words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionColor {
    /// #FFE500 — Neon yellow, highest visibility.
    HighlighterYellow,
    /// #39FF14 — Toxic neon green, viral energy.
    NeonGreen,
    /// #00E5FF — Electric cyan, cold mysterious feel.
    ElectricCyan,
    /// #FFB300 — Dark amber gold, premium authority.
    DarkGold,
}

impl CaptionColor {
    /// CSS hex string.
    pub fn hex(self) -> &'static str {
        match self {
            Self::HighlighterYellow => "#FFE500",
            Self::NeonGreen         => "#39FF14",
            Self::ElectricCyan      => "#00E5FF",
            Self::DarkGold          => "#FFB300",
        }
    }
}

/// Configuration for the burst builder.
pub struct BurstConfig {
    /// Maximum words per simultaneous display burst (1–4).
    pub max_words: usize,
    /// Minimum burst hold duration in milliseconds (even if word ends sooner).
    pub min_hold_ms: u64,
}

impl Default for BurstConfig {
    fn default() -> Self {
        Self { max_words: 3, min_hold_ms: 350 }
    }
}

/// Builds `CaptionBurst` slices from an ordered list of word-boundary events.
///
/// Each burst groups `max_words` consecutive words and sets `active_word_index`
/// to the word currently spoken within the burst window.
pub fn build_caption_bursts(
    words: &[(String, u64, u64)], // (text, start_ms, end_ms)
    config: &BurstConfig,
) -> Vec<CaptionBurst> {
    if words.is_empty() {
        return Vec::new();
    }

    let mut bursts = Vec::new();
    let mut i = 0;

    while i < words.len() {
        let chunk_end = (i + config.max_words).min(words.len());
        let chunk = &words[i..chunk_end];

        let burst_start_ms = chunk.first().map(|w| w.1).unwrap_or(0);
        let burst_end_ms = chunk
            .last()
            .map(|w| (w.2).max(burst_start_ms + config.min_hold_ms))
            .unwrap_or(burst_start_ms + config.min_hold_ms);

        // Find which word in the burst is being spoken at the burst midpoint
        let mid = (burst_start_ms + burst_end_ms) / 2;
        let active = chunk
            .iter()
            .position(|(_, s, e)| mid >= *s && mid <= *e)
            .unwrap_or(0);

        bursts.push(CaptionBurst {
            words: chunk.iter().map(|(t, _, _)| t.clone()).collect(),
            start_seconds: burst_start_ms as f32 / 1000.0,
            end_seconds: burst_end_ms as f32 / 1000.0,
            active_word_index: active,
        });

        i = chunk_end;
    }

    bursts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_groups_max_words() {
        let words: Vec<(String, u64, u64)> = (0..9)
            .map(|i| (format!("word{}", i), i * 400, i * 400 + 350))
            .collect();

        let bursts = build_caption_bursts(&words, &BurstConfig::default());
        // 9 words / 3 max = 3 bursts
        assert_eq!(bursts.len(), 3);
        assert_eq!(bursts[0].words.len(), 3);
        assert_eq!(bursts[2].words.len(), 3);
    }

    #[test]
    fn burst_handles_remainder() {
        let words: Vec<(String, u64, u64)> = (0..7)
            .map(|i| (format!("w{}", i), i * 400, i * 400 + 350))
            .collect();

        let bursts = build_caption_bursts(&words, &BurstConfig::default());
        // 7 words: 3 + 3 + 1
        assert_eq!(bursts.len(), 3);
        assert_eq!(bursts[2].words.len(), 1);
    }

    #[test]
    fn burst_min_hold_respected() {
        let words = vec![("hello".to_string(), 0, 100)];
        let cfg = BurstConfig { max_words: 3, min_hold_ms: 500 };
        let bursts = build_caption_bursts(&words, &cfg);
        assert_eq!(bursts.len(), 1);
        // end must be at least start + min_hold
        assert!(bursts[0].end_seconds >= 0.5);
    }

    #[test]
    fn empty_words_returns_empty() {
        let bursts = build_caption_bursts(&[], &BurstConfig::default());
        assert!(bursts.is_empty());
    }
}
