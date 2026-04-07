//! # AROS Self-Improvement Engine (SIE)
//!
//! The SIE provides the building blocks for Loop 0's six-step lifecycle:
//! PERCEIVE → SELF-MODEL UPDATE → CRITIQUE → POLICY REVISION → IDENTITY CHECK → PERSIST
//!
//! This is a library crate consumed by the AROS kernel. The kernel's Loop 0 orchestrator
//! calls SIE functions — the SIE does not own loop orchestration.

pub mod types;

pub mod self_model;
pub mod critic;
pub mod policy;
pub mod identity;
pub mod shadow;
pub mod perceive;
pub mod persistence;
pub mod telemetry;

/// SIE-specific error types.
pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum SieError {
        #[error("self-model error: {0}")]
        SelfModel(String),

        #[error("critic error: {0}")]
        Critic(String),

        #[error("policy error: {0}")]
        Policy(String),

        #[error("identity check failed: drift {drift:.4} exceeds ceiling {ceiling:.4}")]
        DriftCeilingExceeded { drift: f64, ceiling: f64 },

        #[error("permission denied: {0}")]
        PermissionDenied(String),

        #[error("shadow test error: {0}")]
        ShadowTest(String),

        #[error("persistence error: {0}")]
        Persistence(String),

        #[error("serialization error: {0}")]
        Serialization(#[from] serde_json::Error),
    }
}

pub use error::SieError;
