# v4.16.0 backend.strategy_config.diff actual extraction complete

> Batch: BE-001IC-02
> Node: `backend.strategy_config.diff`
> Parent: `backend.strategy_config`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.strategy_config.diff actual extraction complete.

Moved into `src/backend/strategy_config/diff.rs`:

- `/api/v1/strategy-config/diff` route ownership
- strategy config diff schema and endpoint report model
- graph-version artifact diff builder
- backtest evidence diff builder
- machine trajectory / risk plane / execution capability / metrics evidence
  diff models and helpers
- signature, count map, first divergence, JSON label, and domain map helpers

`src/strategy_config_api.rs` now keeps compatibility re-exports for graph
compile and frontend response callers, plus test-only imports for the existing
unit tests. No graph version storage, backtest record storage, runtime
persistence, frontend response shape, AI proposal binding, artifact, or
preflight behavior changed.

## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `diff endpoint moved`
- `graph version diff compatibility kept`
- `evidence diff compatibility kept`

**Next step**:
BE-001ID-01 backend.strategy_config.diff single_leaf_closeout

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
