# v4.16.0 backend.strategy_config.diff.evidence_diff.execution_capability equivalence baseline and extraction plan

> Batch: BE-001IR-01
> Node: `backend.strategy_config.diff.evidence_diff.execution_capability`
> Parent: `backend.strategy_config.diff.evidence_diff`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff.execution_capability equivalence baseline and extraction plan.

This baseline freezes the execution capability evidence family before code
movement. The next extraction may move only the execution capability report
schema, direct comparison/signature helper, and the local JSON label helper used
by runtime mode / capability / source labels into a child owner.

Allowed movement:

- `StrategyConfigExecutionCapabilityEvidenceDiff`
- `compare_execution_capability_evidence`
- `execution_capability_signature`
- `json_label`

Allowed parent inputs:

- `StrategyConfigEvidenceDiffStatus`
- `StrategyConfigEvidenceCountChange`
- `StrategyConfigEvidenceFirstDivergence`
- `evidence_status`
- `first_divergence`
- `count_by`
- `diff_count_maps`

Planned target:

- `src/backend/strategy_config/diff/evidence_diff/execution_capability.rs`

Forbidden movement:

- machine trajectory child code
- risk plane child code
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
- `execution capability report boundary`
- `runtime capability source status counts`
- `json label helper follows execution capability`

**Next step**:
BE-001IR-02 backend.strategy_config.diff.evidence_diff.execution_capability extract_closeout

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
