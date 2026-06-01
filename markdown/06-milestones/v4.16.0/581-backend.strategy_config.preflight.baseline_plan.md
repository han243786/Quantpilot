# v4.16.0 backend.strategy_config.preflight equivalence baseline and extraction plan

> Batch: BE-001HZ-01
> Node: `backend.strategy_config.preflight`
> Parent: `backend.strategy_config`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.preflight equivalence baseline and extraction plan.

This baseline freezes the preflight pocket before code movement. The next
extraction may move only the preflight endpoint, report schema, decision enum,
blocked-action schema, report builder, and migration sender helper value path
from `src/strategy_config_api.rs` into `src/backend/strategy_config/preflight.rs`.

Allowed movement:

- `STRATEGY_CONFIG_PREFLIGHT_SCHEMA`
- `register_strategy_config_preflight_route`
- `preflight_strategy_config`
- `build_strategy_config_preflight_value`
- `StrategyConfigPreflightReport`
- `PreflightDecision`
- `PreflightBlockedAction`
- `build_preflight_report`
- `blocked`

Compatibility exits that must remain available:

- `crate::strategy_config_api::build_strategy_config_preflight_value` for
  `src/migration_sender.rs`.
- Strategy config unit tests that directly assert preflight decisions.

Forbidden movement:

- `StrategyConfigDiffRequest`
- `StrategyConfigDiffReport`
- `build_diff_report`
- `build_strategy_config_version_diff`
- `build_strategy_config_evidence_diff_for_backtests`
- evidence diff schemas, structs, and helpers
- AI proposal binding validation
- artifact builder/schema/domain owner code
- runtime, graph version, frontend, or release-transition behavior

## Boundary

**Real files**:
- `src/backend/strategy_config/preflight.rs`
- `src/strategy_config_api.rs`
- `src/migration_sender.rs`

**Markers**:
- `preflight endpoint owner`
- `preflight report schema`
- `migration sender compatibility`

**Next step**:
BE-001HZ-02 backend.strategy_config.preflight extract_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot strategy_config --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
