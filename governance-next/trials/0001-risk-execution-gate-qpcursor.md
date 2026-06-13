# QPCursor Trial 0001: risk_execution_gate

> historical_trial: true
> legacy_governance_mode: archived_reference_after_promotion
> status: handoff_ready_evidence
> created_from_commit: `58773907`

This trial was captured before `GOV-GOVERNANCE-NEXT-PROMOTION-01`. After promotion it is evidence that QPCursor handoff works, not an active authority boundary.

## Authority Boundary

| Layer | Status | Rule |
| --- | --- | --- |
| Legacy governance | archived_reference | `markdown/00-matrix-governance/recursive-state.json` remains a temporary cursor store. |
| Governance next | authoritative | QPCursor handoff is now the default governance view. |
| Conflict resolution | governance-next wins | Any conflict is resolved through `governance-next/05-authoritative-operating-model.md`. |

## Short Cursor

```text
QPC://v4.16.0/RM:R6+R7/GH:G4/FFT:root.contracts.runtime_support/MT:root.contracts.runtime_support.v4_runtime_support.risk_execution_gate/LEAF:qrpc_runtime/src/v4_runtime/risk_execution_gate.rs
```

## Long Cursor

| Field | Value |
| --- | --- |
| `cursor_version` | `qpcursor-trial-v0` |
| `cursor_id` | `QPC-TRIAL-0001-risk-execution-gate` |
| `status` | `handoff_ready` |
| `repo_baseline` | `master @ 58773907`, milestone `v4.16.0`, legacy batch `BE-002DZ-01` |
| `mode_stack` | `refactor: RM.R6_module_tree_sync`, `refactor: RM.R7_regression_verification` |
| `super_pipeline` | legacy recursive modularization, `recursive-high-speed-v2`, `terminal_leaf_control_v2` |
| `scope` | `MT=root.contracts.runtime_support.v4_runtime_support.risk_execution_gate`, `LEAF=qrpc_runtime/src/v4_runtime/risk_execution_gate.rs` |
| `interface_freeze` | No public API, route, schema, persistence owner, lock owner, release transition, or sibling horizontal link change. |
| `allowed_workset` | The risk gate leaf, v4 runtime parent facade, legacy milestone closeout docs, module tree rows, recursive state cursor. |
| `next_action` | Continue legacy step `BE-002DZ-02` single leaf closeout, run terminal leaf control, then return to `v4_runtime_support` parent residual judgment. |
| `stop_if` | Any need to change runtime state-machine semantics, Risk Plane contract, ExecutionMachine contract, persistence, route/schema, or cross-child direct calls. |

## Heat Trigger

| Signal | Result |
| --- | --- |
| Runtime v4 state-machine execution gate | G4 |
| Risk Plane decision behavior | G4 |
| Execution capability decision behavior | G4 |
| New Rust child file | G2 minimum, raised by runtime semantics |
| Public API change | none |
| Release transition | not active; AI must not propose |

Final heat: `G4 architecture heat`.

## Legacy State Mapping

| Legacy field | Value |
| --- | --- |
| `current_parent` | `root.contracts.runtime_support.v4_runtime_support.risk_execution_gate` |
| `current_step` | `BE-002DZ-02` |
| `current_phase` | `single_leaf_closeout` |
| `next_recommended_child` | `root.contracts.runtime_support.v4_runtime_support.risk_execution_gate.single_leaf_closeout` |
| `last_closed_extraction` | `BE-002DZ-01` |

## Evidence Captured

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check -p qrpc-runtime` | PASS |
| `cargo test -p qrpc-runtime v4 -- --nocapture` | PASS, 32 passed |
| `cargo check -p quantpilot` | PASS |
| `git diff --check` | PASS |
| `tools/check-utf8.ps1` | PASS |
| `tools/check-full-feature-tree.ps1` | PASS |
| `tools/check-matrix-governance.ps1` | PASS |

## Trial Judgment

This QPCursor sample can be used by a new agent without reading chat history because it identifies:

1. The authoritative legacy cursor.
2. The exact module tree coordinate.
3. The active heat level.
4. The allowed and forbidden workset.
5. The next legal recursive action.
6. The evidence already gathered.

Trial limitation: this is a wrapper only. It does not promote `governance-next` and does not replace the legacy cursor.
