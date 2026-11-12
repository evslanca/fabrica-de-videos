// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Workflow DAG (Directed Acyclic Graph) engine inspired by n8n.
//!
//! Nodes connect via typed inputs and outputs. Nodes can be triggered individually
//! for fast local iteration (e.g. testing just the voice generation) or executed
//! as an end-to-end automated pipeline in topological order.

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::NodeStatus;

/// Errors raised during graph validation, serialization, or execution.
#[derive(Debug, Error)]
pub enum GraphError {
    /// A cycle was detected in the workflow connections.
    #[error("cycle detected in workflow graph involving node '{0}'")]
    CycleDetected(String),
    /// Referenced node does not exist.
    #[error("node '{0}' not found in graph")]
    NodeNotFound(String),
    /// I/O failure when saving or loading a `.darkflow` file.
    #[error("workflow I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization/deserialization failure.
    #[error("workflow JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

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

/// Complete workflow graph schema (`.darkflow`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactoryGraph {
    /// Schema version for forward compatibility.
    #[serde(default = "default_version")]
    pub version: u32,
    /// All nodes in the canvas.
    pub nodes: HashMap<NodeId, FactoryNode>,
    /// All connections / wires between ports.
    pub connections: Vec<Connection>,
}

fn default_version() -> u32 {
    1
}

impl FactoryGraph {
    /// Creates a new empty graph.
    pub fn new() -> Self {
        Self {
            version: 1,
            nodes: HashMap::new(),
            connections: Vec::new(),
        }
    }

    /// Creates the default 6-node Dark Channel pipeline graph.
    pub fn default_dark_pipeline() -> Self {
        let mut g = Self::new();
        let specs = [
            ("script", "Roteiro", "script", 30.0, 80.0),
            ("voice", "Voz / TTS", "voice", 290.0, 40.0),
            ("media", "B-Roll", "media", 290.0, 160.0),
            ("sfx", "SFX & Áudio", "sfx", 550.0, 80.0),
            ("caption", "Legendas", "caption", 550.0, 200.0),
            ("export", "Exportar", "export", 810.0, 120.0),
        ];
        for (id, label, kind, x, y) in specs {
            g.add_node(FactoryNode {
                id: NodeId(id.to_string()),
                label: label.to_string(),
                kind: kind.to_string(),
                pos_x: x,
                pos_y: y,
                status: NodeStatus::Idle,
                error: None,
            });
        }
        let edges = [
            ("script", "narration", "voice", "text"),
            ("script", "keywords", "media", "query"),
            ("voice", "audio", "sfx", "voice_track"),
            ("voice", "boundaries", "caption", "words"),
            ("media", "clips", "export", "video_track"),
            ("sfx", "mix", "export", "audio_track"),
            ("caption", "bursts", "export", "overlay_track"),
        ];
        for (from_n, from_p, to_n, to_p) in edges {
            g.connect(
                NodeId(from_n.to_string()),
                PortId(from_p.to_string()),
                NodeId(to_n.to_string()),
                PortId(to_p.to_string()),
            );
        }
        g
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

    /// Computes a deterministic topological execution order using Kahn's algorithm.
    /// Returns [`GraphError::CycleDetected`] if the graph contains a cycle.
    pub fn execution_order(&self) -> Result<Vec<NodeId>, GraphError> {
        let mut in_degree: HashMap<&NodeId, usize> =
            self.nodes.keys().map(|id| (id, 0)).collect();
        let mut adj: HashMap<&NodeId, Vec<&NodeId>> = HashMap::new();

        for conn in &self.connections {
            if !self.nodes.contains_key(&conn.from_node) {
                return Err(GraphError::NodeNotFound(conn.from_node.0.clone()));
            }
            if !self.nodes.contains_key(&conn.to_node) {
                return Err(GraphError::NodeNotFound(conn.to_node.0.clone()));
            }
            adj.entry(&conn.from_node).or_default().push(&conn.to_node);
            *in_degree.entry(&conn.to_node).or_insert(0) += 1;
        }

        // Sort initial zero-degree nodes for deterministic ordering
        let mut zero_roots: Vec<&NodeId> = in_degree
            .iter()
            .filter_map(|(&id, &deg)| if deg == 0 { Some(id) } else { None })
            .collect();
        zero_roots.sort_by(|a, b| a.0.cmp(&b.0));

        let mut queue: VecDeque<&NodeId> = zero_roots.into_iter().collect();
        let mut order = Vec::with_capacity(self.nodes.len());

        while let Some(u) = queue.pop_front() {
            order.push(u.clone());
            if let Some(neighbors) = adj.get(u) {
                let mut next_ready = Vec::new();
                for &v in neighbors {
                    if let Some(deg) = in_degree.get_mut(v) {
                        *deg -= 1;
                        if *deg == 0 {
                            next_ready.push(v);
                        }
                    }
                }
                next_ready.sort_by(|a, b| a.0.cmp(&b.0));
                for v in next_ready {
                    queue.push_back(v);
                }
            }
        }

        if order.len() != self.nodes.len() {
            let cycle_node = in_degree
                .into_iter()
                .find(|(_, deg)| *deg > 0)
                .map(|(id, _)| id.0.clone())
                .unwrap_or_default();
            return Err(GraphError::CycleDetected(cycle_node));
        }

        Ok(order)
    }

    /// Saves the workflow graph to a `.darkflow` JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), GraphError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Loads a workflow graph from a `.darkflow` JSON file.
    pub fn load_from_file(path: &Path) -> Result<Self, GraphError> {
        let raw = fs::read_to_string(path)?;
        let graph: Self = serde_json::from_str(&raw)?;
        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pipeline_topological_order() {
        let graph = FactoryGraph::default_dark_pipeline();
        let order = graph.execution_order().expect("DAG should be acyclic");
        assert_eq!(order.len(), 6);

        let pos = |name: &str| order.iter().position(|n| n.0 == name).unwrap();
        assert!(pos("script") < pos("voice"));
        assert!(pos("script") < pos("media"));
        assert!(pos("voice") < pos("sfx"));
        assert!(pos("voice") < pos("caption"));
        assert!(pos("sfx") < pos("export"));
        assert!(pos("caption") < pos("export"));
        assert!(pos("media") < pos("export"));
    }

    #[test]
    fn detects_cycle() {
        let mut graph = FactoryGraph::default_dark_pipeline();
        graph.connect(
            NodeId("export".to_string()),
            PortId("out".to_string()),
            NodeId("script".to_string()),
            PortId("in".to_string()),
        );
        let err = graph.execution_order().unwrap_err();
        assert!(matches!(err, GraphError::CycleDetected(_)));
    }

    #[test]
    fn darkflow_roundtrip_serialization() {
        let graph = FactoryGraph::default_dark_pipeline();
        let tmp = std::env::temp_dir().join("test_pipeline.darkflow");
        graph.save_to_file(&tmp).unwrap();
        let loaded = FactoryGraph::load_from_file(&tmp).unwrap();
        assert_eq!(loaded.nodes.len(), 6);
        assert_eq!(loaded.connections.len(), 7);
        let _ = fs::remove_file(tmp);
    }
}
