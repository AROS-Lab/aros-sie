//! Integration tests simulating the full Loop 0 lifecycle:
//! PERCEIVE → SELF-MODEL UPDATE → CRITIQUE → POLICY REVISION → IDENTITY CHECK → PERSIST

use std::collections::HashMap;

use aros_sie::critic::engine::MetaCognitionEngine;
use aros_sie::critic::output::CriticOutput;
use aros_sie::critic::rules::CriticConfig;
use aros_sie::critic::Critic;
use aros_sie::identity::guard::IdentityGuard;
use aros_sie::identity::IdentityChecker;
use aros_sie::perceive::engine::PerceptionEngine;
use aros_sie::perceive::PerceptionSource;
use aros_sie::persistence::store::InMemoryStateStore;
use aros_sie::persistence::StateStore;
use aros_sie::policy::archive::PolicyArchive;
use aros_sie::policy::snapshot::PolicySnapshot;
use aros_sie::policy::PolicyStore;
use aros_sie::self_model::registry::SelfModelRegistry;
use aros_sie::self_model::SelfModel;
use aros_sie::shadow::dataset::{FrozenDataset, TestCase};
use aros_sie::shadow::pipeline::{ScoringFn, ShadowTestPipeline};
use aros_sie::shadow::ShadowEvaluator;
use aros_sie::types::{CapabilityId, Observation, PermissionChange, TaskOutcome};
use uuid::Uuid;

fn make_observation(cap: &str, outcome: TaskOutcome) -> Observation {
    Observation {
        capability: CapabilityId::new(cap),
        outcome,
        task_id: Some(Uuid::new_v4()),
        dag_id: Some(Uuid::new_v4()),
        timestamp: chrono::Utc::now(),
    }
}

fn make_snapshot(
    score: f64,
    parent: Option<Uuid>,
    config: HashMap<String, serde_json::Value>,
    feature_vector: Vec<f64>,
) -> PolicySnapshot {
    PolicySnapshot {
        id: Uuid::new_v4(),
        parent_id: parent,
        score,
        created_at: chrono::Utc::now(),
        config,
        description: format!("policy score={}", score),
        feature_vector,
    }
}

/// Test 1: Happy path — successful task flows through the full lifecycle.
#[test]
fn test_happy_path_full_lifecycle() {
    // 1. PERCEIVE: Ingest successful observations
    let mut perception = PerceptionEngine::new();
    let obs = make_observation("code_gen", TaskOutcome::Success);
    perception.ingest(obs.clone());
    let state = perception.perceive().unwrap();
    assert_eq!(state.recent_observations.len(), 1);

    // 2. SELF-MODEL UPDATE: Update the self-model
    let mut self_model = SelfModelRegistry::new();
    self_model.observe(&obs).unwrap();
    let confidence = self_model.confidence(&CapabilityId::new("code_gen")).unwrap();
    assert!(confidence > 0.5); // Success should increase confidence

    // 3. CRITIQUE: Evaluate the observation
    let engine = MetaCognitionEngine::new(CriticConfig::default(), SelfModelRegistry::new());
    let outputs = engine.evaluate(&obs).unwrap();
    // Success should produce NoAction
    assert!(outputs.iter().any(|o| matches!(o, CriticOutput::NoAction { .. })));

    // 4. POLICY REVISION: No policy change needed (NoAction)
    let mut archive = PolicyArchive::new();
    let policy = make_snapshot(0.8, None, HashMap::new(), vec![1.0, 0.0]);
    let policy_id = archive.commit(policy, None).unwrap();
    assert!(archive.get(policy_id).is_some());

    // 5. IDENTITY CHECK: Verify within bounds
    let baseline = make_snapshot(0.0, None, HashMap::new(), vec![1.0, 0.0]);
    let guard = IdentityGuard::with_defaults(baseline, 0.5);
    let current = archive.head().unwrap();
    let check = guard.check(current).unwrap();
    assert!(check.allowed);

    // 6. PERSIST: Save state
    let mut store = InMemoryStateStore::new();
    let snapshot_bytes = serde_json::to_vec(&self_model.snapshot()).unwrap();
    store.put("self_model/snapshot", snapshot_bytes).unwrap();
    assert!(store.exists("self_model/snapshot").unwrap());
}

/// Test 2: Failure triggers critic POLICY_UPDATE.
#[test]
fn test_failure_triggers_policy_update() {
    let config = CriticConfig {
        policy_update_confidence_threshold: 0.5,
        experiment_confidence_threshold: 0.05, // Very low so PolicyUpdate triggers first
        min_observations_for_policy: 3,
        alert_failure_streak: 10, // High so we don't trigger alert
        ..Default::default()
    };
    let registry = SelfModelRegistry::new();
    let mut engine = MetaCognitionEngine::new(config, registry);

    // Drive confidence low with multiple failures
    for _ in 0..5 {
        let obs = make_observation("reasoning", TaskOutcome::Failure);
        engine.evaluate_mut(&obs).unwrap();
    }

    // This failure should now trigger PolicyUpdate
    let obs = make_observation("reasoning", TaskOutcome::Failure);
    let outputs = engine.evaluate_mut(&obs).unwrap();

    let has_policy_update = outputs
        .iter()
        .any(|o| matches!(o, CriticOutput::PolicyUpdate { .. }));
    assert!(
        has_policy_update,
        "Expected PolicyUpdate after sustained failures, got: {:?}",
        outputs
    );
}

/// Test 3: Identity drift ceiling blocks excessive change.
#[test]
fn test_drift_ceiling_blocks_excessive_change() {
    let baseline = make_snapshot(0.0, None, HashMap::new(), vec![1.0, 0.0, 0.0]);
    let guard = IdentityGuard::with_defaults(baseline, 0.1); // Very tight ceiling

    // Propose a policy that is orthogonal to baseline
    let proposed = make_snapshot(0.5, None, HashMap::new(), vec![0.0, 1.0, 0.0]);
    let result = guard.check(&proposed).unwrap();

    assert!(!result.allowed, "Should be blocked: drift={}", result.drift);
    assert!(result.reason.is_some());
    assert!(result.drift > result.ceiling);
}

/// Test 4: Shadow test validates policy before commit.
#[test]
fn test_shadow_validates_policy() {
    // Create a frozen dataset
    let cases: Vec<TestCase> = (0..20)
        .map(|i| TestCase {
            input: serde_json::json!({ "difficulty": i as f64 / 20.0 }),
            expected_output: serde_json::json!({ "score": 1.0 }),
            ground_truth_score: Some(1.0),
        })
        .collect();
    let dataset = FrozenDataset::new("validation_set", cases);

    // Scoring function: policy threshold * (1 - difficulty)
    let scoring_fn: ScoringFn = Box::new(|policy, input| {
        let threshold = policy.config.get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);
        let difficulty = input["difficulty"].as_f64().unwrap_or(0.5);
        threshold * (1.0 - difficulty)
    });

    let pipeline = ShadowTestPipeline::new(dataset, scoring_fn);

    let mut baseline_config = HashMap::new();
    baseline_config.insert("threshold".to_string(), serde_json::json!(0.5));
    let baseline = make_snapshot(0.5, None, baseline_config, vec![0.5]);

    let mut candidate_config = HashMap::new();
    candidate_config.insert("threshold".to_string(), serde_json::json!(0.9));
    let candidate = make_snapshot(0.0, None, candidate_config, vec![0.9]);

    let result = pipeline.evaluate(&baseline, &candidate).unwrap();

    // Candidate (0.9 threshold) should outperform baseline (0.5 threshold)
    assert!(
        result.imp_at_k > 0.0,
        "Candidate should be better: imp@k={}",
        result.imp_at_k
    );
    assert!(result.candidate_score > result.baseline_score);

    // Only commit if shadow test passes
    let mut archive = PolicyArchive::new();
    let root_id = archive.commit(baseline, None).unwrap();
    if result.imp_at_k > 0.0 {
        let committed = archive.commit(candidate, Some(root_id)).unwrap();
        assert!(archive.get(committed).is_some());
    }
}

/// Test 5: Permission asymmetry blocks unauthorized loosening.
#[test]
fn test_permission_asymmetry_blocks_loosening() {
    let baseline = make_snapshot(0.0, None, HashMap::new(), vec![1.0]);
    let guard = IdentityGuard::with_defaults(baseline, 1.0);

    // Current policy with tight budget
    let mut current_config = HashMap::new();
    current_config.insert("budget.max_tokens".to_string(), serde_json::json!(1000));
    let current = make_snapshot(0.5, None, current_config, vec![1.0]);

    // Proposed policy loosening the budget
    let mut proposed_config = HashMap::new();
    proposed_config.insert("budget.max_tokens".to_string(), serde_json::json!(10000));
    let proposed = make_snapshot(0.5, None, proposed_config, vec![1.0]);

    let classification = guard.classify_permission_change(&current, &proposed);
    assert_eq!(classification, PermissionChange::Loosen);

    // Tightening should be auto-approved
    let mut tight_config = HashMap::new();
    tight_config.insert("budget.max_tokens".to_string(), serde_json::json!(500));
    let tighter = make_snapshot(0.5, None, tight_config, vec![1.0]);

    let classification = guard.classify_permission_change(&current, &tighter);
    assert_eq!(classification, PermissionChange::Tighten);

    // NEVER tier should always be blocked
    let mut never_config = HashMap::new();
    never_config.insert("identity.drift_ceiling".to_string(), serde_json::json!(0.5));
    let current_with_never = make_snapshot(0.5, None, never_config.clone(), vec![1.0]);

    let mut modified_never = HashMap::new();
    modified_never.insert("identity.drift_ceiling".to_string(), serde_json::json!(0.9));
    let proposed_never = make_snapshot(0.5, None, modified_never, vec![1.0]);

    let classification = guard.classify_permission_change(&current_with_never, &proposed_never);
    assert_eq!(classification, PermissionChange::Never);
}
