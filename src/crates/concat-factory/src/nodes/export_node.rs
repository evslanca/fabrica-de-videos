// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Final Assembly and Export Node: Orchestrates rendering into YouTube 16:9 or Shorts 9:16.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::VideoAspect;

/// Output export configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportNodeConfig {
    /// Desired format aspect ratio.
    pub aspect: VideoAspect,
    /// Target frame rate (30 or 60 fps).
    pub frame_rate: u32,
    /// Video bit rate in kbps (e.g. 12000 for 1080p high quality).
    pub video_bitrate_kbps: u32,
    /// Destination directory for exports.
    pub output_dir: PathBuf,
    /// Custom project name / slug.
    pub project_name: String,
}

impl Default for ExportNodeConfig {
    fn default() -> Self {
        Self {
            aspect: VideoAspect::YouTube16x9,
            frame_rate: 30,
            video_bitrate_kbps: 12000,
            output_dir: PathBuf::from("export"),
            project_name: "dark_video_01".to_string(),
        }
    }
}
