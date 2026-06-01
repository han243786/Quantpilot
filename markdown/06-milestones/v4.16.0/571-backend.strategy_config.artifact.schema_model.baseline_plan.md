# v4.16.0 backend.strategy_config.artifact.schema_model equivalence baseline and extraction plan

> Batch: BE-001HS-01
> Node: `backend.strategy_config.artifact.schema_model`
> Parent: `backend.strategy_config.artifact`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.artifact.schema_model equivalence baseline and extraction plan.

## Equivalence Baseline

The schema model move must preserve:

- JSON field names, serde rename rules, defaults, and skip-serializing rules.
- `StrategyConfigArtifactRequest` accepted input fields.
- `StrategyConfigArtifact` output fields and digest input shape.
- Domain enum values: `market`, `observation`, `state_machine`, `risk`,
  `execution`, `evidence`, `ai_governance`, `snapshot`.
- Lifecycle/readiness enum values and all public finding/source/evidence/proposal
  binding shapes.
- Compatibility for residual `src/strategy_config_api.rs`, graph compare, and
  migration sender callers.

## Extraction Plan

BE-001HS-02 may create:

- `src/backend/strategy_config/artifact/schema_model.rs`

Move only these type definitions into that child:

- `StrategyConfigArtifactRequest`
- `StrategyConfigArtifact`
- `StrategyConfigSourceSummary`
- `StrategyConfigCapabilitySummary`
- `ConfigDomainStatus`
- `ConfigDomainId`
- `ConfigDomainLifecycle`
- `ConfigDomainReadiness`
- `ConfigSourceRef`
- `StrategyConfigFinding`
- `RuntimeBoundarySummary`
- `EvidenceAnchorInput`
- `EvidenceAnchor`
- `ProposalBindingInput`
- `ProposalBinding`

Allowed compatibility:

- Re-export schema model types from `backend.strategy_config.artifact`.
- Import those types in the residual root `src/strategy_config_api.rs`.
- Temporarily widen moved type and field visibility to `pub(crate)` while
  preflight/diff/evidence diff remain residual root code.

Forbidden changes:

- Do not move preflight report types, diff report types, evidence diff types, or
  builder functions in this batch.
- Do not change serde attributes, schema version constants, digest input, route
  registration, graph compare, migration sender, frontend schema, or runtime
  mutation behavior.
- Do not claim artifact or strategy_config is closed after this move.

## Boundary

**Real files**:
- `src/backend/strategy_config/artifact.rs`
- `src/backend/strategy_config/artifact/schema_model.rs`
- `src/strategy_config_api.rs`

**Markers**:
- `artifact schema_model baseline_frozen`
- `schema_model extraction plan_frozen`

**Next step**:
BE-001HS-02 backend.strategy_config.artifact.schema_model extract_closeout

---

## Gates

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot strategy_config --lib`
- `cargo test -p quantpilot graph_version_endpoints_list_load_and_restore_versions`
- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
