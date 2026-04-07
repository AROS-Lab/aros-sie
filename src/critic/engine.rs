//! MetaCognitionEngine — evaluates task outcomes and emits typed critic actions.

use std::collections::HashMap;

use crate::error::SieError;
use crate::self_model::SelfModel;
use crate::types::{CapabilityId, MemoryTier, Observation, TaskOutcome};

use super::output::{AlertSeverity, CriticOutput};
use super::rules::CriticConfig;
use super::Critic;

/// The meta-cognition engine that evaluates observations and produces
/// typed action recommendations.
pub struct MetaCognitionEngine<S: SelfModel> {
    config: CriticConfig,
    self_model: S,
    /// Track consecutive failure counts per capability.
    failure_streaks: HashMap<CapabilityId, u32>,
}

impl<S: SelfModel> MetaCognitionEngine<S> {
    /// Create a new engine with the given config and self-model reference.
    pub fn new(config: CriticConfig, self_model: S) -> Self {
        Self {
            config,
            self_model,
            failure_streaks: HashMap::new(),
        }
    }

    /// Get a reference to the underlying self-model.
    pub fn self_model(&self) -> &S {
        &self.self_model
    }

    /// Get a mutable reference to the underlying self-model.
    pub fn self_model_mut(&mut self) -> &mut S {
        &mut self.self_model
    }

    fn evaluate_single(&mut self, observation: &Observation) -> Result<Vec<CriticOutput>, SieError> {
        // First, update the self-model with this observation
        self.self_model.observe(observation)?;

        let mut outputs = Vec::new();
        let cap = &observation.capability;

        match observation.outcome {
            TaskOutcome::Success => {
                // Reset failure streak on success
                self.failure_streaks.remove(cap);
                // Success within expected bounds → NoAction
                outputs.push(CriticOutput::NoAction {
                    reason: format!("Task succeeded for capability '{}'", cap),
                });
            }
            TaskOutcome::Failure => {
                // Increment failure streak
                let streak = self.failure_streaks.entry(cap.clone()).or_insert(0);
                *streak += 1;
                let current_streak = *streak;

                // Always journal failures if configured
                if self.config.always_journal_failures {
                    outputs.push(CriticOutput::MemoryWrite {
                        tier: MemoryTier::L4ErrorJournal,
                        key: format!("failure/{}/{}", cap, observation.timestamp.timestamp()),
                        content: format!(
                            "Failure on capability '{}', streak: {}",
                            cap, current_streak
                        ),
                    });
                }

                // Check if we should emit an Alert (failure streak exceeded)
                if current_streak >= self.config.alert_failure_streak {
                    outputs.push(CriticOutput::Alert {
                        severity: AlertSeverity::Warning,
                        message: format!(
                            "Capability '{}' has failed {} times consecutively",
                            cap, current_streak
                        ),
                        related_task: observation.task_id,
                    });
                }

                // Check confidence level for policy/experiment recommendations
                if let Some(confidence) = self.self_model.confidence(cap) {
                    if confidence < self.config.experiment_confidence_threshold {
                        // Very low confidence → recommend experiment to gather data
                        outputs.push(CriticOutput::Experiment {
                            hypothesis: format!(
                                "Capability '{}' may need different approach (confidence: {:.3})",
                                cap, confidence
                            ),
                            proposed_change: serde_json::json!({
                                "capability": cap.to_string(),
                                "action": "shadow_test_alternative"
                            }),
                            evaluation_metric: "imp@50".to_string(),
                        });
                    } else if confidence < self.config.policy_update_confidence_threshold {
                        // Determine if we have enough data to recommend policy change
                        let snap = self.self_model.snapshot();
                        let obs_count = snap
                            .capabilities
                            .get(cap)
                            .map(|c| c.observation_count)
                            .unwrap_or(0);

                        if obs_count >= self.config.min_observations_for_policy {
                            outputs.push(CriticOutput::PolicyUpdate {
                                parameter: format!("capability.{}.strategy", cap),
                                old_value: serde_json::json!("default"),
                                new_value: serde_json::json!("fallback"),
                                rationale: format!(
                                    "Confidence for '{}' is {:.3} after {} observations",
                                    cap, confidence, obs_count
                                ),
                            });
                        }
                    }
                }
            }
            TaskOutcome::Degraded => {
                // Degraded is a soft signal — write to session memory
                outputs.push(CriticOutput::MemoryWrite {
                    tier: MemoryTier::L2Session,
                    key: format!("degraded/{}/{}", cap, observation.timestamp.timestamp()),
                    content: format!("Degraded performance on capability '{}'", cap),
                });
            }
        }

        Ok(outputs)
    }
}

impl<S: SelfModel> Critic for MetaCognitionEngine<S> {
    fn evaluate(&self, observation: &Observation) -> Result<Vec<CriticOutput>, SieError> {
        self.evaluate_without_update(observation)
    }

    fn critique_batch(&self, observations: &[Observation]) -> Result<Vec<CriticOutput>, SieError> {
        let mut all_outputs = Vec::new();
        for obs in observations {
            all_outputs.extend(self.evaluate(obs)?);
        }
        Ok(all_outputs)
    }
}

impl<S: SelfModel> MetaCognitionEngine<S> {
    /// Evaluate with mutable access (preferred path — updates self-model and streaks).
    pub fn evaluate_mut(&mut self, observation: &Observation) -> Result<Vec<CriticOutput>, SieError> {
        self.evaluate_single(observation)
    }

    /// Evaluate without updating internal state (for the immutable Critic trait).
    fn evaluate_without_update(&self, observation: &Observation) -> Result<Vec<CriticOutput>, SieError> {
        let mut outputs = Vec::new();
        let cap = &observation.capability;

        match observation.outcome {
            TaskOutcome::Success => {
                outputs.push(CriticOutput::NoAction {
                    reason: format!("Task succeeded for capability '{}'", cap),
                });
            }
            TaskOutcome::Failure => {
                let streak = self.failure_streaks.get(cap).copied().unwrap_or(0) + 1;

                if self.config.always_journal_failures {
                    outputs.push(CriticOutput::MemoryWrite {
                        tier: MemoryTier::L4ErrorJournal,
                        key: format!("failure/{}/{}", cap, observation.timestamp.timestamp()),
                        content: format!("Failure on capability '{}', streak: {}", cap, streak),
                    });
                }

                if streak >= self.config.alert_failure_streak {
                    outputs.push(CriticOutput::Alert {
                        severity: AlertSeverity::Warning,
                        message: format!(
                            "Capability '{}' has failed {} times consecutively",
                            cap, streak
                        ),
                        related_task: observation.task_id,
                    });
                }

                if let Some(confidence) = self.self_model.confidence(cap) {
                    if confidence < self.config.experiment_confidence_threshold {
                        outputs.push(CriticOutput::Experiment {
                            hypothesis: format!(
                                "Capability '{}' may need different approach (confidence: {:.3})",
                                cap, confidence
                            ),
                            proposed_change: serde_json::json!({
                                "capability": cap.to_string(),
                                "action": "shadow_test_alternative"
                            }),
                            evaluation_metric: "imp@50".to_string(),
                        });
                    } else if confidence < self.config.policy_update_confidence_threshold {
                        outputs.push(CriticOutput::PolicyUpdate {
                            parameter: format!("capability.{}.strategy", cap),
                            old_value: serde_json::json!("default"),
                            new_value: serde_json::json!("fallback"),
                            rationale: format!(
                                "Low confidence for '{}': {:.3}",
                                cap, confidence
                            ),
                        });
                    }
                }
            }
            TaskOutcome::Degraded => {
                outputs.push(CriticOutput::MemoryWrite {
                    tier: MemoryTier::L2Session,
                    key: format!("degraded/{}/{}", cap, observation.timestamp.timestamp()),
                    content: format!("Degraded performance on capability '{}'", cap),
                });
            }
        }

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_model::registry::SelfModelRegistry;
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
    fn test_success_produces_no_action() {
        let registry = SelfModelRegistry::new();
        let mut engine = MetaCognitionEngine::new(CriticConfig::default(), registry);
        let obs = make_observation("code_gen", TaskOutcome::Success);
        let outputs = engine.evaluate_mut(&obs).unwrap();
        assert_eq!(outputs.len(), 1);
        assert!(matches!(outputs[0], CriticOutput::NoAction { .. }));
    }

    #[test]
    fn test_failure_journals_to_l4() {
        let registry = SelfModelRegistry::new();
        let mut engine = MetaCognitionEngine::new(CriticConfig::default(), registry);
        let obs = make_observation("reasoning", TaskOutcome::Failure);
        let outputs = engine.evaluate_mut(&obs).unwrap();
        assert!(outputs.iter().any(|o| matches!(o, CriticOutput::MemoryWrite { tier: MemoryTier::L4ErrorJournal, .. })));
    }

    #[test]
    fn test_failure_streak_triggers_alert() {
        let registry = SelfModelRegistry::new();
        let config = CriticConfig {
            alert_failure_streak: 2,
            ..Default::default()
        };
        let mut engine = MetaCognitionEngine::new(config, registry);

        // First failure — no alert
        let obs = make_observation("code_gen", TaskOutcome::Failure);
        let outputs1 = engine.evaluate_mut(&obs).unwrap();
        assert!(!outputs1.iter().any(|o| matches!(o, CriticOutput::Alert { .. })));

        // Second failure — alert!
        let obs2 = make_observation("code_gen", TaskOutcome::Failure);
        let outputs2 = engine.evaluate_mut(&obs2).unwrap();
        assert!(outputs2.iter().any(|o| matches!(o, CriticOutput::Alert { .. })));
    }

    #[test]
    fn test_success_resets_failure_streak() {
        let registry = SelfModelRegistry::new();
        let config = CriticConfig {
            alert_failure_streak: 2,
            ..Default::default()
        };
        let mut engine = MetaCognitionEngine::new(config, registry);

        // One failure
        engine.evaluate_mut(&make_observation("code_gen", TaskOutcome::Failure)).unwrap();
        // One success — resets streak
        engine.evaluate_mut(&make_observation("code_gen", TaskOutcome::Success)).unwrap();
        // Another failure — should NOT trigger alert (streak reset)
        let outputs = engine.evaluate_mut(&make_observation("code_gen", TaskOutcome::Failure)).unwrap();
        assert!(!outputs.iter().any(|o| matches!(o, CriticOutput::Alert { .. })));
    }

    #[test]
    fn test_degraded_writes_to_session_memory() {
        let registry = SelfModelRegistry::new();
        let mut engine = MetaCognitionEngine::new(CriticConfig::default(), registry);
        let obs = make_observation("code_gen", TaskOutcome::Degraded);
        let outputs = engine.evaluate_mut(&obs).unwrap();
        assert!(outputs.iter().any(|o| matches!(o, CriticOutput::MemoryWrite { tier: MemoryTier::L2Session, .. })));
    }

    #[test]
    fn test_low_confidence_triggers_experiment() {
        let registry = SelfModelRegistry::new();
        let config = CriticConfig {
            experiment_confidence_threshold: 0.4,
            ..Default::default()
        };
        let mut engine = MetaCognitionEngine::new(config, registry);

        // Many failures to drive confidence low
        for _ in 0..10 {
            engine.evaluate_mut(&make_observation("code_gen", TaskOutcome::Failure)).unwrap();
        }
        let outputs = engine.evaluate_mut(&make_observation("code_gen", TaskOutcome::Failure)).unwrap();
        assert!(outputs.iter().any(|o| matches!(o, CriticOutput::Experiment { .. })));
    }

    #[test]
    fn test_immutable_evaluate() {
        let registry = SelfModelRegistry::new();
        let engine = MetaCognitionEngine::new(CriticConfig::default(), registry);
        let obs = make_observation("code_gen", TaskOutcome::Success);
        let outputs = engine.evaluate(&obs).unwrap();
        assert!(!outputs.is_empty());
    }
}
