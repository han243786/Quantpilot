# v4.16.0 backend.ops_governance.sandbox single leaf closeout continues split

> Batch: BE-001LO-03
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` remains open and must continue splitting.

After BE-001LO-02, the leaf owns the sandbox route facade, implementation body, compatibility runner exports, disk loader, metric helpers, replay-shape helper, proposal lookup, and embedded unit tests. This is too large and too semantically mixed to close as one leaf.

Likely next child candidates:

- `backend.ops_governance.sandbox.report_api`
- `backend.ops_governance.sandbox.verification_run`
- `backend.ops_governance.sandbox.metrics_verdict`
- `backend.ops_governance.sandbox.replay_shape`
- `backend.ops_governance.sandbox.comparison_metrics`
- `backend.ops_governance.sandbox.disk_loader`

BE-001LP-01 must choose one child and freeze it before any movement.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `backend.ops_governance.sandbox` has a named route facade and implementation owner. |
| parent_child_communication_kept | PASS | Parent ops governance calls the sandbox child facade; root compatibility bridge only exposes runner and loader. |
| equivalence_baseline_freezable | PASS | BE-001LO-01 froze route, runner, loader, metric, storage, DTO, AppState, and runtime compatibility boundaries. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | Route handlers, `run_sandbox_verification`, and `load_sandbox_report_from_disk` are distinct externally relevant boundaries. |
| state_machine_phase | PASS | Verification runner owns proposal status gate, replay window, report persistence, cache update, and evidence metric side effect. |
| strategy_branch | PASS | Route report access, verification execution, metric verdict, replay-shape comparison, and disk loader are separate governance branches. |
| independent_failure_mode | PASS | Route not-found, proposal status denial, metric verdict, storage quota, JSON parse, and path validation failures differ. |
| reuse_pressure | PASS | Runtime mutation already reuses runner and disk loader through the root compatibility bridge. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | Candidate children own durable behavior and distinct failure modes. |
| communication_cost_rises | NO | Child extraction should reduce mixed route/runner/metric/loader ownership while preserving parent bridge rules. |
| local_proof_missing | NO | Embedded unit tests exist for metric diff, verdict, and replay-shape behavior; compile and governance gates cover route wiring. |
| line_count_only | NO | Continue decision is driven by public API, reused runner/loader, state side effects, and failure modes. |

leaf_split_decision_result

`stop_split_false`

`backend.ops_governance.sandbox stop_split: false`.

The next recursive step returns to this node as a parent residual judgment and must select one child before code movement.

## Next Step

BE-001LP-01 backend.ops_governance.sandbox parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
