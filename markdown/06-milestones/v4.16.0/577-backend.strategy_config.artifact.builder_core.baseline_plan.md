# v4.16.0 backend.strategy_config.artifact.builder_core equivalence baseline and extraction plan

> Batch: BE-001HW-01
> Node: `backend.strategy_config.artifact.builder_core`
> Parent: `backend.strategy_config.artifact`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.artifact.builder_core equivalence baseline and extraction plan.

## Equivalence Baseline

The builder core move must preserve:

- Artifact id format: `strategy_config_<16 hex chars>`.
- Default `strategy_id`, `strategy_version`, `source_mode`, runtime mode, and
  capability source behavior.
- Source digest calculation, including QS source wrapper digest.
- Capability snapshot current/stale/safe_fallback status behavior.
- Runtime boundary normalization, including legacy `live` to `PaperActual`.
- Evidence anchor and proposal binding normalization defaults.
- Artifact digest input shape and canonical JSON SHA-256 output.
- Graph version diff and migration sender preflight compatibility.

## Extraction Plan

BE-001HW-02 may create:

- `src/backend/strategy_config/artifact/builder_core.rs`

Move only these artifact construction items into that child:

- `STRATEGY_CONFIG_ARTIFACT_SCHEMA`
- `build_strategy_config_artifact`
- `version_artifact_request`
- `build_source_summary`
- `build_capability_summary`
- `build_runtime_boundary`
- `normalize_evidence_anchors`
- `normalize_proposal_bindings`
- `artifact_digest_input`
- `digest_option_value`
- `digest_for_value`
- `infer_source_mode`
- `non_empty`

Allowed compatibility:

- Re-export `build_strategy_config_artifact`, `version_artifact_request`, and
  `non_empty` from `backend.strategy_config.artifact` while residual preflight,
  diff, and evidence diff still live in `src/strategy_config_api.rs`.
- Keep old `strategy_config_api::build_strategy_config_artifact`,
  `strategy_config_api::StrategyConfigArtifactRequest`, and
  `strategy_config_api::EvidenceAnchorInput` compatibility paths until their
  callers are migrated by their own selected leaves.

Forbidden changes:

- Do not move preflight report logic, diff/evidence diff logic, test fixtures, or
  route registration in this batch.
- Do not change digest inputs, default values, runtime boundary semantics, graph
  compare behavior, migration sender behavior, frontend schema, or runtime
  mutation behavior.

## Boundary

**Real files**:
- `src/backend/strategy_config/artifact.rs`
- `src/backend/strategy_config/artifact/builder_core.rs`
- `src/backend/strategy_config/artifact/domain_projection.rs`
- `src/backend/strategy_config/artifact/schema_model.rs`
- `src/strategy_config_api.rs`

**Markers**:
- `builder_core baseline_frozen`
- `builder_core plan_frozen`

**Next step**:
BE-001HW-02 backend.strategy_config.artifact.builder_core extract_closeout

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
