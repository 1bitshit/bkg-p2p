//! Run spine types — shared across crew, flow, and agent task runs.
//!
//! This module defines the unified run model for v0.5:
//!   `RunRecord` — the top-level row (kind, status, spec fingerprint, timestamps)
//!   `RunEvent`  — append-only event log entry for a single run

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The kind of run being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    #[default]
    AgentTask,
    Crew,
    Flow,
}

impl std::fmt::Display for RunKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentTask => write!(f, "agent_task"),
            Self::Crew => write!(f, "crew"),
            Self::Flow => write!(f, "flow"),
        }
    }
}

/// High-level run status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// A single append-only event in a run's event stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEvent {
    /// Monotonically increasing sequence number (0-based within a run).
    pub seq: u64,
    /// ISO-8601 timestamp (UTC).
    pub ts: DateTime<Utc>,
    /// Event kind discriminator.
    #[serde(rename = "type")]
    pub kind: RunEventKind,
    /// Human-readable description.
    pub message: String,
    /// Optional structured payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// Discriminator for `RunEvent.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunEventKind {
    #[default]
    Queued,
    Started,
    PassStarted,
    PassCompleted,
    ToolStarted,
    ToolCompleted,
    InferenceCompleted,
    Error,
    Cancelled,
    CheckpointSaved,
    Log,
}

/// Top-level run record stored in the `runs` redb table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    /// Unique run identifier (UUID v4 string).
    pub run_id: String,
    /// Kind of run.
    pub kind: RunKind,
    /// Current status.
    pub status: RunStatus,
    /// User-supplied name (crew name, flow name, or agent task description).
    pub name: String,
    /// Hash of canonical spec JSON (for contract-lock / tamper detection).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_fingerprint: Option<String>,
    /// Redacted inputs summary (no secrets).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs_summary: Option<serde_json::Value>,
    /// ISO-8601 creation timestamp.
    pub created_at: DateTime<Utc>,
    /// ISO-8601 last-updated timestamp.
    pub updated_at: DateTime<Utc>,
    /// ISO-8601 completion timestamp (set when terminal status reached).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Final output (present for `Completed` runs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// Error message (present for `Failed` runs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RunRecord {
    pub fn new(run_id: impl Into<String>, kind: RunKind, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            run_id: run_id.into(),
            kind,
            status: RunStatus::Pending,
            name: name.into(),
            spec_fingerprint: None,
            inputs_summary: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            output: None,
            error: None,
        }
    }
}
