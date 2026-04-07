# AROS Self-Improvement Engine (SIE)

The self-improvement layer of the **Agent Runtime Operating System (AROS)** — a library crate implementing Loop 0's six-step meta-cognition lifecycle with Bayesian self-modeling, policy evolution, shadow testing, and identity-preserving drift control.

## Architecture

The SIE is a **library crate** consumed by the AROS kernel. The kernel's Loop 0 orchestrator calls SIE functions — the SIE does not own loop orchestration.

```
Loop 0 Orchestrator (kernel)
  │
  ├── PERCEIVE ─────────→ sie::perceive::PerceptionEngine
  ├── SELF-MODEL UPDATE ─→ sie::self_model::SelfModelRegistry
  ├── CRITIQUE ──────────→ sie::critic::MetaCognitionEngine
  ├── POLICY REVISION ───→ sie::policy::PolicyArchive
  ├── IDENTITY CHECK ────→ sie::identity::IdentityGuard
  └── PERSIST ───────────→ sie::persistence::StateStore
```

**Freudian Architecture Mapping** (internal structure, not user-facing):
- **Drive (Id)** → Objective Function
- **Model (Ego)** → Self-Model Registry
- **Policy (Ego)** → Harness Configuration
- **Memory (Past Self)** → L1–L4 Context Memory
- **Critic (Superego evaluator)** → Meta-Cognition Engine
- **Identity (Superego ideal)** → Meta-Goal Registry

## Modules

### Self-Model (`src/self_model/`)

Bayesian calibration of agent capabilities using Beta distributions. Each capability is tracked as `Beta(α,β)` — updated on task success/failure, with temporal decay to weight recent observations.

| Component | Description |
|-----------|-------------|
| **SelfModel** trait | Probabilistic capability modeling with confidence tracking |
| **SelfModelRegistry** | Concrete implementation with Beta distribution per capability |
| **BetaDistribution** | Success/failure updates, confidence intervals, decay |
| **ModelSnapshot** | Serializable state for persistence and restore |

### Critic (`src/critic/`)

Meta-cognition engine that evaluates task outcomes against self-model expectations and emits typed action recommendations.

| Output | When |
|--------|------|
| `PolicyUpdate` | Sustained low confidence suggests strategy change |
| `MemoryWrite` | Failure/degraded outcome should be recorded |
| `ToolAction` | A tool action (e.g., re-run test) is recommended |
| `Alert` | Consecutive failure streak exceeds threshold |
| `NoAction` | Observation is within expected bounds |
| `Experiment` | Very low confidence suggests A/B testing |

### Policy Archive (`src/policy/`)

Branching evolution log for policy snapshots, organized as a DAG (not a linear log).

| Component | Description |
|-----------|-------------|
| **PolicyStore** trait | Version-controlled policy management with branching and ancestry |
| **PolicyArchive** | DAG-based storage with fork-from-any-node (quality-diversity exploration) |
| **PolicySnapshot** | Versioned config state with feature vectors for identity comparison |
| **SkillLibrary** | Two-tier: TaskSkill (domain-specific) + MetaSkill (domain-general) |
| **PolicyDiff** | Structured comparison between policy versions |
| **best_k** | Population-based selection for quality-diversity (HyperAgents-inspired) |

### Identity Guard (`src/identity/`)

Prevents the ship-of-Theseus problem by enforcing cumulative drift ceilings and permission asymmetry.

| Component | Description |
|-----------|-------------|
| **IdentityChecker** trait | Drift measurement via cosine similarity against identity anchors |
| **IdentityGuard** | Blocks changes exceeding cumulative drift ceiling |
| **PermissionAsymmetry** | Tighten = auto-approve, Loosen = human-review, NEVER = kernel-blocked |

### Shadow Testing (`src/shadow/`)

Validates policy changes against frozen historical data without production side effects.

| Component | Description |
|-----------|-------------|
| **ShadowEvaluator** trait | Run candidate policy against baseline on historical task data |
| **ShadowTestPipeline** | Replays frozen datasets through candidate policies |
| **imp@k metric** | Improvement at rank k compared to baseline (from HyperAgents paper) |
| **ShadowTestResult** | Baseline/candidate scores with sample count |

### Perception (`src/perceive/`)

Aggregates telemetry signals (task outcomes, resource utilization) into a unified perception state for Loop 0's PERCEIVE step.

| Component | Description |
|-----------|-------------|
| **PerceptionSource** trait | Ingest observations, produce unified perception state |
| **PerceptionEngine** | Concrete implementation with observation windowing |

### Persistence (`src/persistence/`)

Trait-based state store abstraction with an in-memory implementation. The kernel provides the production SQLite/WAL backend.

| Component | Description |
|-----------|-------------|
| **StateStore** trait | Key-value contract (put/get/delete/list_keys/exists) with namespaced string keys |
| **InMemoryStateStore** | Reference implementation for testing |
| **SieEvent** | Event sourcing types: SelfModelUpdated, CriticOutput, PolicyCommitted, PolicyForked, IdentityChecked, PermissionClassified, ShadowTestRun, DecayApplied |

### Telemetry (`src/telemetry/`)

OTLP-compatible tracing span definitions for all SIE operations. Each Meta Loop step emits structured spans for observability.

### Common Types (`src/types.rs`)

| Type | Description |
|------|-------------|
| **CapabilityId** | Unique capability identifier |
| **Observation** | Task outcome record with capability, outcome, task/dag IDs |
| **TaskOutcome** | Success / Failure / Degraded |
| **MemoryTier** | L1Working, L2Session, L3LongTerm, L4ErrorJournal |
| **LoopOrigin** | Loop0Meta, Loop1Agentic, Loop2Harness |
| **SecurityZone** | Green, Yellow, Red |
| **Priority** | P0, P1, P2 |
| **PermissionChange** | Tighten, Loosen, Never |
| **ScoredItem\<T\>** | Scored wrapper for policy evaluation |

## Integration with aros-kernel

The SIE exposes trait-based abstractions for the kernel to consume:

| Trait | Kernel Integration Point |
|-------|-------------------------|
| `SelfModel` | Loop 0 SELF-MODEL UPDATE step |
| `Critic` | Loop 0 CRITIQUE step |
| `PolicyStore` | Loop 0 POLICY REVISION step |
| `IdentityChecker` | Loop 0 IDENTITY CHECK step |
| `ShadowEvaluator` | Policy validation before commit |
| `PerceptionSource` | Loop 0 PERCEIVE step |
| `StateStore` | Persistence (kernel provides SQLite/WAL impl) |

**State store keys** (written by kernel on `MetaCycleComplete`):
- `sie/identity/last_drift` — latest drift score
- `sie/policy/head` — current policy snapshot ID (updated when policy changes)
- `sie/meta/last_cycle` — latest meta-cycle ID

## Usage

```bash
# Build
cargo build

# Run all tests (78 tests: 73 unit + 5 integration)
cargo test

# Run specific module tests
cargo test self_model
cargo test critic
cargo test policy
cargo test identity
cargo test shadow

# Run integration tests (full Loop 0 lifecycle simulation)
cargo test --test loop0_lifecycle

# Clippy
cargo clippy -- -D warnings

# This is a library crate — add as a dependency:
# [dependencies]
# aros-sie = { path = "../aros-sie" }
```

## Test Suite

78 tests (73 unit + 5 integration) across 13 modules covering:
- Self-model Bayesian calibration and decay
- Critic output type classification
- Policy archive branching, forking, ancestry traversal
- Identity drift measurement and ceiling enforcement
- Permission asymmetry (tighten/loosen/never)
- Shadow testing with imp@k metric
- State store CRUD and prefix listing
- Event sourcing type serialization
- Full Loop 0 lifecycle integration (5 scenarios: happy path, failure-triggered policy update, drift ceiling blocking, shadow test validation, permission asymmetry enforcement)

## Tech Stack

- **Language:** Rust (Edition 2024)
- **Async runtime:** Tokio
- **Serialization:** serde + serde_json
- **Telemetry:** tracing
- **UUIDs:** uuid v4
- **Time:** chrono
- **Randomness:** rand (for shadow test sampling)

## License

Private — AROS-Lab
