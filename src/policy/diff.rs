//! Policy diff — comparison utilities between policy snapshots.

use std::collections::{HashMap, HashSet};

use super::snapshot::PolicySnapshot;

/// A diff between two policy snapshots.
#[derive(Debug, Clone)]
pub struct PolicyDiff {
    /// Keys that were added (present in `new` but not `old`).
    pub added: HashMap<String, serde_json::Value>,
    /// Keys that were removed (present in `old` but not `new`).
    pub removed: HashMap<String, serde_json::Value>,
    /// Keys that changed value.
    pub changed: HashMap<String, (serde_json::Value, serde_json::Value)>,
    /// Keys that remained the same.
    pub unchanged_count: usize,
}

impl PolicyDiff {
    /// Compute the diff between two snapshots.
    pub fn compute(old: &PolicySnapshot, new: &PolicySnapshot) -> Self {
        let old_keys: HashSet<&String> = old.config.keys().collect();
        let new_keys: HashSet<&String> = new.config.keys().collect();

        let mut added = HashMap::new();
        let mut removed = HashMap::new();
        let mut changed = HashMap::new();
        let mut unchanged_count = 0;

        // Added keys
        for key in new_keys.difference(&old_keys) {
            added.insert((*key).clone(), new.config[*key].clone());
        }

        // Removed keys
        for key in old_keys.difference(&new_keys) {
            removed.insert((*key).clone(), old.config[*key].clone());
        }

        // Changed/unchanged keys
        for key in old_keys.intersection(&new_keys) {
            let old_val = &old.config[*key];
            let new_val = &new.config[*key];
            if old_val != new_val {
                changed.insert((*key).clone(), (old_val.clone(), new_val.clone()));
            } else {
                unchanged_count += 1;
            }
        }

        Self {
            added,
            removed,
            changed,
            unchanged_count,
        }
    }

    /// Returns true if there are no differences.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Total number of changes (added + removed + changed).
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_snapshot(config: HashMap<String, serde_json::Value>) -> PolicySnapshot {
        PolicySnapshot {
            id: Uuid::new_v4(),
            parent_id: None,
            score: 0.0,
            created_at: chrono::Utc::now(),
            config,
            description: "test".to_string(),
            feature_vector: vec![],
        }
    }

    #[test]
    fn test_identical_snapshots() {
        let mut config = HashMap::new();
        config.insert("key1".to_string(), serde_json::json!("value1"));
        let snap = make_snapshot(config);
        let diff = PolicyDiff::compute(&snap, &snap);
        assert!(diff.is_empty());
        assert_eq!(diff.unchanged_count, 1);
    }

    #[test]
    fn test_added_keys() {
        let old = make_snapshot(HashMap::new());
        let mut new_config = HashMap::new();
        new_config.insert("new_key".to_string(), serde_json::json!(42));
        let new = make_snapshot(new_config);
        let diff = PolicyDiff::compute(&old, &new);
        assert_eq!(diff.added.len(), 1);
        assert!(diff.added.contains_key("new_key"));
    }

    #[test]
    fn test_removed_keys() {
        let mut old_config = HashMap::new();
        old_config.insert("old_key".to_string(), serde_json::json!("gone"));
        let old = make_snapshot(old_config);
        let new = make_snapshot(HashMap::new());
        let diff = PolicyDiff::compute(&old, &new);
        assert_eq!(diff.removed.len(), 1);
    }

    #[test]
    fn test_changed_values() {
        let mut old_config = HashMap::new();
        old_config.insert("key".to_string(), serde_json::json!(1));
        let old = make_snapshot(old_config);

        let mut new_config = HashMap::new();
        new_config.insert("key".to_string(), serde_json::json!(2));
        let new = make_snapshot(new_config);

        let diff = PolicyDiff::compute(&old, &new);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.change_count(), 1);
    }
}
