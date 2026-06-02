# v4.16.0 backend.strategy_config.diff.evidence_diff.metrics actual extraction complete

> Batch: BE-001IU-02
> Node: `backend.strategy_config.diff.evidence_diff.metrics`
> Parent: `backend.strategy_config.diff.evidence_diff`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff.metrics actual extraction complete.

Moved into `src/backend/strategy_config/diff/evidence_diff/metrics.rs`:

- `StrategyConfigEvidenceMetricsDiff`
- `StrategyConfigEvidenceFieldDiff`
- `compare_evidence_metrics`
- `evidence_field`
- `stable_float`

`src/backend/strategy_config/diff/evidence_diff.rs` now declares and re-exports
the child while retaining evidence report assembly and shared helper ownership.
No route, frontend, graph/backtest storage, or release-transition behavior
changed.

## Boundary

**Real files**:
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff/evidence_diff/metrics.rs`
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`

**Markers**:
- `metrics report boundary moved`
- `summary field diff compatibility kept`
- `stable float formatting moved`
- `shared helpers retained in evidence parent`

**Next step**:
BE-001IV-01 backend.strategy_config.diff.evidence_diff.metrics single_leaf_closeout

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
