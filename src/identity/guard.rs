//! IdentityGuard — enforces cumulative drift ceiling and permission asymmetry.

use crate::error::SieError;
use crate::policy::snapshot::PolicySnapshot;
use crate::types::PermissionChange;

use super::distance::drift_distance;
use super::permissions::{classify_change, PermissionConfig};
use super::{IdentityCheckResult, IdentityChecker};

/// Guards against excessive policy drift from the baseline identity.
pub struct IdentityGuard {
    /// The baseline snapshot taken at initialization.
    baseline: PolicySnapshot,
    /// Maximum allowed drift (0.0 = no change, 1.0 = orthogonal, 2.0 = opposite).
    drift_ceiling: f64,
    /// Permission asymmetry configuration.
    permission_config: PermissionConfig,
}

impl IdentityGuard {
    /// Create a new guard with the given baseline and drift ceiling.
    pub fn new(baseline: PolicySnapshot, drift_ceiling: f64, permission_config: PermissionConfig) -> Self {
        Self {
            baseline,
            drift_ceiling,
            permission_config,
        }
    }

    /// Create with default permission config.
    pub fn with_defaults(baseline: PolicySnapshot, drift_ceiling: f64) -> Self {
        Self::new(baseline, drift_ceiling, PermissionConfig::default())
    }
}

impl IdentityChecker for IdentityGuard {
    fn check(&self, proposed: &PolicySnapshot) -> Result<IdentityCheckResult, SieError> {
        let drift = drift_distance(&self.baseline.feature_vector, &proposed.feature_vector);

        let allowed = drift <= self.drift_ceiling;
        let reason = if !allowed {
            Some(format!(
                "Cumulative drift {:.4} exceeds ceiling {:.4}",
                drift, self.drift_ceiling
            ))
        } else {
            None
        };

        Ok(IdentityCheckResult {
            drift,
            ceiling: self.drift_ceiling,
            allowed,
            reason,
        })
    }

    fn classify_permission_change(
        &self,
        current: &PolicySnapshot,
        proposed: &PolicySnapshot,
    ) -> PermissionChange {
        // Check all changed parameters and return the most restrictive classification
        let mut worst = PermissionChange::Tighten;

        for (key, new_val) in &proposed.config {
            if let Some(old_val) = current.config.get(key) {
                if old_val != new_val {
                    let classification = classify_change(&self.permission_config, key, old_val, new_val);
                    match classification {
                        PermissionChange::Never => return PermissionChange::Never,
                        PermissionChange::Loosen => worst = PermissionChange::Loosen,
                        PermissionChange::Tighten => {} // already the default
                    }
                }
            } else {
                // New key added — conservative: treat as loosen
                worst = PermissionChange::Loosen;
            }
        }

        worst
    }

    fn current_drift(&self) -> f64 {
        // Without a "current" snapshot, report 0.0
        // In practice, the kernel tracks the current snapshot separately
        0.0
    }

    fn reset_baseline(&mut self, baseline: PolicySnapshot) {
        self.baseline = baseline;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_snapshot(feature_vector: Vec<f64>, config: HashMap<String, serde_json::Value>) -> PolicySnapshot {
        PolicySnapshot {
            id: Uuid::new_v4(),
            parent_id: None,
            score: 0.0,
            created_at: chrono::Utc::now(),
            config,
            description: "test".to_string(),
            feature_vector,
        }
    }

    #[test]
    fn test_identical_passes() {
        let baseline = make_snapshot(vec![1.0, 0.0, 0.0], HashMap::new());
        let guard = IdentityGuard::with_defaults(baseline.clone(), 0.5);
        let result = guard.check(&baseline).unwrap();
        assert!(result.allowed);
        assert!(result.drift < 1e-10);
    }

    #[test]
    fn test_small_drift_passes() {
        let baseline = make_snapshot(vec![1.0, 0.0, 0.0], HashMap::new());
        let guard = IdentityGuard::with_defaults(baseline, 0.5);
        let proposed = make_snapshot(vec![0.95, 0.05, 0.0], HashMap::new());
        let result = guard.check(&proposed).unwrap();
        assert!(result.allowed);
    }

    #[test]
    fn test_large_drift_blocked() {
        let baseline = make_snapshot(vec![1.0, 0.0], HashMap::new());
        let guard = IdentityGuard::with_defaults(baseline, 0.1);
        let proposed = make_snapshot(vec![0.0, 1.0], HashMap::new()); // orthogonal = drift 1.0
        let result = guard.check(&proposed).unwrap();
        assert!(!result.allowed);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_permission_never_blocks() {
        let mut config = HashMap::new();
        config.insert("identity.drift_ceiling".to_string(), serde_json::json!(0.3));
        let current = make_snapshot(vec![], config.clone());

        let mut new_config = config;
        new_config.insert("identity.drift_ceiling".to_string(), serde_json::json!(0.5));
        let proposed = make_snapshot(vec![], new_config);

        let baseline = make_snapshot(vec![1.0], HashMap::new());
        let guard = IdentityGuard::with_defaults(baseline, 0.5);

        let classification = guard.classify_permission_change(&current, &proposed);
        assert_eq!(classification, PermissionChange::Never);
    }

    #[test]
    fn test_tightening_auto_approved() {
        let mut config = HashMap::new();
        config.insert("budget.max_tokens".to_string(), serde_json::json!(2000));
        let current = make_snapshot(vec![], config);

        let mut new_config = HashMap::new();
        new_config.insert("budget.max_tokens".to_string(), serde_json::json!(1000));
        let proposed = make_snapshot(vec![], new_config);

        let baseline = make_snapshot(vec![1.0], HashMap::new());
        let guard = IdentityGuard::with_defaults(baseline, 0.5);

        let classification = guard.classify_permission_change(&current, &proposed);
        assert_eq!(classification, PermissionChange::Tighten);
    }

    #[test]
    fn test_loosening_requires_review() {
        let mut config = HashMap::new();
        config.insert("budget.max_tokens".to_string(), serde_json::json!(1000));
        let current = make_snapshot(vec![], config);

        let mut new_config = HashMap::new();
        new_config.insert("budget.max_tokens".to_string(), serde_json::json!(5000));
        let proposed = make_snapshot(vec![], new_config);

        let baseline = make_snapshot(vec![1.0], HashMap::new());
        let guard = IdentityGuard::with_defaults(baseline, 0.5);

        let classification = guard.classify_permission_change(&current, &proposed);
        assert_eq!(classification, PermissionChange::Loosen);
    }

    #[test]
    fn test_reset_baseline() {
        let baseline = make_snapshot(vec![1.0, 0.0], HashMap::new());
        let mut guard = IdentityGuard::with_defaults(baseline, 0.1);

        let new_baseline = make_snapshot(vec![0.0, 1.0], HashMap::new());
        guard.reset_baseline(new_baseline.clone());

        // After reset, the proposed (same as new baseline) should pass
        let result = guard.check(&new_baseline).unwrap();
        assert!(result.allowed);
    }
}
