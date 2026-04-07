//! SelfModelRegistry — the concrete implementation of the SelfModel trait.

use std::collections::HashMap;

use crate::error::SieError;
use crate::types::{CapabilityId, Observation, TaskOutcome};

use super::calibration::BetaDistribution;
use super::snapshot::{CapabilitySnapshot, ModelSnapshot};
use super::SelfModel;

/// Registry that tracks Bayesian confidence for each capability.
pub struct SelfModelRegistry {
    capabilities: HashMap<CapabilityId, BetaDistribution>,
}

impl SelfModelRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with the given capabilities (uniform priors).
    pub fn with_capabilities(caps: &[CapabilityId]) -> Self {
        let mut capabilities = HashMap::new();
        for cap in caps {
            capabilities.insert(cap.clone(), BetaDistribution::new());
        }
        Self { capabilities }
    }

    fn get_or_insert(&mut self, capability: &CapabilityId) -> &mut BetaDistribution {
        self.capabilities
            .entry(capability.clone())
            .or_default()
    }
}

impl Default for SelfModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfModel for SelfModelRegistry {
    fn observe(&mut self, observation: &Observation) -> Result<(), SieError> {
        let dist = self.get_or_insert(&observation.capability);
        match observation.outcome {
            TaskOutcome::Success => dist.record_success(),
            TaskOutcome::Failure => dist.record_failure(),
            TaskOutcome::Degraded => {
                // Degraded counts as half-success: increment both slightly
                dist.alpha += 0.5;
                dist.beta += 0.5;
                dist.observation_count += 1;
            }
        }
        Ok(())
    }

    fn confidence(&self, capability: &CapabilityId) -> Option<f64> {
        self.capabilities.get(capability).map(|d| d.mean())
    }

    fn confidence_interval(&self, capability: &CapabilityId, percentile: f64) -> Option<(f64, f64)> {
        // Map common percentiles to number of standard deviations
        let num_std = match percentile {
            p if (p - 0.90).abs() < 0.01 => 1.645,
            p if (p - 0.95).abs() < 0.01 => 1.96,
            p if (p - 0.99).abs() < 0.01 => 2.576,
            p => {
                // Rough approximation for other percentiles
                // Using inverse normal approximation
                if p <= 0.0 || p >= 1.0 {
                    return None;
                }
                // Simple approximation: 1.0 for 68%, scale linearly
                (p * 3.0).min(3.0)
            }
        };
        self.capabilities
            .get(capability)
            .map(|d| d.confidence_interval(num_std))
    }

    fn decay(&mut self, lambda: f64) {
        for dist in self.capabilities.values_mut() {
            dist.decay(lambda);
        }
    }

    fn snapshot(&self) -> ModelSnapshot {
        let capabilities = self
            .capabilities
            .iter()
            .map(|(id, dist)| {
                (
                    id.clone(),
                    CapabilitySnapshot {
                        capability: id.clone(),
                        alpha: dist.alpha,
                        beta: dist.beta,
                        observation_count: dist.observation_count,
                    },
                )
            })
            .collect();
        ModelSnapshot {
            capabilities,
            timestamp: chrono::Utc::now(),
        }
    }

    fn restore(&mut self, snapshot: &ModelSnapshot) -> Result<(), SieError> {
        self.capabilities.clear();
        for (id, cap_snap) in &snapshot.capabilities {
            self.capabilities.insert(
                id.clone(),
                BetaDistribution {
                    alpha: cap_snap.alpha,
                    beta: cap_snap.beta,
                    observation_count: cap_snap.observation_count,
                },
            );
        }
        Ok(())
    }

    fn capabilities(&self) -> Vec<CapabilityId> {
        self.capabilities.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Observation;
    use uuid::Uuid;

    fn make_observation(cap: &str, outcome: TaskOutcome) -> Observation {
        Observation {
            capability: CapabilityId::new(cap),
            outcome,
            task_id: Some(Uuid::new_v4()),
            dag_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_new_capability_starts_uniform() {
        let registry = SelfModelRegistry::with_capabilities(&[CapabilityId::new("code_gen")]);
        let conf = registry.confidence(&CapabilityId::new("code_gen")).unwrap();
        assert!((conf - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_success_increases_confidence() {
        let mut registry = SelfModelRegistry::new();
        let obs = make_observation("reasoning", TaskOutcome::Success);
        registry.observe(&obs).unwrap();
        let conf = registry.confidence(&CapabilityId::new("reasoning")).unwrap();
        assert!(conf > 0.5);
    }

    #[test]
    fn test_failure_decreases_confidence() {
        let mut registry = SelfModelRegistry::new();
        let obs = make_observation("reasoning", TaskOutcome::Failure);
        registry.observe(&obs).unwrap();
        let conf = registry.confidence(&CapabilityId::new("reasoning")).unwrap();
        assert!(conf < 0.5);
    }

    #[test]
    fn test_degraded_preserves_mean() {
        let mut registry = SelfModelRegistry::new();
        let obs = make_observation("code_gen", TaskOutcome::Degraded);
        registry.observe(&obs).unwrap();
        let conf = registry.confidence(&CapabilityId::new("code_gen")).unwrap();
        // Degraded adds 0.5 to both α and β, so mean stays at 0.5
        assert!((conf - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_many_successes_high_confidence() {
        let mut registry = SelfModelRegistry::new();
        for _ in 0..20 {
            registry
                .observe(&make_observation("code_gen", TaskOutcome::Success))
                .unwrap();
        }
        let conf = registry.confidence(&CapabilityId::new("code_gen")).unwrap();
        assert!(conf > 0.9);
    }

    #[test]
    fn test_snapshot_and_restore() {
        let mut registry = SelfModelRegistry::new();
        for _ in 0..5 {
            registry
                .observe(&make_observation("reasoning", TaskOutcome::Success))
                .unwrap();
        }
        let snap = registry.snapshot();
        let conf_before = registry.confidence(&CapabilityId::new("reasoning")).unwrap();

        let mut new_registry = SelfModelRegistry::new();
        new_registry.restore(&snap).unwrap();
        let conf_after = new_registry
            .confidence(&CapabilityId::new("reasoning"))
            .unwrap();
        assert!((conf_before - conf_after).abs() < f64::EPSILON);
    }

    #[test]
    fn test_decay_reduces_certainty() {
        let mut registry = SelfModelRegistry::new();
        for _ in 0..10 {
            registry
                .observe(&make_observation("code_gen", TaskOutcome::Success))
                .unwrap();
        }
        let snap_before = registry.snapshot();
        let alpha_before = snap_before.capabilities[&CapabilityId::new("code_gen")].alpha;

        registry.decay(0.5);

        let snap_after = registry.snapshot();
        let alpha_after = snap_after.capabilities[&CapabilityId::new("code_gen")].alpha;
        assert!(alpha_after < alpha_before);
    }

    #[test]
    fn test_confidence_interval() {
        let mut registry = SelfModelRegistry::new();
        for _ in 0..20 {
            registry
                .observe(&make_observation("code_gen", TaskOutcome::Success))
                .unwrap();
        }
        let (lower, upper) = registry
            .confidence_interval(&CapabilityId::new("code_gen"), 0.95)
            .unwrap();
        assert!(lower >= 0.0);
        assert!(upper <= 1.0);
        assert!(lower < upper);
        // With 20 successes, lower bound should be high
        assert!(lower > 0.7);
    }

    #[test]
    fn test_unknown_capability_returns_none() {
        let registry = SelfModelRegistry::new();
        assert!(registry.confidence(&CapabilityId::new("unknown")).is_none());
    }
}
