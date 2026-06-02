# v4.16.0 backend.ops_governance.sandbox.verification_run single leaf closeout continues split

> Batch: BE-001LS-03
> Node: `backend.ops_governance.sandbox.verification_run`
> Parent: `backend.ops_governance.sandbox`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run` remains open and must continue splitting.

After BE-001LS-02, the runner owns the reusable verification boundary and still contains several distinct phases:

- proposal lookup and static-check gate;
- replay window generation;
- comparison/metric/verdict/warning calls;
- report assembly;
- sandbox report persistence commit.

Most phases are orchestration over already registered sandbox helpers, but the persistence commit owns a durable side-effect cluster and should be isolated before the runner is closed.

Likely next child candidates:

- `backend.ops_governance.sandbox.verification_run.report_commit`
- `backend.ops_governance.sandbox.verification_run.report_assembly`
- `backend.ops_governance.sandbox.verification_run.replay_window`
- `backend.ops_governance.sandbox.verification_run.proposal_gate`

BE-001LT-01 must choose one child and freeze it before any movement.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `verification_run` has a dedicated child file and public runner boundary. |
| parent_child_communication_kept | PASS | The runner uses sandbox parent-controlled helper boundaries. |
| equivalence_baseline_freezable | PASS | BE-001LS-01 froze runner ordering, side effects, compatibility callers, and helper ownership. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | `run_sandbox_verification` is reused by report API and runtime mutation through the root bridge. |
| state_machine_phase | PASS | The runner gates proposal status, computes replay evidence, persists a report, updates cache, and increments evidence metrics. |
| strategy_branch | PARTIAL | Most helper branches are already delegated, but persistence commit is still mixed into runner orchestration. |
| independent_failure_mode | PASS | Proposal denial, metric/comparison failure, storage quota failure, persistence IO failure, and cache/metric side effects differ. |
| reuse_pressure | PASS | Runtime mutation and report API both reuse the runner. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | `report_commit` owns quota, persistence, cache insert, and evidence metric side effects. |
| communication_cost_rises | NO | Isolating report commit reduces mixed orchestration and side-effect ownership. |
| local_proof_missing | PARTIAL | No dedicated runner side-effect test exists, so only the durable side-effect cluster should be split next. |
| line_count_only | NO | Continue decision is driven by storage/report side effects, not line count. |

leaf_split_decision_result

`stop_split_false`

`backend.ops_governance.sandbox.verification_run stop_split: false`.

The next recursive step returns to this node as a parent residual judgment and must select one child before code movement.

## Next Step

BE-001LT-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
