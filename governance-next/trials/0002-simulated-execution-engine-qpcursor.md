# QPCursor Trial 0002: simulated_execution_engine

> governance_next_trial: true
> legacy_governance_authority: preserved
> status: actual_extraction_complete
> created_from_commit: `201f0d4d`

This trial wraps the second recursive step after `risk_execution_gate` closeout.

## Authority Boundary

| Layer | Status | Rule |
| --- | --- | --- |
| Legacy governance | authoritative | `markdown/00-matrix-governance/recursive-state.json` remains the source of truth. |
| Governance next | trial wrapper | This file adds a QPCursor handoff view only. |
| Conflict resolution | legacy wins | Any conflict falls back to the legacy recursive protocol. |

## Short Cursor

```text
QPC://v4.16.0/RM:R1+R2/GH:G4/FFT:root.contracts.runtime_support/MT:root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine/LEAF:qrpc_runtime/src/v4_simulated_execution.rs
```

## Long Cursor

| Field | Value |
| --- | --- |
| `cursor_version` | `qpcursor-trial-v0` |
| `cursor_id` | `QPC-TRIAL-0002-simulated-execution-engine` |
| `status` | `actual_extraction_complete` |
| `repo_baseline` | `master @ 201f0d4d`, milestone `v4.16.0`, legacy batch `BE-002EA-01` |
| `mode_stack` | `refactor: RM.R1_old_behavior_freeze`, `refactor: RM.R2_code_module_dependency_inventory` |
| `super_pipeline` | legacy recursive modularization, `recursive-high-speed-v2`, `terminal_leaf_control_v2` |
| `scope` | `MT=root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine`, `LEAF=qrpc_runtime/src/v4_simulated_execution.rs` |
| `interface_freeze` | No public API, route, schema, persistence owner, lock owner, release transition, or sibling horizontal link change. |
| `allowed_workset` | `qrpc_runtime/src/v4_runtime.rs`, `qrpc_runtime/src/v4_simulated_execution.rs`, planned simulated execution child file, legacy baseline docs, module tree rows, recursive state cursor. |
| `next_action` | Continue legacy step `BE-002EC-02` single_leaf_closeout for simulated execution behavior. |
| `stop_if` | Any need to change trading semantics, order lifecycle behavior, fill/accounting math, Risk Plane contract, ExecutionMachine contract, persistence, route/schema, or cross-child direct calls. |

## Heat Trigger

| Signal | Result |
| --- | --- |
| Runtime v4 simulated order lifecycle | G4 |
| Trading execution semantics | G4 |
| Fill/accounting/asset curve behavior | G4 |
| Oversized implementation leaf | G2 minimum, raised by runtime semantics |
| Public API change | none planned |
| Release transition | not active; AI must not propose |

Final heat: `G4 architecture heat`.

## Legacy State Mapping

| Legacy field | Value |
| --- | --- |
| `current_parent` | `root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine` |
| `current_step` | `BE-002EC-02` |
| `current_phase` | `single_leaf_closeout` |
| `next_recommended_child` | `root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine.single_leaf_closeout` |
| `last_closed_child` | `root.contracts.runtime_support.v4_runtime_support.risk_execution_gate` |

## Evidence Captured

| Gate | Result |
| --- | --- |
| Terminal leaf control | `split_decision=CONTINUE`, `governance_packaging=precision_single_leaf`, `final_decision=PRECISION` |
| Precision baseline | Completed in BE-002EB-01 |
| Actual extraction | Completed in BE-002EC-01 working tree; final gate results recorded by the authoritative legacy commit |
| `git diff --check` | Pending for BE-002EC-01 commit |
| UTF-8 check | Pending for BE-002EC-01 commit |
| full feature tree check | Pending for BE-002EC-01 commit |
| matrix governance check | Pending for BE-002EC-01 commit |

## Trial Judgment

This QPCursor is sufficient for handoff because it names the legacy authority, heat level, stop conditions, and next legal recursive action.

Post GOV-GOVERNANCE-NEXT-OPTIMIZATION-01 judgment:

- The earlier `WAVE` ambiguity is resolved by separating `split_decision` from `governance_packaging`.
- `qrpc_runtime/src/v4_simulated_execution.rs` now evaluates as `split_decision=CONTINUE`, `governance_packaging=precision_single_leaf`, `final_decision=PRECISION`.
- The next baseline must be a precision baseline before code movement.
- BE-002EB-01 baseline exposed a path-registration guard: planned future files must not be written into `module-tree.md` as checkable paths before they exist.
- BE-002EC-01 extraction kept transition matching and event payload validation outside `simulated_execution_engine`, preventing helper leakage into the new child.
