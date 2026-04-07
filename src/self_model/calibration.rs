//! Bayesian calibration logic for Beta distributions.
//!
//! Each capability is modeled as Beta(α, β):
//! - Prior: Beta(1, 1) (uniform)
//! - On success: α += 1
//! - On failure: β += 1
//! - Mean confidence: α / (α + β)
//! - Decay: multiply both by λ to weight recent observations

use serde::{Deserialize, Serialize};

/// A Beta distribution representing confidence in a capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaDistribution {
    pub alpha: f64,
    pub beta: f64,
    pub observation_count: u64,
}

impl Default for BetaDistribution {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            observation_count: 0,
        }
    }
}

impl BetaDistribution {
    /// Create a new Beta distribution with uniform prior.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update on a successful observation.
    pub fn record_success(&mut self) {
        self.alpha += 1.0;
        self.observation_count += 1;
    }

    /// Update on a failed observation.
    pub fn record_failure(&mut self) {
        self.beta += 1.0;
        self.observation_count += 1;
    }

    /// Mean of the Beta distribution: α / (α + β).
    pub fn mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Variance of the Beta distribution.
    pub fn variance(&self) -> f64 {
        let sum = self.alpha + self.beta;
        (self.alpha * self.beta) / (sum * sum * (sum + 1.0))
    }

    /// Approximate confidence interval using normal approximation.
    /// Returns (lower, upper) bounds at the given number of standard deviations.
    pub fn confidence_interval(&self, num_std_devs: f64) -> (f64, f64) {
        let mean = self.mean();
        let std_dev = self.variance().sqrt();
        let lower = (mean - num_std_devs * std_dev).max(0.0);
        let upper = (mean + num_std_devs * std_dev).min(1.0);
        (lower, upper)
    }

    /// Apply temporal decay: multiply both α and β by λ (0 < λ ≤ 1).
    /// This reduces the weight of old observations while preserving the mean.
    pub fn decay(&mut self, lambda: f64) {
        let lambda = lambda.clamp(0.0, 1.0);
        self.alpha *= lambda;
        self.beta *= lambda;
        // Ensure minimum values to prevent degenerate distributions
        if self.alpha < 1.0 {
            self.alpha = 1.0;
        }
        if self.beta < 1.0 {
            self.beta = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_prior() {
        let dist = BetaDistribution::new();
        assert!((dist.mean() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_success_increases_confidence() {
        let mut dist = BetaDistribution::new();
        let initial = dist.mean();
        dist.record_success();
        assert!(dist.mean() > initial);
    }

    #[test]
    fn test_failure_decreases_confidence() {
        let mut dist = BetaDistribution::new();
        let initial = dist.mean();
        dist.record_failure();
        assert!(dist.mean() < initial);
    }

    #[test]
    fn test_decay_preserves_minimum() {
        let mut dist = BetaDistribution::new();
        dist.decay(0.1);
        assert!(dist.alpha >= 1.0);
        assert!(dist.beta >= 1.0);
    }

    #[test]
    fn test_confidence_interval_bounds() {
        let mut dist = BetaDistribution::new();
        for _ in 0..10 {
            dist.record_success();
        }
        let (lower, upper) = dist.confidence_interval(2.0);
        assert!(lower >= 0.0);
        assert!(upper <= 1.0);
        assert!(lower < upper);
    }
}
