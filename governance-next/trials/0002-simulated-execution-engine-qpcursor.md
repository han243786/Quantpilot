# QPCursor Trial 0002: simulated_execution_engine

> governance_next_trial: true
> legacy_governance_authority: preserved
> status: handoff_ready
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
| `status` | `handoff_ready` |
| `repo_baseline` | `master @ 201f0d4d`, milestone `v4.16.0`, legacy batch `BE-002EA-01` |
| `mode_stack` | `refactor: RM.R1_old_behavior_freeze`, `refactor: RM.R2_code_module_dependency_inventory` |
| `super_pipeline` | legacy recursive modularization, `recursive-high-speed-v2`, `terminal_leaf_control_v2` |
| `scope` | `MT=root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine`, `LEAF=qrpc_runtime/src/v4_simulated_execution.rs` |
| `interface_freeze` | No public API, route, schema, persistence owner, lock owner, release transition, or sibling horizontal link change. |
| `allowed_workset` | `qrpc_runtime/src/v4_runtime.rs`, `qrpc_runtime/src/v4_simulated_execution.rs`, planned simulated execution child file, legacy baseline docs, module tree rows, recursive state cursor. |
| `next_action` | Continue legacy step `BE-002EB-01` baseline_plan for simulated execution behavior before code movement. |
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
| `current_step` | `BE-002EB-01` |
| `current_phase` | `baseline_plan` |
| `next_recommended_child` | `root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine.baseline_plan` |
| `last_closed_child` | `root.contracts.runtime_support.v4_runtime_support.risk_execution_gate` |

## Evidence Captured

| Gate | Result |
| --- | --- |
| Terminal leaf control | WAVE, score 56, oversized over 800 LOC |
| `git diff --check` | Pending for the commit that records this trial |
| UTF-8 check | Pending for the commit that records this trial |
| full feature tree check | Pending for the commit that records this trial |
| matrix governance check | Pending for the commit that records this trial |

## Trial Judgment

This QPCursor is sufficient for handoff because it names the legacy authority, heat level, stop conditions, and next legal recursive action. It also exposes one governance quality issue: `WAVE` is ambiguous for an oversized G4 candidate and needs an escalation override rule before promote.
