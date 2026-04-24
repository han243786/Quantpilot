# QuantPilot Artifact Governance

## Purpose

This document is the P1 governance layer for persisted artifacts.
It defines:

- artifact manifest version rules
- retention defaults
- safe cleanup boundaries
- evidence required when artifact schemas change

This document does not introduce new runtime or backtest features.
It only governs the storage and maintenance of artifacts that already exist.

## Artifact scope

Current storage patterns in this repository include:

- `storage/graphs/*.json`
- `storage/graphs/*.qs`
- `storage/graphs/latest.json`
- `storage/runs/*.json`
- `storage/backtests/<backtest_id>/*` when present
- `storage/audit/*.json`
- `storage/test-*` directories created by tests and deterministic replay runs
- root-level storage logs such as `*.log`, `*.err.log`, `*.out.log`

## Schema version policy

Artifact families must carry explicit `schema_version` values.
The current repository already contains versioned artifacts such as:

- `quantpilot/reproducibility-manifest/v1`
- `quantpilot/backtest-spec/v1`
- `quantpilot/run-spec/v1`
- `quantpilot/strategy-artifact/v1`
- `quantpilot/compile-artifact/v1`

### Versioning rules

- Any structural artifact change must preserve or intentionally bump `schema_version`.
- A version bump requires:
  - a doc update in this file
  - updated fixtures or snapshots
  - a migration note or compatibility statement
- Do not silently repurpose an existing `schema_version`.
- Backward-compatible additive fields may remain in the same version only when old readers can safely ignore them.

## Manifest expectations

The reproducibility manifest is the artifact anchor for backtest bundles.
At minimum it should keep:

- manifest identity: `manifest_id`, `schema_version`, `created_at_ms`
- run identity: `backtest_id`, `graph_id`, `compile_id`
- compile identity: `protocol_name`, `config_hash`
- summary and account snapshot
- embedded compile and run specs or stable references to them

### Current governance expectation

- `manifest.json` is the entry point for backtest artifact inspection.
- Companion files such as `metrics.json`, `event_log.json`, `equity_curve.json`, and `trade_ledger.json` are secondary views.
- Frontend pages should prefer manifest-driven or artifact-first reads instead of assuming a legacy summary payload.

## Retention defaults

Retention is conservative during beta.
The default policy is 鈥渒eep production-facing records, clean ephemeral test material.鈥?
| Storage area | Default retention | Cleanup default |
|---|---|---|
| `storage/graphs/*.json` | retain | never touched by cleanup script |
| `storage/graphs/latest.json` | retain | never touched by cleanup script |
| `storage/graphs/*.qs` | retain | never touched by cleanup script |
| `storage/runs/*.json` | retain during beta | not deleted by default |
| `storage/backtests/**` | retain during beta | not deleted by default |
| `storage/audit/*.json` | retain during beta | not deleted by default |
| `storage/test-*` | ephemeral | eligible for cleanup |
| `storage/*.log`, `storage/*.err.log`, `storage/*.out.log` | operational | optional cleanup only |

### Why this default

- graph and run records still help debugging and regression review
- backtest bundles are the current artifact-first truth for history pages
- test-generated artifacts accumulate quickly and create noise without adding long-term product value

## Cleanup policy

The repository now includes a safe cleanup entry point:

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\cleanup-artifacts.ps1`

Default behavior:

- dry-run only
- scoped to `storage/`
- targets ephemeral `storage/test-*` directories older than the chosen threshold
- does not touch production-facing graph, run, or backtest artifacts

Optional behavior:

- `-IncludeLogs` also includes root-level storage logs
- `-Mode execute` performs the deletion after the same path checks pass

## Safety rules

- Cleanup must resolve all targets under the repository `storage/` directory.
- Cleanup must never recurse outside the declared storage root.
- Cleanup must never delete `storage/graphs/latest.json`.
- Cleanup must never delete production-facing graphs, runs, or backtests in the default mode.
- Cleanup output must list every target before deletion in dry-run mode.

## Change checklist

When artifact schemas or retention behavior change, update:

- this governance document
- fixture samples or snapshots
- any artifact-reading frontend tests
- any cleanup tooling assumptions

## References

- [implementation-support-matrix.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-support-matrix.md)
- [implementation-testing-module.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-testing-module.md)
- [Current Status And Release State](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)

