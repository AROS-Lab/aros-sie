# AROS Self-Improvement Engine (SIE)

The Self-Improvement Engine for AROS (Agent Runtime OS). Provides self-model calibration, meta-cognition, policy evolution, shadow testing, and identity enforcement for the Meta Loop (Loop 0).

## Architecture

```
Loop 0 Orchestrator (kernel)
  |
  |-- PERCEIVE ----------> sie::perceive::PerceptionEngine
  |-- SELF-MODEL UPDATE --> sie::self_model::SelfModelRegistry
  |-- CRITIQUE ----------> sie::critic::MetaCognitionEngine
  |-- POLICY REVISION ---> sie::policy::PolicyArchive
  |-- IDENTITY CHECK ----> sie::identity::IdentityGuard
  |-- PERSIST -----------> sie::persistence::StateStore
```

The SIE is a **library crate** consumed by the AROS kernel. The kernel's Loop 0 orchestrator calls SIE functions — the SIE does not own loop orchestration.

## Modules

### Self-Model (`src/self_model/`)
Bayesian calibration of agent capabilities using Beta distributions. Each capability is tracked as `Beta(alpha, beta)` — updated on task success/failure, with temporal decay to weight recent observations.

- `SelfModelRegistry` — concrete implementation of the `SelfModel` trait
- `BetaDistribution` — success/failure updates, confidence intervals, decay
- `ModelSnapshot` — serializable state for persistence and restore

### Critic (`src/critic/`)
Meta-cognition engine that evaluates task outcomes against self-model expectations and emits typed action recommendations.

**Six output types:**
| Output | When |
|--------|------|
| `PolicyUpdate` | Sustained low confidence suggests strategy change |
| `MemoryWrite` | Failure/degraded outcome should be recorded |
| `ToolAction` | A tool action (e.g., re-run test) is recommended |
| `Alert` | Consecutive failure streak exceeds threshold |
| `NoAction` | Observation is within expected bounds |
| `Experiment` | Very low confidence suggests A/B testing |

### Policy (`src/policy/`)
Branching evolution log for policy snapshots, organized as a DAG (not a linear log).

- `PolicyArchive` — DAG-based storage with fork-from-any-node (quality-diversity exploration)
- `PolicySnapshot` — versioned config state with feature vectors for identity comparison
- `SkillLibrary` — two-tier library: task-skills (domain-specific) + meta-skills (domain-general)
- `PolicyDiff` — comparison utilities between snapshots

### Identity (`src/identity/`)
Prevents the ship-of-Theseus problem by enforcing cumulative drift ceilings and permission asymmetry.

- `IdentityGuard` — cosine similarity between policy vectors, blocks changes exceeding ceiling
- `PermissionAsymmetry` — tighten=auto-approve, loosen=human-review, NEVER=always blocked
- Distance metrics for policy comparison

### Shadow (`src/shadow/`)
Validates policy changes against frozen historical data without production side effects.

- `ShadowTestPipeline` — replays frozen datasets through candidate policies
- `imp@k` metric — improvement at rank k compared to baseline
- `FrozenDataset` — serializable historical test cases

### Perceive (`src/perceive/`)
Aggregates telemetry signals (task outcomes, resource utilization) into a unified perception state for Loop 0's PERCEIVE step.

### Persistence (`src/persistence/`)
Trait-based state store abstraction with an in-memory implementation. The kernel provides the production SQLite/WAL backend.

### Telemetry (`src/telemetry/`)
OTLP-compatible tracing span definitions for all SIE operations.

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

# Build docs
cargo doc --no-deps --open
```

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

State store keys used by the kernel:
- `sie/identity/last_drift` — latest drift score
- `sie/policy/head` — current policy snapshot ID
- `sie/meta/last_cycle` — latest meta-cycle ID

## Tech Stack

- Rust (edition 2024)
- tokio, serde, serde_json, tracing, thiserror, uuid, chrono, rand

## License

Private — AROS-Lab
