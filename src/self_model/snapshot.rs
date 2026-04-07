//! Serializable snapshots of the self-model state.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::CapabilityId;

/// A snapshot of a single capability's Beta distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySnapshot {
    pub capability: CapabilityId,
    pub alpha: f64,
    pub beta: f64,
    pub observation_count: u64,
}

/// A serializable snapshot of the entire self-model state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSnapshot {
    pub capabilities: HashMap<CapabilityId, CapabilitySnapshot>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
