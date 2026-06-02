# v4.16.0 backend.strategy_config.diff.evidence_diff.risk_plane equivalence baseline and extraction plan

> Batch: BE-001IO-01
> Node: `backend.strategy_config.diff.evidence_diff.risk_plane`
> Parent: `backend.strategy_config.diff.evidence_diff`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff.risk_plane equivalence baseline and extraction plan.

This baseline freezes the risk plane evidence family before code movement. The
next extraction may move only the risk-plane report schema and its direct
comparison/signature helper into a child owner. Shared helper types and
functions used by execution capability / metrics remain in the evidence_diff
parent unless a later parent residual judgment selects them.

Allowed movement:

- `StrategyConfigRiskPlaneEvidenceDiff`
- `compare_risk_plane_evidence`
- `risk_decision_signature`

Allowed parent inputs:

- `StrategyConfigEvidenceDiffStatus`
- `StrategyConfigEvidenceCountChange`
- `StrategyConfigEvidenceFirstDivergence`
- `evidence_status`
- `first_divergence`
- `count_by`
- `diff_count_maps`
- `non_empty`

Planned target:

- `src/backend/strategy_config/diff/evidence_diff/risk_plane.rs`

Forbidden movement:

- machine trajectory child code
- execution capability report/comparison/signature helpers
- metrics report/comparison helpers
- shared helper types/functions
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
- `risk plane report boundary`
- `allow reject decision counts`
- `risk reason first divergence compatibility`

**Next step**:
BE-001IO-02 backend.strategy_config.diff.evidence_diff.risk_plane extract_closeout

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
