//! Policy Archive — branching evolution log with quality-diversity exploration.
//!
//! Policies are stored as a DAG (not a linear log). Supports forking from any
//! node, not just HEAD. Includes a two-tier skill library.

pub mod archive;
pub mod diff;
pub mod skill_library;
pub mod snapshot;

use crate::error::SieError;
use snapshot::PolicySnapshot;
use uuid::Uuid;

/// Trait for the policy storage component.
///
/// Implementations maintain a versioned archive of policy snapshots
/// organized as a DAG for quality-diversity exploration.
pub trait PolicyStore: Send + Sync {
    /// Commit a new policy snapshot as a child of the given parent.
    /// If parent_id is None, creates a root node.
    fn commit(&mut self, snapshot: PolicySnapshot, parent_id: Option<Uuid>) -> Result<Uuid, SieError>;

    /// Fork a new branch from an existing snapshot.
    fn fork(&mut self, from_id: Uuid, snapshot: PolicySnapshot) -> Result<Uuid, SieError>;

    /// Retrieve a policy snapshot by ID.
    fn get(&self, id: Uuid) -> Option<&PolicySnapshot>;

    /// Get the current HEAD snapshot (most recently committed on the main branch).
    fn head(&self) -> Option<&PolicySnapshot>;

    /// Get the k best-scoring snapshots across all branches.
    fn best_k(&self, k: usize) -> Vec<&PolicySnapshot>;

    /// List all snapshot IDs in the archive.
    fn list_ids(&self) -> Vec<Uuid>;

    /// Get the parent chain from a snapshot back to the root.
    fn ancestry(&self, id: Uuid) -> Vec<Uuid>;
}
