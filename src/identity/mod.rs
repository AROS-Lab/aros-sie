//! Identity Guard — drift ceiling enforcement and permission asymmetry.
//!
//! Prevents the ship-of-Theseus problem by measuring cumulative policy drift
//! against a baseline using cosine similarity. Enforces permission asymmetry:
//! tighten=auto-approve, loosen=human-review, NEVER tier.

pub mod distance;
pub mod guard;
pub mod permissions;

use crate::error::SieError;
use crate::policy::snapshot::PolicySnapshot;
use crate::types::PermissionChange;

/// Result of an identity check.
#[derive(Debug, Clone)]
pub struct IdentityCheckResult {
    /// Current drift distance from baseline (0.0 = identical, 1.0 = maximally different).
    pub drift: f64,
    /// The configured ceiling.
    pub ceiling: f64,
    /// Whether the proposed change is allowed.
    pub allowed: bool,
    /// If not allowed, the reason.
    pub reason: Option<String>,
}

/// Trait for the identity checking component.
pub trait IdentityChecker: Send + Sync {
    /// Check whether a proposed policy change is within identity bounds.
    fn check(&self, proposed: &PolicySnapshot) -> Result<IdentityCheckResult, SieError>;

    /// Classify a permission change as tighten/loosen/never.
    fn classify_permission_change(
        &self,
        current: &PolicySnapshot,
        proposed: &PolicySnapshot,
    ) -> PermissionChange;

    /// Get the current cumulative drift from baseline.
    fn current_drift(&self) -> f64;

    /// Reset the baseline to the given snapshot.
    fn reset_baseline(&mut self, baseline: PolicySnapshot);
}
