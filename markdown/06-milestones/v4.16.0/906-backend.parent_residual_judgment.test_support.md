# v4.16.0 backend parent residual judgment selects test_support

> Batch: BE-001PA-01
> Node: `backend`
> Selected child: `backend.test_support`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend` returns to its last top-level residual after `backend.app_state_wiring` closed.

Decision:

`next_child: backend.test_support`

## Closed Backend Children

Already closed in the current recursive scope:

- `backend.interface_boundary`;
- `backend.runtime`;
- `backend.graph_compile`;
- `backend.capability`;
- `backend.strategy_config`;
- `backend.storage_security`;
- `backend.ops_governance`;
- `backend.app_state_wiring`.

## Open Backend Residuals

| Residual | Status |
| --- | --- |
| `backend.test_support` | Selected next. Existing facade owns only test scenario route registration. |

## Selection Rationale

`backend.test_support` is selected because it is the only remaining backend top-level residual:

- `src/backend/test_support.rs` delegates route registration to `scenario`;
- `src/backend/test_support/scenario.rs` delegates to `api_test_scenario::register_test_scenario_routes`;
- legacy tests, test runner, E2E cleanup, and test asset retirement remain explicitly deferred.

## Hard Boundaries

The next `backend.test_support` closeout must not:

- delete or retire legacy tests;
- rewrite integration test behavior;
- change production route ownership;
- change test scenario response schema;
- start E2E cleanup;
- start release transition logic.

## Next Step

BE-001PB-01 `backend.test_support` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
