# v4.16.0 backend.ops_governance.alerts.predicate_checks single leaf closeout stops further split

> Batch: BE-001MZ-03
> Node: `backend.ops_governance.alerts.predicate_checks`
> Parent: `backend.ops_governance.alerts`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.predicate_checks` is closed as a final predicate dispatch leaf.

Decision:

`stop_split: true`

The child now owns one alert predicate evaluation contract:

- rule-name dispatch for alert predicates;
- evidence metric counter predicates;
- user-scoped operational state predicates;
- storage watermark predicate;
- unknown-rule fallback to `false`.

## Split Gate Result

| Rule | Result | Evidence |
| --- | --- | --- |
| Concrete owner exists? | Yes, but already singular. | One parent-mediated alert predicate contract exposed through `should_fire_alert`. |
| Independent IO or state failure mode? | Partial only. | `check_storage_watermark` has storage IO, while the rest are read-only predicate branches; extracting one helper now would fragment dispatch without a stronger local proof surface. |
| Parent-child communication would improve? | No. | Deeper split would add private fan-out modules under the same rule-name dispatcher while trigger_engine still needs the parent bridge. |
| Local proof would improve? | No. | Current direct tests cover rule catalog only; deeper predicate grouping would still be proven mainly by compile and existing alerts handler tests. |
| Line count only? | Mostly yes. | Further split would be evidence/user/storage helper grouping, not a new public, handler, persistence, or state-machine owner. |

## Closed Boundary

Closed child:

`backend.ops_governance.alerts.predicate_checks`

Remaining alerts residual queue:

- `backend.ops_governance.alerts.persistence`;
- startup initialization residual under the alerts parent;
- list/read route residual under the alerts parent if a later proposal proves a useful owner boundary.

## Hard Boundaries

Future alerts batches must not reopen `predicate_checks` unless a new explicit proposal proves a real owner boundary beyond the current predicate dispatch contract.

Trigger orchestration and persistence implementation remain outside this child and must continue through parent-mediated bridges until their own baselines are frozen.

## Next Step

BE-001NA-01 backend.ops_governance.alerts parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
