# v4.16.0 backend.strategy_config.diff.artifact_diff actual extraction complete

> Batch: BE-001IF-02
> Node: `backend.strategy_config.diff.artifact_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.strategy_config.diff.artifact_diff actual extraction complete.

Moved into `src/backend/strategy_config/diff/artifact_diff.rs`:

- `/api/v1/strategy-config/diff` route owner
- `STRATEGY_CONFIG_DIFF_SCHEMA`
- `register_routes`
- `register_strategy_config_diff_route`
- `diff_strategy_config`
- `build_strategy_config_version_diff`
- `StrategyConfigDiffRequest`
- `StrategyConfigDiffReport`
- `StrategyConfigDigestChange`
- `StrategyConfigDomainChange`
- `build_diff_report`
- `domains_by_id`

`src/backend/strategy_config/diff.rs` now declares `artifact_diff` and
re-exports `build_strategy_config_version_diff` plus
`StrategyConfigDiffReport` for the existing `strategy_config_api`
compatibility surface. Evidence diff remains in the parent file for the next
recursive pass.

No artifact diff schema version, graph-version compare behavior, route path,
frontend response shape, backtest evidence diff behavior, graph storage,
runtime persistence, AI proposal binding, or release-transition behavior
changed.

## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/backend/strategy_config/diff/artifact_diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `artifact diff child moved`
- `route bridge kept`
- `evidence diff remains parent residual`

**Next step**:
BE-001IG-01 backend.strategy_config.diff.artifact_diff single_leaf_closeout

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
