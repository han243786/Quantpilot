# v4.16.0 backend.strategy_config.diff.evidence_diff.machine_trajectory equivalence baseline and extraction plan

> Batch: BE-001IL-01
> Node: `backend.strategy_config.diff.evidence_diff.machine_trajectory`
> Parent: `backend.strategy_config.diff.evidence_diff`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff.machine_trajectory equivalence baseline and extraction plan.

This baseline freezes the machine trajectory evidence family before code
movement. The next extraction may move only the machine trajectory report schema
and its direct comparison/signature helpers into a child owner. Shared helper
types and functions used by risk plane / execution capability / metrics remain
in the evidence_diff parent unless a later parent residual judgment selects
them.

Allowed movement:

- `StrategyConfigMachineTrajectoryEvidenceDiff`
- `compare_machine_trajectory_evidence`
- `machine_trajectory_signature`
- `machine_terminal_state`
- `transition_hit_counts`

Allowed parent inputs:

- `StrategyConfigEvidenceDiffStatus`
- `StrategyConfigEvidenceCountChange`
- `StrategyConfigEvidenceFirstDivergence`
- `evidence_status`
- `first_divergence`
- `sorted_unique`
- `diff_count_maps`

Forbidden movement:

- risk plane report/comparison/signature helpers
- execution capability report/comparison/signature helpers
- metrics report/comparison helpers
- `StrategyConfigEvidenceDiffReport`
- `build_strategy_config_evidence_diff_for_backtests`
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
- `src/frontend_api_types.rs`

**Markers**:
- `machine trajectory report boundary`
- `transition hit signature`
- `first divergence compatibility`

**Next step**:
BE-001IL-02 backend.strategy_config.diff.evidence_diff.machine_trajectory extract_closeout

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
