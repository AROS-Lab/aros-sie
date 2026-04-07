//! Tracing span definitions for SIE operations.
//!
//! Provides instrumentation macros and span constructors for all SIE
//! operations, producing OTLP-compatible structured telemetry.

use tracing::{info_span, Span};
use uuid::Uuid;

/// Create a span for self-model update operations.
pub fn self_model_update_span(capability: &str, outcome: &str) -> Span {
    info_span!(
        "sie.self_model.update",
        capability = %capability,
        outcome = %outcome,
        otel.kind = "INTERNAL"
    )
}

/// Create a span for critic evaluation operations.
pub fn critic_evaluate_span(observation_count: usize) -> Span {
    info_span!(
        "sie.critic.evaluate",
        observation_count = observation_count,
        otel.kind = "INTERNAL"
    )
}

/// Create a span for policy commit operations.
pub fn policy_commit_span(snapshot_id: Uuid, parent_id: Option<Uuid>) -> Span {
    info_span!(
        "sie.policy.commit",
        snapshot_id = %snapshot_id,
        parent_id = ?parent_id,
        otel.kind = "INTERNAL"
    )
}

/// Create a span for policy fork operations.
pub fn policy_fork_span(from_id: Uuid, new_id: Uuid) -> Span {
    info_span!(
        "sie.policy.fork",
        from_id = %from_id,
        new_id = %new_id,
        otel.kind = "INTERNAL"
    )
}

/// Create a span for identity check operations.
pub fn identity_check_span(drift: f64, ceiling: f64) -> Span {
    info_span!(
        "sie.identity.check",
        drift = drift,
        ceiling = ceiling,
        otel.kind = "INTERNAL"
    )
}

/// Create a span for shadow test operations.
pub fn shadow_test_span(dataset_name: &str, k: usize) -> Span {
    info_span!(
        "sie.shadow.test",
        dataset = %dataset_name,
        k = k,
        otel.kind = "INTERNAL"
    )
}

/// Create a span for perception operations.
pub fn perception_span(observation_count: usize) -> Span {
    info_span!(
        "sie.perceive",
        observation_count = observation_count,
        otel.kind = "INTERNAL"
    )
}

/// Create a span for decay operations.
pub fn decay_span(lambda: f64, capabilities_count: usize) -> Span {
    info_span!(
        "sie.self_model.decay",
        lambda = lambda,
        capabilities_count = capabilities_count,
        otel.kind = "INTERNAL"
    )
}
