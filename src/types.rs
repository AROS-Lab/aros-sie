//! Shared types used across all SIE modules.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an agent capability (e.g., "code_generation", "reasoning").
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

impl CapabilityId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An observation of a task outcome used to update the self-model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub capability: CapabilityId,
    pub outcome: TaskOutcome,
    pub task_id: Option<Uuid>,
    pub dag_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// The outcome of a task execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskOutcome {
    /// Task completed successfully.
    Success,
    /// Task failed.
    Failure,
    /// Task completed but with degraded quality.
    Degraded,
}

/// Memory tiers aligned with the AROS architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTier {
    /// L1: In-context working memory, not expendable.
    L1Working,
    /// L2: Session memory, recency-biased, partially expendable.
    L2Session,
    /// L3: Long-term VectorDB memory, expendable.
    L3LongTerm,
    /// L4: Error journal, pattern-matched, expendable.
    L4ErrorJournal,
}

/// Which loop originated a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopOrigin {
    Loop0Meta,
    Loop1Agentic,
    Loop2Harness,
}

/// Security zone for request routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityZone {
    /// Any provider allowed.
    Green,
    /// Approved providers only.
    Yellow,
    /// Local models only (Ollama) or pre-approved endpoints.
    Red,
}

/// Priority levels for request scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Loop 0 meta-observations, health checks. Reserved budget.
    P0,
    /// Loop 1 task execution. Standard admission control.
    P1,
    /// SIE experiments, A/B comparisons. Spare capacity only.
    P2,
}

/// Classification of a permission change for the asymmetry rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionChange {
    /// Making permissions more restrictive — auto-approved.
    Tighten,
    /// Making permissions less restrictive — requires human review.
    Loosen,
    /// Attempting to modify a NEVER-tier permission — always blocked.
    Never,
}

/// A scored value for policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredItem<T> {
    pub item: T,
    pub score: f64,
}

/// Result of a shadow test evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowTestResult {
    pub baseline_score: f64,
    pub candidate_score: f64,
    pub imp_at_k: f64,
    pub sample_count: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of critic output from the meta-cognition engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriticOutputType {
    PolicyUpdate,
    MemoryWrite,
    ToolAction,
    Alert,
    NoAction,
    Experiment,
}

/// Skill tier classification for the two-tier skill library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTier {
    /// Domain-specific skills (e.g., "rust_debugging", "sql_optimization").
    TaskSkill,
    /// Domain-general meta-skills (e.g., "decomposition", "self_monitoring").
    MetaSkill,
}
