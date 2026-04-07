//! PolicyArchive — DAG-based branching evolution log for policy snapshots.

use std::collections::HashMap;

use uuid::Uuid;

use crate::error::SieError;

use super::snapshot::PolicySnapshot;
use super::PolicyStore;

/// A DAG-based archive of policy snapshots supporting branching evolution.
pub struct PolicyArchive {
    /// All snapshots indexed by ID.
    snapshots: HashMap<Uuid, PolicySnapshot>,
    /// The current HEAD (most recently committed on main branch).
    head_id: Option<Uuid>,
    /// Root nodes (snapshots with no parent).
    roots: Vec<Uuid>,
}

impl PolicyArchive {
    /// Create a new empty archive.
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
            head_id: None,
            roots: Vec::new(),
        }
    }

    /// Get the number of snapshots in the archive.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if the archive is empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

impl Default for PolicyArchive {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyStore for PolicyArchive {
    fn commit(&mut self, snapshot: PolicySnapshot, parent_id: Option<Uuid>) -> Result<Uuid, SieError> {
        let id = snapshot.id;

        // Validate parent exists if specified
        if let Some(pid) = parent_id
            && !self.snapshots.contains_key(&pid) {
                return Err(SieError::Policy(format!("Parent snapshot {} not found", pid)));
        }

        if parent_id.is_none() {
            self.roots.push(id);
        }

        self.snapshots.insert(id, snapshot);
        self.head_id = Some(id);
        Ok(id)
    }

    fn fork(&mut self, from_id: Uuid, snapshot: PolicySnapshot) -> Result<Uuid, SieError> {
        if !self.snapshots.contains_key(&from_id) {
            return Err(SieError::Policy(format!("Cannot fork from unknown snapshot {}", from_id)));
        }

        let id = snapshot.id;
        let mut forked = snapshot;
        forked.parent_id = Some(from_id);
        self.snapshots.insert(id, forked);
        // Fork does NOT move HEAD — it creates a branch
        Ok(id)
    }

    fn get(&self, id: Uuid) -> Option<&PolicySnapshot> {
        self.snapshots.get(&id)
    }

    fn head(&self) -> Option<&PolicySnapshot> {
        self.head_id.and_then(|id| self.snapshots.get(&id))
    }

    fn best_k(&self, k: usize) -> Vec<&PolicySnapshot> {
        let mut sorted: Vec<&PolicySnapshot> = self.snapshots.values().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(k);
        sorted
    }

    fn list_ids(&self) -> Vec<Uuid> {
        self.snapshots.keys().copied().collect()
    }

    fn ancestry(&self, id: Uuid) -> Vec<Uuid> {
        let mut chain = Vec::new();
        let mut current = Some(id);
        while let Some(cid) = current {
            chain.push(cid);
            current = self.snapshots.get(&cid).and_then(|s| s.parent_id);
        }
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_snapshot(score: f64, parent: Option<Uuid>) -> PolicySnapshot {
        PolicySnapshot {
            id: Uuid::new_v4(),
            parent_id: parent,
            score,
            created_at: chrono::Utc::now(),
            config: HashMap::new(),
            description: format!("test snapshot score={}", score),
            feature_vector: vec![score, 1.0 - score],
        }
    }

    #[test]
    fn test_commit_root() {
        let mut archive = PolicyArchive::new();
        let snap = make_snapshot(0.8, None);
        let id = archive.commit(snap, None).unwrap();
        assert_eq!(archive.len(), 1);
        assert!(archive.get(id).is_some());
        assert!(archive.head().is_some());
    }

    #[test]
    fn test_commit_with_parent() {
        let mut archive = PolicyArchive::new();
        let root = make_snapshot(0.5, None);
        let root_id = archive.commit(root, None).unwrap();

        let child = make_snapshot(0.7, Some(root_id));
        let child_id = archive.commit(child, Some(root_id)).unwrap();

        assert_eq!(archive.len(), 2);
        assert_eq!(archive.head().unwrap().id, child_id);
    }

    #[test]
    fn test_commit_invalid_parent() {
        let mut archive = PolicyArchive::new();
        let snap = make_snapshot(0.5, None);
        let result = archive.commit(snap, Some(Uuid::new_v4()));
        assert!(result.is_err());
    }

    #[test]
    fn test_fork_creates_branch() {
        let mut archive = PolicyArchive::new();
        let root = make_snapshot(0.5, None);
        let root_id = archive.commit(root, None).unwrap();

        let branch = make_snapshot(0.9, None);
        let branch_id = archive.fork(root_id, branch).unwrap();

        // HEAD should still be root (fork doesn't move HEAD)
        assert_eq!(archive.head().unwrap().id, root_id);
        // Branch should exist and have root as parent
        let branch_snap = archive.get(branch_id).unwrap();
        assert_eq!(branch_snap.parent_id, Some(root_id));
    }

    #[test]
    fn test_fork_from_nonexistent() {
        let mut archive = PolicyArchive::new();
        let snap = make_snapshot(0.5, None);
        let result = archive.fork(Uuid::new_v4(), snap);
        assert!(result.is_err());
    }

    #[test]
    fn test_best_k() {
        let mut archive = PolicyArchive::new();
        for score in [0.3, 0.9, 0.5, 0.7, 0.1] {
            let snap = make_snapshot(score, None);
            archive.commit(snap, None).unwrap();
        }
        let best = archive.best_k(3);
        assert_eq!(best.len(), 3);
        assert!(best[0].score >= best[1].score);
        assert!(best[1].score >= best[2].score);
        assert!((best[0].score - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ancestry_chain() {
        let mut archive = PolicyArchive::new();
        let root = make_snapshot(0.5, None);
        let root_id = archive.commit(root, None).unwrap();

        let child = make_snapshot(0.6, Some(root_id));
        let child_id = archive.commit(child, Some(root_id)).unwrap();

        let grandchild = make_snapshot(0.7, Some(child_id));
        let gc_id = archive.commit(grandchild, Some(child_id)).unwrap();

        let chain = archive.ancestry(gc_id);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0], gc_id);
        assert_eq!(chain[1], child_id);
        assert_eq!(chain[2], root_id);
    }
}
