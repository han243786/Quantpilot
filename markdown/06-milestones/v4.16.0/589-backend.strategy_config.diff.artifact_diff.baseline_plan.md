# v4.16.0 backend.strategy_config.diff.artifact_diff equivalence baseline and extraction plan

> Batch: BE-001IF-01
> Node: `backend.strategy_config.diff.artifact_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.artifact_diff equivalence baseline and extraction plan.

This baseline freezes the artifact diff pocket before code movement. The next
extraction may move only the route-level artifact diff request/report and
graph-version artifact bridge out of `src/backend/strategy_config/diff.rs` into
a child owner. Evidence diff stays in the parent for the next recursive pass.

Allowed movement:

- `STRATEGY_CONFIG_DIFF_SCHEMA`
- `register_strategy_config_diff_route`
- `diff_strategy_config`
- `build_strategy_config_version_diff`
- `StrategyConfigDiffRequest`
- `StrategyConfigDiffReport`
- `StrategyConfigDigestChange`
- `StrategyConfigDomainChange`
- `build_diff_report`
- `domains_by_id`

Compatibility exits that must remain available:

- `crate::strategy_config_api::build_strategy_config_version_diff`
- `crate::strategy_config_api::StrategyConfigDiffReport`
- `backend.strategy_config.diff::register_routes`

Forbidden movement:

- `STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA`
- `build_strategy_config_evidence_diff_for_backtests`
- `StrategyConfigEvidenceDiffStatus`
- all evidence diff report structs
- `load_bound_backtest_for_evidence`
- evidence comparison, signature, metrics, count-map, JSON-label, and
  first-divergence helpers
- graph version storage ownership
- backtest record storage ownership
- frontend response shape changes
- AI proposal binding, artifact, preflight, runtime, or release-transition
  behavior

## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `artifact diff endpoint boundary`
- `graph version bridge compatibility`
- `evidence diff excluded`

**Next step**:
BE-001IF-02 backend.strategy_config.diff.artifact_diff extract_closeout

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
