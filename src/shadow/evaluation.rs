//! imp@k metric computation and A/B comparison logic.
//!
//! imp@k (improvement at rank k) measures how much a candidate policy
//! improves over a baseline when evaluated on the top-k results.

use crate::types::ShadowTestResult;

/// Compute imp@k: improvement of candidate over baseline at rank k.
///
/// Given baseline scores and candidate scores (both sorted descending),
/// imp@k = (mean of top-k candidate scores) - (mean of top-k baseline scores)
///        / (mean of top-k baseline scores)
///
/// A positive value means the candidate is better.
pub fn imp_at_k(baseline_scores: &[f64], candidate_scores: &[f64], k: usize) -> f64 {
    if k == 0 || baseline_scores.is_empty() || candidate_scores.is_empty() {
        return 0.0;
    }

    let baseline_top_k = top_k_mean(baseline_scores, k);
    let candidate_top_k = top_k_mean(candidate_scores, k);

    if baseline_top_k.abs() < f64::EPSILON {
        // Avoid division by zero; if baseline is zero, any improvement is infinite
        // Return raw difference instead
        return candidate_top_k;
    }

    (candidate_top_k - baseline_top_k) / baseline_top_k.abs()
}

/// Compute the mean of the top-k values from a slice (sorted descending).
fn top_k_mean(scores: &[f64], k: usize) -> f64 {
    let mut sorted = scores.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let take = k.min(sorted.len());
    if take == 0 {
        return 0.0;
    }
    sorted[..take].iter().sum::<f64>() / take as f64
}

/// Build a ShadowTestResult from baseline and candidate score arrays.
pub fn compare(baseline_scores: &[f64], candidate_scores: &[f64], k: usize) -> ShadowTestResult {
    let baseline_mean = if baseline_scores.is_empty() {
        0.0
    } else {
        baseline_scores.iter().sum::<f64>() / baseline_scores.len() as f64
    };
    let candidate_mean = if candidate_scores.is_empty() {
        0.0
    } else {
        candidate_scores.iter().sum::<f64>() / candidate_scores.len() as f64
    };

    ShadowTestResult {
        baseline_score: baseline_mean,
        candidate_score: candidate_mean,
        imp_at_k: imp_at_k(baseline_scores, candidate_scores, k),
        sample_count: baseline_scores.len().min(candidate_scores.len()),
        timestamp: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imp_at_k_improvement() {
        let baseline = vec![0.5, 0.4, 0.3, 0.2, 0.1];
        let candidate = vec![0.8, 0.7, 0.6, 0.2, 0.1];
        let result = imp_at_k(&baseline, &candidate, 3);
        // Top-3 baseline mean: (0.5+0.4+0.3)/3 = 0.4
        // Top-3 candidate mean: (0.8+0.7+0.6)/3 = 0.7
        // imp@3 = (0.7 - 0.4) / 0.4 = 0.75
        assert!((result - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_imp_at_k_regression() {
        let baseline = vec![0.8, 0.7, 0.6];
        let candidate = vec![0.3, 0.2, 0.1];
        let result = imp_at_k(&baseline, &candidate, 3);
        assert!(result < 0.0); // Candidate is worse
    }

    #[test]
    fn test_imp_at_k_equal() {
        let scores = vec![0.5, 0.5, 0.5];
        let result = imp_at_k(&scores, &scores, 3);
        assert!(result.abs() < 1e-10);
    }

    #[test]
    fn test_imp_at_k_empty() {
        assert_eq!(imp_at_k(&[], &[0.5], 1), 0.0);
        assert_eq!(imp_at_k(&[0.5], &[], 1), 0.0);
        assert_eq!(imp_at_k(&[0.5], &[0.5], 0), 0.0);
    }

    #[test]
    fn test_compare_result() {
        let baseline = vec![0.4, 0.5, 0.3];
        let candidate = vec![0.7, 0.8, 0.6];
        let result = compare(&baseline, &candidate, 2);
        assert!(result.imp_at_k > 0.0);
        assert_eq!(result.sample_count, 3);
    }
}
