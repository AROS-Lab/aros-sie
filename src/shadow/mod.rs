//! Shadow Testing Pipeline — validate policy changes against frozen historical data.
//!
//! Replays frozen datasets through candidate policies and computes improvement
//! metrics (imp@k) without production side effects.

pub mod dataset;
pub mod evaluation;
pub mod pipeline;

use crate::error::SieError;
use crate::policy::snapshot::PolicySnapshot;
use crate::types::ShadowTestResult;

/// Trait for the shadow testing component.
pub trait ShadowEvaluator: Send + Sync {
    /// Run a shadow test comparing a candidate policy against the baseline.
    fn evaluate(
        &self,
        baseline: &PolicySnapshot,
        candidate: &PolicySnapshot,
    ) -> Result<ShadowTestResult, SieError>;

    /// Run evaluation at a specific rank k for the imp@k metric.
    fn evaluate_at_k(
        &self,
        baseline: &PolicySnapshot,
        candidate: &PolicySnapshot,
        k: usize,
    ) -> Result<ShadowTestResult, SieError>;
}
