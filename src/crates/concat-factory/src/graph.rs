// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Workflow DAG (Directed Acyclic Graph) engine inspired by n8n.
//!
//! Nodes connect via typed inputs and outputs. Nodes can be triggered individually
//! for fast local iteration (e.g. testing just the voice generation) or executed
//! as an end-to-end automated pipeline.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::NodeStatus;

/// Unique identifier for a node in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

/// Port identifier for inputs/outputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub String);

/// Data packet carried through connections between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum PortData {
    /// Text content (prompts, scripts, ideas).
    Text(String),
    /// Audio file path on disk (.wav, .mp3).
    Audio(PathBuf),
    /// Video clip file path on disk (.mp4, .mov).
    Video(PathBuf),
    /// Image file path on disk (.png, .jpg).
    Image(PathBuf),
    /// List of media paths.
    MediaList(Vec<PathBuf>),
    /// Subtitle timestamps and word markers.
    TimedWords(Vec<TimedWord>),
}

/// Sized timestamp marker for a single spoken word.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedWord {
    /// The word itself.
    pub word: String,
    /// Start time in milliseconds.
    pub start_ms: u64,
    /// End time in milliseconds.
    pub end_ms: u64,
}

/// Connection between an output port of a source node and an input port of a target node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// ID of the origin node.
    pub from_node: NodeId,
    /// ID of the origin output port.
    pub from_port: PortId,
    /// ID of the target node.
    pub to_node: NodeId,
    /// ID of the target input port.
    pub to_port: PortId,
}

/// A node in the factory graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryNode {
    /// Node identifier.
    pub id: NodeId,
    /// Human-readable label (e.g., "Edge-TTS Voice Generator").
    pub label: String,
    /// Node type kind (script, voice, media, sfx, captions, export).
    pub kind: String,
    /// Canvas position X.
    pub pos_x: f32,
    /// Canvas position Y.
    pub pos_y: f32,
    /// Current operational status (for LEDs and UI indicators).
    pub status: NodeStatus,
    /// Error message if status is Error.
    pub error: Option<String>,
}

/// Complete workflow graph schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactoryGraph {
    /// All nodes in the canvas.
    pub nodes: HashMap<NodeId, FactoryNode>,
    /// All connections / wires between ports.
    pub connections: Vec<Connection>,
}

impl FactoryGraph {
    /// Creates a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: FactoryNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Connects two ports.
    pub fn connect(
        &mut self,
        from_node: NodeId,
        from_port: PortId,
        to_node: NodeId,
        to_port: PortId,
    ) {
        self.connections.push(Connection {
            from_node,
            from_port,
            to_node,
            to_port,
        });
    }
}
