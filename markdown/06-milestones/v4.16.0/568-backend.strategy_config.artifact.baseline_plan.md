# v4.16.0 backend.strategy_config.artifact equivalence baseline and extraction plan

> Batch: BE-001HQ-01
> Node: `backend.strategy_config.artifact`
> Parent: `backend.strategy_config`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.artifact equivalence baseline and extraction plan.

This freezes the artifact owner before code movement. The current implementation
is still in `src/strategy_config_api.rs`; the child facade in
`src/backend/strategy_config/artifact.rs` only delegates route registration.

## Equivalence Baseline

**Inputs**:

- `StrategyConfigArtifactRequest` JSON from `/api/v1/strategy-config/artifact`.
- Graph/version callers that build strategy config diff through graph compare.
- Migration sender preflight caller that constructs `StrategyConfigArtifactRequest`.
- Current capability snapshot hash from `build_capability_response`.

**Outputs**:

- `StrategyConfigArtifact` with unchanged schema version, artifact id format,
  source digests, capability summary, config domains, runtime boundary, evidence
  anchors, proposal bindings, and `artifact_digest`.
- Existing error codes from canonical digest failures.
- No route path or response schema change.

**Current owner pockets**:

- Route registration and HTTP handler: `register_strategy_config_artifact_route`,
  `create_strategy_config_artifact`.
- Shared artifact schema: `StrategyConfigArtifactRequest`,
  `StrategyConfigArtifact`, source/capability/domain/runtime/evidence/proposal
  structs and enums.
- Shared artifact builders: `build_strategy_config_artifact`,
  `version_artifact_request`, `build_source_summary`,
  `build_capability_summary`, `build_runtime_boundary`,
  `normalize_evidence_anchors`, `normalize_proposal_bindings`,
  `build_config_domains`, all domain builders, `artifact_digest_input`,
  `digest_option_value`, `digest_for_value`, `infer_source_mode`, `non_empty`,
  and `finding`.

## Extraction Plan

BE-001HQ-02 may move only the artifact pocket into
`src/backend/strategy_config/artifact.rs`.

Allowed changes:

- Make `backend.strategy_config.artifact` own artifact route registration and
  artifact handler.
- Move artifact request/response/common domain structs and artifact helper
  functions into the artifact child.
- Widen visibility only to the minimum required `pub(crate)` surface for
  compatibility shims, preflight/diff residual code, graph compare, and migration
  sender callers.
- Keep `src/strategy_config_api.rs` as compatibility/residual owner for
  preflight, diff, evidence diff, and tests until those children are selected.

Forbidden changes:

- Do not move preflight decision logic, diff report logic, evidence diff logic,
  graph compare behavior, migration sender semantics, frontend types, or runtime
  mutation AI proposal checks.
- Do not create sibling horizontal links. Dependent code may use the old root
  shim or parent-mediated re-export until its own child extraction is selected.
- Do not claim `backend.strategy_config` is closed after artifact extraction.
- Do not introduce release transition or performance shortcut proposals.

## Boundary

**Real files**:
- `src/backend/strategy_config/artifact.rs`
- `src/strategy_config_api.rs`
- `src/migration_sender.rs`
- `src/backend/graph_compile/graph.rs`

**Markers**:
- `strategy_config artifact baseline_frozen`
- `artifact extraction plan_frozen`

**Next step**:
BE-001HQ-02 backend.strategy_config.artifact extract_closeout

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
