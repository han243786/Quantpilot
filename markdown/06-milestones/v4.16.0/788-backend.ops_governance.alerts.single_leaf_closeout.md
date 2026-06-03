# v4.16.0 backend.ops_governance.alerts single leaf closeout continues split

> Batch: BE-001MR-03
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts` completed its first physical extraction, but it is not small enough to close as a final leaf.

Decision:

`stop_split: false`

Current owned files:

- `src/backend/ops_governance/alerts.rs`
- `src/backend/ops_governance/alerts/handlers.rs`
- `src/alert_engine.rs` as startup compatibility bridge

The leaf now owns:

- alert route facade registration;
- default alert rules;
- alert list/read handlers;
- alert acknowledgment flow;
- alert trigger and auto-recovery flow;
- metric predicate checks;
- alert firing persistence;
- startup initialization target.

## Split Gate Result

Further splitting is required by the recursive split rules:

| Rule | Result | Evidence |
| --- | --- | --- |
| Concrete owner exists? | Yes. Rule catalog, route handlers, trigger checks, recovery cleanup, and persistence are distinct owners. |
| Independent IO or state failure mode? | Yes. Acknowledge write path, trigger write path, disk persistence, resolved cleanup, and read-only listing fail differently. |
| Parent-child communication would improve? | Yes. Extracting tested static rules and focused flows reduces the size of the alert parent handler file. |
| Local proof would improve? | Yes. Existing rule catalog tests can move with the first child. |
| Line count only? | No. The split is driven by owner boundaries and tests, not only size. |

## Candidate Queue

| Candidate | Boundary | Initial decision |
| --- | --- | --- |
| `backend.ops_governance.alerts.rule_catalog` | `default_alert_rules` and existing rule catalog tests. | Prefer first because it is pure and already tested. |
| `backend.ops_governance.alerts.acknowledge_flow` | Acknowledge route, state transition, not_found mapping, persistence call. | Keep queued. |
| `backend.ops_governance.alerts.trigger_engine` | Trigger loop, deduplication, predicate dispatch, auto-recovery, cleanup. | Keep queued. |
| `backend.ops_governance.alerts.predicate_checks` | Metric-specific alert predicate helpers. | Consider after trigger_engine baseline. |
| `backend.ops_governance.alerts.persistence` | `persist_alert_firing` and storage quota / atomic write. | Consider after write flows are isolated. |

## Hard Boundaries

Further alerts splitting must not move:

- snapshots route or handler owner;
- runbook route or handler owner;
- chaos route or handler owner;
- closed hotswap internals;
- closed sandbox internals;
- AppState owner or lock order;
- DTO schema owner in `src/frontend_api_types.rs`;
- runtime/capability/storage security internals;
- release transition policy.

## Next Step

BE-001MS-01 backend.ops_governance.alerts parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers::tests`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
