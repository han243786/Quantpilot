# v4.16.0 backend.strategy_config.diff equivalence baseline and extraction plan

> Batch: BE-001IC-01
> Node: `backend.strategy_config.diff`
> Parent: `backend.strategy_config`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff equivalence baseline and extraction plan.

This baseline freezes the diff pocket before code movement. The next extraction
may move strategy config diff endpoint/report code and backtest evidence diff
code into `src/backend/strategy_config/diff.rs`, while preserving old
`crate::strategy_config_api::*` compatibility exports for callers that still
enter through root API types.

Allowed movement:

- `STRATEGY_CONFIG_DIFF_SCHEMA`
- `STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA`
- `register_strategy_config_diff_route`
- `diff_strategy_config`
- `build_strategy_config_version_diff`
- `build_strategy_config_evidence_diff_for_backtests`
- `StrategyConfigDiffRequest`
- `StrategyConfigDiffReport`
- `StrategyConfigEvidenceDiffStatus`
- all strategy config diff/evidence diff report structs
- `build_diff_report`
- `load_bound_backtest_for_evidence`
- `build_strategy_config_evidence_diff`
- machine trajectory, risk plane, execution capability, metrics, signature,
  count-map, first-divergence, domain-map, and JSON-label helpers

Compatibility exits that must remain available:

- `crate::strategy_config_api::build_strategy_config_version_diff`
- `crate::strategy_config_api::build_strategy_config_evidence_diff_for_backtests`
- `crate::strategy_config_api::StrategyConfigDiffReport`
- `crate::strategy_config_api::StrategyConfigEvidenceDiffReport`

Forbidden movement:

- artifact owner code
- preflight owner code
- AI proposal binding validation
- graph version storage/state ownership
- backtest record storage ownership
- runtime mutation/evidence persistence ownership
- frontend response shape changes
- release-transition behavior

## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `diff endpoint owner`
- `graph version diff compatibility`
- `evidence diff compatibility`

**Next step**:
BE-001IC-02 backend.strategy_config.diff extract_closeout

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
