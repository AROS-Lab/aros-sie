//! PerceptionEngine — aggregates telemetry signals into a unified perception state.

use std::collections::HashMap;

use crate::error::SieError;
use crate::types::Observation;

use super::{PerceptionSource, PerceptionState};

/// Concrete perception engine that collects and aggregates observations.
pub struct PerceptionEngine {
    /// All ingested observations.
    observations: Vec<Observation>,
    /// Current resource utilization (updated externally).
    resource_utilization: f64,
    /// Current active task count (updated externally).
    active_task_count: usize,
}

impl PerceptionEngine {
    /// Create a new perception engine.
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
            resource_utilization: 0.0,
            active_task_count: 0,
        }
    }

    /// Update resource utilization (called by kernel).
    pub fn set_resource_utilization(&mut self, utilization: f64) {
        self.resource_utilization = utilization.clamp(0.0, 1.0);
    }

    /// Update active task count (called by kernel).
    pub fn set_active_task_count(&mut self, count: usize) {
        self.active_task_count = count;
    }

    /// Get the number of stored observations.
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }
}

impl Default for PerceptionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PerceptionSource for PerceptionEngine {
    fn ingest(&mut self, observation: Observation) {
        self.observations.push(observation);
    }

    fn perceive(&self) -> Result<PerceptionState, SieError> {
        let mut by_loop: HashMap<String, Vec<Observation>> = HashMap::new();
        for obs in &self.observations {
            let key = format!("{:?}", obs.capability);
            by_loop.entry(key).or_default().push(obs.clone());
        }

        Ok(PerceptionState {
            recent_observations: self.observations.clone(),
            resource_utilization: self.resource_utilization,
            active_task_count: self.active_task_count,
            by_loop,
            timestamp: chrono::Utc::now(),
        })
    }

    fn prune(&mut self, max_age: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_age;
        self.observations.retain(|obs| obs.timestamp > cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CapabilityId, TaskOutcome};
    use uuid::Uuid;

    fn make_observation(cap: &str) -> Observation {
        Observation {
            capability: CapabilityId::new(cap),
            outcome: TaskOutcome::Success,
            task_id: Some(Uuid::new_v4()),
            dag_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_ingest_and_perceive() {
        let mut engine = PerceptionEngine::new();
        engine.ingest(make_observation("code_gen"));
        engine.ingest(make_observation("reasoning"));

        let state = engine.perceive().unwrap();
        assert_eq!(state.recent_observations.len(), 2);
    }

    #[test]
    fn test_resource_utilization() {
        let mut engine = PerceptionEngine::new();
        engine.set_resource_utilization(0.75);
        let state = engine.perceive().unwrap();
        assert!((state.resource_utilization - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_prune() {
        let mut engine = PerceptionEngine::new();
        // Add an old observation
        let mut old_obs = make_observation("old");
        old_obs.timestamp = chrono::Utc::now() - chrono::Duration::hours(2);
        engine.ingest(old_obs);
        // Add a recent observation
        engine.ingest(make_observation("recent"));

        engine.prune(chrono::Duration::hours(1));
        assert_eq!(engine.observation_count(), 1);
    }

    #[test]
    fn test_active_task_count() {
        let mut engine = PerceptionEngine::new();
        engine.set_active_task_count(5);
        let state = engine.perceive().unwrap();
        assert_eq!(state.active_task_count, 5);
    }
}
