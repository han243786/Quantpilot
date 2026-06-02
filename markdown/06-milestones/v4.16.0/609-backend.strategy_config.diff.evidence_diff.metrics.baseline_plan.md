# v4.16.0 backend.strategy_config.diff.evidence_diff.metrics equivalence baseline and extraction plan

> Batch: BE-001IU-01
> Node: `backend.strategy_config.diff.evidence_diff.metrics`
> Parent: `backend.strategy_config.diff.evidence_diff`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff.metrics equivalence baseline and extraction plan.

This baseline freezes the metrics evidence family before code movement. The next
extraction may move only the metrics report schema and direct field comparison
helpers into a child owner.

Allowed movement:

- `StrategyConfigEvidenceMetricsDiff`
- `StrategyConfigEvidenceFieldDiff`
- `compare_evidence_metrics`
- `evidence_field`
- `stable_float`

Allowed parent inputs:

- `StrategyConfigEvidenceDiffStatus`
- `evidence_status`

Planned target:

- `src/backend/strategy_config/diff/evidence_diff/metrics.rs`

Forbidden movement:

- machine trajectory child code
- risk plane child code
- execution capability child code
- shared count/divergence helpers
- `StrategyConfigEvidenceCountChange`
- `StrategyConfigEvidenceFirstDivergence`
- `StrategyConfigEvidenceDiffReport`
- `build_strategy_config_evidence_diff_for_backtests`
- `build_strategy_config_evidence_diff`
- `load_bound_backtest_for_evidence`
- artifact diff child code
- graph/backtest storage ownership
- frontend response shape changes
- release-transition behavior

## Boundary

**Real files**:
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`

**Markers**:
- `metrics report boundary`
- `summary field diff compatibility`
- `stable float formatting`

**Next step**:
BE-001IU-02 backend.strategy_config.diff.evidence_diff.metrics extract_closeout

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
