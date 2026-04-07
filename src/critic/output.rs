//! Typed critic outputs — the six action types from meta-cognition.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::MemoryTier;

/// Typed output from the meta-cognition engine.
///
/// Each variant carries data specific to the action type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriticOutput {
    /// Recommend a policy parameter change.
    PolicyUpdate {
        parameter: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
        rationale: String,
    },

    /// Write something to a memory tier.
    MemoryWrite {
        tier: MemoryTier,
        key: String,
        content: String,
    },

    /// Trigger a tool action (e.g., re-run a failing test).
    ToolAction {
        tool_name: String,
        args: serde_json::Value,
        rationale: String,
    },

    /// Alert requiring human attention.
    Alert {
        severity: AlertSeverity,
        message: String,
        related_task: Option<Uuid>,
    },

    /// No action needed — observation is within expected bounds.
    NoAction {
        reason: String,
    },

    /// Propose an experiment (A/B test, shadow test, etc.).
    Experiment {
        hypothesis: String,
        proposed_change: serde_json::Value,
        evaluation_metric: String,
    },
}

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}
