# v4.16.0 backend.strategy_config.diff.evidence_diff actual extraction complete

> Batch: BE-001II-02
> Node: `backend.strategy_config.diff.evidence_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff actual extraction complete.

Moved into `src/backend/strategy_config/diff/evidence_diff.rs`:

- `STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA`
- `build_strategy_config_evidence_diff_for_backtests`
- `StrategyConfigEvidenceDiffStatus`
- all evidence diff report schemas
- `StrategyConfigEvidenceCountChange`
- `StrategyConfigEvidenceFieldDiff`
- `StrategyConfigEvidenceFirstDivergence`
- `load_bound_backtest_for_evidence`
- `build_strategy_config_evidence_diff`
- machine trajectory, risk plane, execution capability, and metrics comparison helpers
- signature, first divergence, sorted unique, count map, diff count map, and JSON label helpers

`src/backend/strategy_config/diff.rs` is now a parent facade that declares
`artifact_diff` and `evidence_diff`, delegates route registration to
`artifact_diff`, and re-exports compatibility surfaces for
`strategy_config_api`. `artifact_diff` was not changed by this extraction.

No evidence diff schema version, missing backtest diagnostics, graph mismatch
diagnostics, v4 artifact diagnostics, frontend response shape, artifact diff,
graph storage, backtest storage, runtime persistence, AI proposal binding, or
release-transition behavior changed.

## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff/artifact_diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `evidence diff child moved`
- `artifact diff untouched`
- `diff parent facade only`

**Next step**:
BE-001IJ-01 backend.strategy_config.diff.evidence_diff single_leaf_closeout

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
