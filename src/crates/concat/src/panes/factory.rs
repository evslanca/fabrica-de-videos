// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Factory pane: the visual workflow canvas controller.
//!
//! Follows the same pattern as the other panes in this module:
//! owns its state, every change arrives as a [`FactoryMsg`], and
//! [`FactoryPane::update`] is the only code that mutates it.

use crate::panes::Msg;

/// Everything that can happen to the factory pane.
#[derive(Clone, Debug)]
pub enum FactoryMsg {
    /// User opened the factory pane from the dock or menu.
    Open,
    /// User closed the factory pane.
    Close,
    /// User clicked a node card to select it.
    NodeSelected { id: String, kind: String },
    /// User pressed "run" on a single node card.
    TestNode { id: String },
    /// User triggered the master "FABRICATE" button.
    Fabricate,
    /// User started the batch production queue.
    BatchStart,
    /// User stopped the batch production queue.
    BatchStop,
    /// A batch item finished (success or failure).
    BatchItemFinished { id: String, success: bool },
    /// Progress update from an active pipeline stage.
    StageProgress { node_id: String, progress: f32, phase: String },
}

impl From<FactoryMsg> for Msg {
    fn from(msg: FactoryMsg) -> Self {
        Msg::Factory(msg)
    }
}

/// State for the factory workflow canvas.
#[derive(Default)]
pub struct FactoryPane {
    /// Whether the pane is currently shown.
    pub open: bool,
    /// ID of the currently selected node.
    pub selected_node_id: Option<String>,
    /// Whether the batch queue is running.
    pub batch_running: bool,
}

impl FactoryPane {
    pub fn update(&mut self, msg: FactoryMsg) {
        match msg {
            FactoryMsg::Open => {
                self.open = true;
            }
            FactoryMsg::Close => {
                self.open = false;
                self.selected_node_id = None;
            }
            FactoryMsg::NodeSelected { id, .. } => {
                self.selected_node_id = Some(id);
            }
            FactoryMsg::TestNode { id } => {
                log::info!("Factory: testing node '{}'", id);
            }
            FactoryMsg::Fabricate => {
                log::info!("Factory: fabricating video — dispatching full pipeline");
            }
            FactoryMsg::BatchStart => {
                self.batch_running = true;
                log::info!("Factory: batch production started");
            }
            FactoryMsg::BatchStop => {
                self.batch_running = false;
                log::info!("Factory: batch production stopped");
            }
            FactoryMsg::BatchItemFinished { id, success } => {
                log::info!(
                    "Factory: batch item '{}' finished — {}",
                    id,
                    if success { "✓ success" } else { "✗ failed" }
                );
            }
            FactoryMsg::StageProgress { node_id, progress, phase } => {
                log::debug!(
                    "Factory: node '{}' — {:.0}% ({})",
                    node_id,
                    progress * 100.0,
                    phase
                );
            }
        }
    }
}
