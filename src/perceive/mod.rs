//! Perception Engine — aggregates telemetry signals for Loop 0 PERCEIVE step.
//!
//! Collects model adapter spans, task outcomes, and resource metrics into
//! a unified perception state.

pub mod engine;

use serde::{Deserialize, Serialize};

use crate::error::SieError;
use crate::types::Observation;

/// Aggregated perception state from telemetry signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionState {
    /// Recent observations from task executions.
    pub recent_observations: Vec<Observation>,
    /// Current resource utilization (0.0 to 1.0).
    pub resource_utilization: f64,
    /// Number of active tasks.
    pub active_task_count: usize,
    /// Observations grouped by loop origin.
    pub by_loop: std::collections::HashMap<String, Vec<Observation>>,
    /// Timestamp of this perception snapshot.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Trait for the perception component.
pub trait PerceptionSource: Send + Sync {
    /// Ingest a new observation.
    fn ingest(&mut self, observation: Observation);

    /// Produce the current perception state.
    fn perceive(&self) -> Result<PerceptionState, SieError>;

    /// Clear observations older than the given duration.
    fn prune(&mut self, max_age: chrono::Duration);
}
