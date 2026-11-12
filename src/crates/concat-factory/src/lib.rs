// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Industrial Video Factory: automated dark video generation engine for YouTube and Social Media.
//!
//! Provides a reactive DAG (Directed Acyclic Graph) workflow engine where nodes
//! represent production stages (Scripting, Voice Synthesis, Stock Media Curation,
//! Procedural SFX, Synchronized Captions, Anti-Reused Content Effects, and Timeline Assembly).

pub mod batch;
pub mod graph;
pub mod nodes;
pub mod pipeline;
pub mod retention;
pub mod thumbnail;

use serde::{Deserialize, Serialize};

/// Aspect ratio / format of the target video.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VideoAspect {
    /// 16:9 Landscape for standard YouTube videos (1920x1080 or 3840x2160).
    #[default]
    YouTube16x9,
    /// 9:16 Portrait for Shorts, Reels, TikTok (1080x1920).
    Vertical9x16,
}

impl VideoAspect {
    /// Native canvas width for the aspect ratio.
    pub fn width(self) -> u32 {
        match self {
            Self::YouTube16x9 => 1920,
            Self::Vertical9x16 => 1080,
        }
    }

    /// Native canvas height for the aspect ratio.
    pub fn height(self) -> u32 {
        match self {
            Self::YouTube16x9 => 1080,
            Self::Vertical9x16 => 1920,
        }
    }
}

/// Dark channel theme and atmospheric styling preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DarkPreset {
    /// Unsolved mysteries, strange occurrences, atmospheric tension.
    #[default]
    Mystery,
    /// Stoicism, ancient philosophy, deep contrast, dramatic shadows.
    Stoic,
    /// Crime stories, dark psychology, gritty noir tones.
    TrueCrime,
    /// Horror, paranormal, eerie drones and high-contrast visuals.
    Horror,
    /// Space anomalies, scientific mysteries, cosmic ambient vibes.
    CosmicCuriosity,
}

/// Operational state of a node or execution task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// Node is waiting for input or unconfigured.
    #[default]
    Idle,
    /// Node is actively processing work.
    Working,
    /// Task finished successfully.
    Success,
    /// Execution encountered an error.
    Error,
}
