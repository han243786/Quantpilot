# v4.16.0 backend.test_support single leaf closeout

> Batch: BE-001PB-01
> Node: `backend.test_support`
> Parent: `backend`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.test_support` is closed as a thin test-support route facade.

Decision:

`stop_split: true`

## Current White-Box Boundary

| Surface | Owner | Status |
| --- | --- | --- |
| `register_test_scenario_routes` | `src/backend/test_support.rs` -> `scenario` -> `api_test_scenario::register_test_scenario_routes` | Preserved. |
| Legacy backend tests | `src/tests_backend.rs` and integration test files | Preserved. |
| Test runner | `src/test_runner.rs` | Preserved. |
| Test asset retirement registry | `markdown/06-milestones/v4.16.0/05-测试资产汰换登记.md` | Preserved as deferred governance input. |

## Equivalence Evidence

The current leaf matches the earlier BE-001C-09 and BE-001E-08 boundary:

- `src/backend/test_support.rs` remains only a backend route facade;
- `src/backend/test_support/scenario.rs` remains a compatibility bridge to the original test scenario route owner;
- no legacy tests were removed;
- no integration test behavior, test runner behavior, or E2E cleanup scope changed;
- no production route owner was introduced.

## Split Decision Rules

The required leaf split rules were evaluated:

| Rule | Result |
| --- | --- |
| Public boundary | One route registration facade; no separate production public owner emerges. |
| State-machine phase | Not applicable; no runtime state transition owner lives here. |
| Strategy branch | Not applicable; test scenario routing is not a strategy family. |
| Independent failure mode | Deeper split would only separate a route facade from its compatibility bridge. |
| Communication cost | Further split would add parent mediation without improving proof quality. |

## Hard Boundaries

This closeout does not authorize:

- deleting or retiring old tests;
- starting E2E cleanup;
- changing test scenario schema;
- changing production route ownership;
- rewriting integration test semantics;
- release transition connection proposals.

## Parent Return

Return to the `backend` parent closeout.

All backend top-level residuals in the current recursive scope are now closed:

- `backend.interface_boundary`;
- `backend.runtime`;
- `backend.graph_compile`;
- `backend.capability`;
- `backend.strategy_config`;
- `backend.storage_security`;
- `backend.ops_governance`;
- `backend.app_state_wiring`;
- `backend.test_support`.

## Next Step

BE-001PC-01 `backend` parent_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
