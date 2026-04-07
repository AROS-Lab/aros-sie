//! SIE event types for event sourcing / audit trail.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{CapabilityId, PermissionChange, TaskOutcome};

/// Events emitted by SIE operations for the audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SieEvent {
    /// Self-model was updated with a new observation.
    SelfModelUpdated {
        capability: CapabilityId,
        outcome: TaskOutcome,
        new_confidence: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Critic produced an output.
    CriticOutput {
        output_type: String,
        summary: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A new policy snapshot was committed.
    PolicyCommitted {
        snapshot_id: Uuid,
        parent_id: Option<Uuid>,
        score: f64,
        description: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A policy branch was forked.
    PolicyForked {
        from_id: Uuid,
        new_id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Identity check was performed.
    IdentityChecked {
        drift: f64,
        ceiling: f64,
        allowed: bool,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Permission change was classified.
    PermissionClassified {
        change_type: PermissionChange,
        parameters_affected: Vec<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Shadow test was run.
    ShadowTestRun {
        baseline_score: f64,
        candidate_score: f64,
        imp_at_k: f64,
        k: usize,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Self-model decay was applied.
    DecayApplied {
        lambda: f64,
        capabilities_affected: usize,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}
