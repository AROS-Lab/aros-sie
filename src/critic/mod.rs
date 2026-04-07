//! Meta-Cognition Engine (Critic) — evaluates task outcomes and emits typed actions.
//!
//! Six output types: PolicyUpdate, MemoryWrite, ToolAction, Alert, NoAction, Experiment.
//! Feeds the Loop 0 CRITIQUE step.

pub mod engine;
pub mod output;
pub mod rules;

use crate::error::SieError;
use crate::types::Observation;
use output::CriticOutput;

/// Trait for the meta-cognition critic component.
///
/// Implementations evaluate task outcomes against the self-model's expectations
/// and produce typed action recommendations.
pub trait Critic: Send + Sync {
    /// Evaluate a task observation and produce a critic output.
    fn evaluate(&self, observation: &Observation) -> Result<Vec<CriticOutput>, SieError>;

    /// Critique a batch of observations (e.g., end-of-DAG summary).
    fn critique_batch(&self, observations: &[Observation]) -> Result<Vec<CriticOutput>, SieError>;
}
