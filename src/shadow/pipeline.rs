//! ShadowTestPipeline — replays frozen datasets to evaluate policy candidates.

use crate::error::SieError;
use crate::policy::snapshot::PolicySnapshot;
use crate::types::ShadowTestResult;

use super::dataset::FrozenDataset;
use super::evaluation;
use super::ShadowEvaluator;

/// Scoring function type: given a policy and a test case input, produce a score.
pub type ScoringFn = Box<dyn Fn(&PolicySnapshot, &serde_json::Value) -> f64 + Send + Sync>;

/// Pipeline that evaluates policy candidates against frozen historical data.
pub struct ShadowTestPipeline {
    /// The frozen dataset to test against.
    dataset: FrozenDataset,
    /// Scoring function that evaluates a policy on a test case.
    scoring_fn: ScoringFn,
}

impl ShadowTestPipeline {
    /// Create a new pipeline with the given dataset and scoring function.
    pub fn new(dataset: FrozenDataset, scoring_fn: ScoringFn) -> Self {
        Self {
            dataset,
            scoring_fn,
        }
    }

    /// Score all test cases in the dataset with a given policy.
    fn score_all(&self, policy: &PolicySnapshot) -> Vec<f64> {
        self.dataset
            .cases
            .iter()
            .map(|tc| (self.scoring_fn)(policy, &tc.input))
            .collect()
    }
}

impl ShadowEvaluator for ShadowTestPipeline {
    fn evaluate(
        &self,
        baseline: &PolicySnapshot,
        candidate: &PolicySnapshot,
    ) -> Result<ShadowTestResult, SieError> {
        self.evaluate_at_k(baseline, candidate, 50)
    }

    fn evaluate_at_k(
        &self,
        baseline: &PolicySnapshot,
        candidate: &PolicySnapshot,
        k: usize,
    ) -> Result<ShadowTestResult, SieError> {
        if self.dataset.is_empty() {
            return Err(SieError::ShadowTest("Dataset is empty".to_string()));
        }

        let baseline_scores = self.score_all(baseline);
        let candidate_scores = self.score_all(candidate);

        Ok(evaluation::compare(&baseline_scores, &candidate_scores, k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadow::dataset::TestCase;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_snapshot(config_val: f64) -> PolicySnapshot {
        let mut config = HashMap::new();
        config.insert("threshold".to_string(), serde_json::json!(config_val));
        PolicySnapshot {
            id: Uuid::new_v4(),
            parent_id: None,
            score: 0.0,
            created_at: chrono::Utc::now(),
            config,
            description: "test".to_string(),
            feature_vector: vec![config_val],
        }
    }

    fn make_dataset() -> FrozenDataset {
        let cases = (0..10)
            .map(|i| TestCase {
                input: serde_json::json!({ "difficulty": i as f64 / 10.0 }),
                expected_output: serde_json::json!({ "score": 1.0 }),
                ground_truth_score: Some(1.0),
            })
            .collect();
        FrozenDataset::new("test_dataset", cases)
    }

    #[test]
    fn test_pipeline_candidate_better() {
        let dataset = make_dataset();
        // Scoring: policy threshold * (1.0 - difficulty)
        let scoring_fn: ScoringFn = Box::new(|policy, input| {
            let threshold = policy.config["threshold"].as_f64().unwrap_or(0.5);
            let difficulty = input["difficulty"].as_f64().unwrap_or(0.5);
            threshold * (1.0 - difficulty)
        });

        let pipeline = ShadowTestPipeline::new(dataset, scoring_fn);
        let baseline = make_snapshot(0.5);
        let candidate = make_snapshot(0.9); // Higher threshold = better scores

        let result = pipeline.evaluate(&baseline, &candidate).unwrap();
        assert!(result.imp_at_k > 0.0, "Candidate should be better");
        assert!(result.candidate_score > result.baseline_score);
    }

    #[test]
    fn test_pipeline_empty_dataset() {
        let dataset = FrozenDataset::new("empty", vec![]);
        let scoring_fn: ScoringFn = Box::new(|_, _| 0.0);
        let pipeline = ShadowTestPipeline::new(dataset, scoring_fn);
        let snap = make_snapshot(0.5);
        let result = pipeline.evaluate(&snap, &snap);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_at_specific_k() {
        let dataset = make_dataset();
        let scoring_fn: ScoringFn = Box::new(|policy, input| {
            let threshold = policy.config["threshold"].as_f64().unwrap_or(0.5);
            let difficulty = input["difficulty"].as_f64().unwrap_or(0.5);
            threshold * (1.0 - difficulty)
        });

        let pipeline = ShadowTestPipeline::new(dataset, scoring_fn);
        let baseline = make_snapshot(0.5);
        let candidate = make_snapshot(0.8);

        let result = pipeline.evaluate_at_k(&baseline, &candidate, 3).unwrap();
        assert!(result.imp_at_k > 0.0);
        assert_eq!(result.sample_count, 10);
    }
}
