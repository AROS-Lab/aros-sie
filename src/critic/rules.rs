//! Configurable critique rules and thresholds.

use serde::{Deserialize, Serialize};

/// Configuration for the meta-cognition engine's decision thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticConfig {
    /// Confidence threshold below which a failure triggers a PolicyUpdate.
    /// If the self-model confidence for the capability is below this after failure,
    /// the critic recommends a policy change.
    pub policy_update_confidence_threshold: f64,

    /// Number of consecutive failures before emitting an Alert.
    pub alert_failure_streak: u32,

    /// Confidence threshold below which the critic recommends an Experiment
    /// to gather more data about the capability.
    pub experiment_confidence_threshold: f64,

    /// Minimum number of observations before the critic will emit PolicyUpdate
    /// (avoids overreacting to sparse data).
    pub min_observations_for_policy: u64,

    /// If true, always write failure observations to the error journal (L4).
    pub always_journal_failures: bool,
}

impl Default for CriticConfig {
    fn default() -> Self {
        Self {
            policy_update_confidence_threshold: 0.4,
            alert_failure_streak: 3,
            experiment_confidence_threshold: 0.3,
            min_observations_for_policy: 5,
            always_journal_failures: true,
        }
    }
}
