//! Self-Model Registry — Bayesian calibration of agent capabilities.
//!
//! Tracks capability confidence using Beta distributions, updated
//! on each task observation. Feeds the Loop 0 SELF-MODEL UPDATE step.

pub mod calibration;
pub mod registry;
pub mod snapshot;

use crate::error::SieError;
use crate::types::{CapabilityId, Observation};
use snapshot::ModelSnapshot;

/// Trait for the self-model component.
///
/// Implementations maintain a probabilistic model of the agent's capabilities,
/// updating beliefs based on observed task outcomes.
pub trait SelfModel: Send + Sync {
    /// Record an observation and update the capability model.
    fn observe(&mut self, observation: &Observation) -> Result<(), SieError>;

    /// Get the current confidence (mean of Beta distribution) for a capability.
    fn confidence(&self, capability: &CapabilityId) -> Option<f64>;

    /// Get the confidence interval (lower, upper) at the given percentile.
    fn confidence_interval(&self, capability: &CapabilityId, percentile: f64) -> Option<(f64, f64)>;

    /// Apply temporal decay to all capability models, weighting recent observations.
    fn decay(&mut self, lambda: f64);

    /// Take a serializable snapshot of the current model state.
    fn snapshot(&self) -> ModelSnapshot;

    /// Restore model state from a snapshot.
    fn restore(&mut self, snapshot: &ModelSnapshot) -> Result<(), SieError>;

    /// List all tracked capabilities.
    fn capabilities(&self) -> Vec<CapabilityId>;
}
