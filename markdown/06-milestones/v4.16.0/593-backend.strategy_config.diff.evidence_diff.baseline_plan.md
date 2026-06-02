# v4.16.0 backend.strategy_config.diff.evidence_diff equivalence baseline and extraction plan

> Batch: BE-001II-01
> Node: `backend.strategy_config.diff.evidence_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff equivalence baseline and extraction plan.

This baseline freezes the evidence diff pocket before code movement. The next
extraction may move only backtest evidence diagnostics, v4 evidence artifact
comparison, evidence report schemas, and supporting evidence helper functions
out of `src/backend/strategy_config/diff.rs` into a child owner.

Allowed movement:

- `STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA`
- `build_strategy_config_evidence_diff_for_backtests`
- `StrategyConfigEvidenceDiffStatus`
- all `StrategyConfig*EvidenceDiff` report structs
- `StrategyConfigEvidenceCountChange`
- `StrategyConfigEvidenceFieldDiff`
- `StrategyConfigEvidenceFirstDivergence`
- `load_bound_backtest_for_evidence`
- `build_strategy_config_evidence_diff`
- `compare_machine_trajectory_evidence`
- `compare_risk_plane_evidence`
- `compare_execution_capability_evidence`
- `compare_evidence_metrics`
- `evidence_field`
- `stable_float`
- `evidence_status`
- machine trajectory / risk / execution capability signature helpers
- `first_divergence`
- `sorted_unique`
- `count_by`
- `diff_count_maps`
- `json_label`

Compatibility exits that must remain available:

- `crate::strategy_config_api::build_strategy_config_evidence_diff_for_backtests`
- `crate::strategy_config_api::StrategyConfigEvidenceDiffReport`
- test-only access to evidence comparison helpers used by existing
  `strategy_config_api` tests

Forbidden movement:

- `artifact_diff` child code
- `StrategyConfigDiffRequest`
- `StrategyConfigDiffReport`
- `build_strategy_config_version_diff`
- route-level `/api/v1/strategy-config/diff` handler
- graph version storage ownership
- backtest record storage ownership
- frontend response shape changes
- AI proposal binding, preflight, artifact, runtime, or release-transition
  behavior

## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/backend/strategy_config/diff/artifact_diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `evidence diff report boundary`
- `backtest evidence diagnostics`
- `artifact diff excluded`

**Next step**:
BE-001II-02 backend.strategy_config.diff.evidence_diff extract_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot strategy_config --lib`
- `cargo test -p quantpilot graph_version_endpoints_list_load_and_restore_versions`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
