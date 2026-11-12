// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Batch Production Line: Manages queue of topics, scripts, and autonomous generation.

use std::collections::VecDeque;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::VideoAspect;

/// One item in the factory batch queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    /// Unique item id.
    pub id: String,
    /// Topic or title of the video.
    pub topic: String,
    /// Raw script or prompt for generation.
    pub script: String,
    /// Output format.
    pub aspect: VideoAspect,
    /// Target directory for final bundle (video, thumbnail, metadata).
    pub destination_dir: PathBuf,
    /// Progress percentage (0.0 to 100.0).
    pub progress: f32,
    /// Status description.
    pub current_phase: String,
}

/// Batch production queue manager.
#[derive(Debug, Default)]
pub struct BatchQueue {
    items: VecDeque<BatchItem>,
}

impl BatchQueue {
    /// Creates a new queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes an item to the end of the line.
    pub fn enqueue(&mut self, item: BatchItem) {
        self.items.push_back(item);
    }

    /// Pops the next item for processing.
    pub fn next_item(&mut self) -> Option<BatchItem> {
        self.items.pop_front()
    }

    /// Total count of pending items.
    pub fn pending_count(&self) -> usize {
        self.items.len()
    }
}
