# v4.16.0 backend.strategy_config.diff.evidence_diff.machine_trajectory actual extraction complete

> Batch: BE-001IL-02
> Node: `backend.strategy_config.diff.evidence_diff.machine_trajectory`
> Parent: `backend.strategy_config.diff.evidence_diff`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff.machine_trajectory actual extraction complete.

Moved into `src/backend/strategy_config/diff/evidence_diff/machine_trajectory.rs`:

- `StrategyConfigMachineTrajectoryEvidenceDiff`
- `compare_machine_trajectory_evidence`
- `machine_trajectory_signature`
- `machine_terminal_state`
- `transition_hit_counts`

`src/backend/strategy_config/diff/evidence_diff.rs` now declares the child and
re-exports the report/comparison surface. Shared helper types and functions
(`StrategyConfigEvidenceDiffStatus`, count changes, first divergence,
`evidence_status`, `first_divergence`, `sorted_unique`, `diff_count_maps`, and
`count_by`) remain in the evidence_diff parent for sibling reuse.

No machine trajectory status, visited-state, transition-hit, terminal-state,
first-divergence, risk plane, execution capability, metrics, frontend response
shape, or graph version behavior changed.

## Boundary

**Real files**:
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff/evidence_diff/machine_trajectory.rs`
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`

**Markers**:
- `machine trajectory child moved`
- `shared helpers retained in evidence parent`
- `risk plane remains open`

**Next step**:
BE-001IM-01 backend.strategy_config.diff.evidence_diff.machine_trajectory single_leaf_closeout

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
