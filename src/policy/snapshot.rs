//! Policy snapshot — a versioned point-in-time capture of policy state.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A versioned snapshot of the agent's policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySnapshot {
    /// Unique identifier for this snapshot.
    pub id: Uuid,
    /// Parent snapshot ID (None for root).
    pub parent_id: Option<Uuid>,
    /// Numeric score for quality-diversity selection.
    pub score: f64,
    /// When this snapshot was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The policy configuration as key-value pairs.
    pub config: HashMap<String, serde_json::Value>,
    /// Human-readable description of what changed.
    pub description: String,
    /// Vectorized representation for cosine similarity (identity check).
    pub feature_vector: Vec<f64>,
}
