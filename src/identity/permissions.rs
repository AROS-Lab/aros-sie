//! Permission asymmetry enforcement.
//!
//! Tighten = auto-approve, Loosen = human-review, NEVER = always blocked.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::types::PermissionChange;

/// Configuration for permission asymmetry rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    /// Parameters that can NEVER be modified (the NEVER tier).
    pub never_modify: HashSet<String>,
    /// Parameters where increasing the value means loosening (requires review).
    /// For example, "max_retries" — increasing means more permissive.
    pub loosen_on_increase: HashSet<String>,
    /// Parameters where decreasing the value means loosening (requires review).
    /// For example, "min_confidence" — decreasing means less restrictive.
    pub loosen_on_decrease: HashSet<String>,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        let mut never_modify = HashSet::new();
        never_modify.insert("identity.drift_ceiling".to_string());
        never_modify.insert("security.zone_override".to_string());

        let mut loosen_on_increase = HashSet::new();
        loosen_on_increase.insert("budget.max_tokens".to_string());
        loosen_on_increase.insert("retry.max_attempts".to_string());

        let mut loosen_on_decrease = HashSet::new();
        loosen_on_decrease.insert("quality.min_confidence".to_string());
        loosen_on_decrease.insert("safety.min_review_score".to_string());

        Self {
            never_modify,
            loosen_on_increase,
            loosen_on_decrease,
        }
    }
}

/// Classify whether a parameter change is tightening, loosening, or NEVER.
pub fn classify_change(
    config: &PermissionConfig,
    param: &str,
    old_value: &serde_json::Value,
    new_value: &serde_json::Value,
) -> PermissionChange {
    // NEVER tier — always blocked
    if config.never_modify.contains(param) {
        return PermissionChange::Never;
    }

    // Try to extract numeric values for directional comparison
    let (old_num, new_num) = match (as_f64(old_value), as_f64(new_value)) {
        (Some(o), Some(n)) => (o, n),
        _ => {
            // Non-numeric values: any change is treated as loosening (conservative)
            if old_value != new_value {
                return PermissionChange::Loosen;
            }
            return PermissionChange::Tighten; // no actual change
        }
    };

    if config.loosen_on_increase.contains(param) {
        if new_num > old_num {
            PermissionChange::Loosen
        } else {
            PermissionChange::Tighten
        }
    } else if config.loosen_on_decrease.contains(param) {
        if new_num < old_num {
            PermissionChange::Loosen
        } else {
            PermissionChange::Tighten
        }
    } else {
        // Unknown parameter: any change is treated as loosening (conservative default)
        if (new_num - old_num).abs() > f64::EPSILON {
            PermissionChange::Loosen
        } else {
            PermissionChange::Tighten
        }
    }
}

fn as_f64(v: &serde_json::Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_never_tier() {
        let config = PermissionConfig::default();
        let result = classify_change(
            &config,
            "identity.drift_ceiling",
            &serde_json::json!(0.3),
            &serde_json::json!(0.5),
        );
        assert_eq!(result, PermissionChange::Never);
    }

    #[test]
    fn test_loosen_on_increase() {
        let config = PermissionConfig::default();
        let result = classify_change(
            &config,
            "budget.max_tokens",
            &serde_json::json!(1000),
            &serde_json::json!(2000),
        );
        assert_eq!(result, PermissionChange::Loosen);
    }

    #[test]
    fn test_tighten_on_decrease_for_loosen_increase() {
        let config = PermissionConfig::default();
        let result = classify_change(
            &config,
            "budget.max_tokens",
            &serde_json::json!(2000),
            &serde_json::json!(1000),
        );
        assert_eq!(result, PermissionChange::Tighten);
    }

    #[test]
    fn test_loosen_on_decrease() {
        let config = PermissionConfig::default();
        let result = classify_change(
            &config,
            "quality.min_confidence",
            &serde_json::json!(0.8),
            &serde_json::json!(0.5),
        );
        assert_eq!(result, PermissionChange::Loosen);
    }

    #[test]
    fn test_unknown_param_change_is_loosen() {
        let config = PermissionConfig::default();
        let result = classify_change(
            &config,
            "unknown.param",
            &serde_json::json!(1),
            &serde_json::json!(2),
        );
        assert_eq!(result, PermissionChange::Loosen);
    }

    #[test]
    fn test_non_numeric_change_is_loosen() {
        let config = PermissionConfig::default();
        let result = classify_change(
            &config,
            "some.param",
            &serde_json::json!("old"),
            &serde_json::json!("new"),
        );
        assert_eq!(result, PermissionChange::Loosen);
    }
}
