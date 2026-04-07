//! Frozen historical dataset management for shadow testing.

use serde::{Deserialize, Serialize};

/// A single test case in a frozen dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Input context for the test case.
    pub input: serde_json::Value,
    /// Expected/baseline output.
    pub expected_output: serde_json::Value,
    /// Ground truth score (if available).
    pub ground_truth_score: Option<f64>,
}

/// A frozen dataset for shadow testing — immutable historical data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenDataset {
    /// Human-readable name for this dataset.
    pub name: String,
    /// When this dataset was captured.
    pub captured_at: chrono::DateTime<chrono::Utc>,
    /// The test cases.
    pub cases: Vec<TestCase>,
}

impl FrozenDataset {
    /// Create a new dataset.
    pub fn new(name: impl Into<String>, cases: Vec<TestCase>) -> Self {
        Self {
            name: name.into(),
            captured_at: chrono::Utc::now(),
            cases,
        }
    }

    /// Number of test cases in this dataset.
    pub fn len(&self) -> usize {
        self.cases.len()
    }

    /// Check if the dataset is empty.
    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }

    /// Serialize to JSON bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
