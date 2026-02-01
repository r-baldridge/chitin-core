// crates/chitin-core/src/provenance.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::identity::NodeIdentity;

/// Full provenance chain for a Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// The agent/node that created this Polyp.
    pub creator: NodeIdentity,
    /// Attribution to the original source material.
    pub source: SourceAttribution,
    /// Processing pipeline that produced this Polyp.
    pub pipeline: ProcessingPipeline,
}

/// Attribution to the original source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAttribution {
    /// IPFS CID of the original source document (if available).
    pub source_cid: Option<String>,
    /// URL of the original source (if web-sourced).
    pub source_url: Option<String>,
    /// Human-readable title or description of the source.
    pub title: Option<String>,
    /// License under which the source is available.
    pub license: Option<String>,
    /// Timestamp when the source was accessed.
    pub accessed_at: DateTime<Utc>,
}

/// Describes the processing pipeline that produced the Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPipeline {
    /// Ordered list of processing steps (e.g., "chunk", "clean", "embed", "prove").
    pub steps: Vec<PipelineStep>,
    /// Total wall-clock time for the pipeline (milliseconds).
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub version: String,
    pub params: serde_json::Value,
}
