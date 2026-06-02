# v4.16.0 backend.ops_governance.sandbox.report_api single leaf closeout stops further split

> Batch: BE-001LQ-03
> Node: `backend.ops_governance.sandbox.report_api`
> Parent: `backend.ops_governance.sandbox`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.report_api` is closed as a single leaf after BE-001LQ-02 moved route registration and route handlers into a dedicated child file.

Current owned file:

- `src/backend/ops_governance/sandbox/report_api.rs`

The leaf owns:

- sandbox report API route registration;
- GET sandbox report handler;
- POST sandbox request handler;
- memory-first report lookup call site;
- disk fallback call site;
- runner call site.

It does not own runner, loader, metric, replay-shape, comparison, runtime mutation, or root bridge internals.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `backend.ops_governance.sandbox.report_api` has a dedicated child file and route owner. |
| parent_child_communication_kept | PASS | Report API calls runner and loader through sandbox parent exports. |
| equivalence_baseline_freezable | PASS | BE-001LQ-01 froze route paths, handlers, call sites, and non-owned boundaries. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PARTIAL | GET and POST handlers exist, but both are tiny API adapters over parent-owned helpers. |
| state_machine_phase | NO | Verification lifecycle remains in `run_sandbox_verification`, not report API. |
| strategy_branch | NO | GET and POST are entrypoints into one report API surface. |
| independent_failure_mode | PARTIAL | GET disk miss and POST runner error differ, but each delegates to parent-owned helpers. |
| reuse_pressure | NO | No separate reuse pressure exists inside report API. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | YES | Splitting GET and POST would create tiny endpoint files without durable owners. |
| communication_cost_rises | YES | More files would add bridge overhead around two short route adapters. |
| local_proof_missing | YES | No dedicated route-level sandbox API smoke test exists. |
| line_count_only | YES | Further split would be driven mostly by function count. |

leaf_split_decision_result

`stop_split_true`

`backend.ops_governance.sandbox.report_api stop_split: true`.

The next recursive step returns to `backend.ops_governance.sandbox` parent residual judgment.

## Next Step

BE-001LR-01 backend.ops_governance.sandbox parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
