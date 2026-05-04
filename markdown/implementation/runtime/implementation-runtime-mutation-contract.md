# Runtime Mutation Contract

This is the active contract source of truth for controlled runtime parameter
mutation. The planning worklist tracks delivery status; this document owns the
stable field contract, safety rules, evidence behavior, and Block 4 handoff
constraints.

## Scope

Runtime mutation covers operator-initiated parameter proposals, safe-window
checks, activation at explicit boundaries, rollback to ledger-backed parameter
versions, governed evidence events, frontend reader/display behavior, contract
snapshots, and health metrics.

Runtime mutation does not grant live execution authority by itself. Permission
boundary and capability context checks remain mandatory before proposal,
activation, or rollback.

## Field Ownership

| Surface | Owner | Source of truth | Update rule |
|---|---|---|---|
| Mutation request/record shape | Backend | `src/frontend_api_types.rs` | Add fields as typed structs/enums first, then update snapshot, reader, tests, and docs. |
| Mutation ledger persistence | Backend | `src/runtime_persistence.rs` | Ledger records are append-only. Never delete or rewrite activation/rollback history to express state. |
| Mutation API behavior | Backend | `src/runtime_api.rs` | Proposal, activation, safe-window denial, and rollback must validate capability context before mutation state changes. |
| Event envelope classification | Backend | `src/runtime_event_projection.rs` | Every `ParameterMutation*` event is `system` + `key` and must pass governed envelope validation. |
| Timeline/report projection | Backend + frontend | `src/runtime_response_mapping.rs`, `frontend/src/utils/runtimeTimeline.js`, `frontend/src/utils/runtimeEvidenceSummary.js` | Mutation lifecycle evidence must remain visible in timeline, replay, compact evidence, and report sections. |
| Frontend reader contract | Frontend | `frontend/src/utils/runtimeMutation.js` | UI consumes normalized states only; raw JSON is not the primary user surface. |
| Frontend display | Frontend | `frontend/src/components/RuntimeMutationPanel.jsx` | Display current, pending, denied, active, rollback, and failed states with disabled unsafe actions. |
| Contract snapshots | Tests | `tests/fixtures/runtime/mutation_contract_snapshot.json` | Snapshot drift is allowed only with an intentional contract update and matching doc change. |
| Health metrics | Backend | `/api/runtime/evidence/health` | Metrics are observational. They must not change user-visible mutation behavior. |

## Stable Status Values

- `proposed`
- `rejected`
- `activation_scheduled`
- `activated`
- `activation_failed`
- `safe_window_denied`
- `rollback_scheduled`
- `rolled_back`
- `rollback_failed`

## Safety Rules

- `immediate` activation is forbidden. Supported boundaries are
  `next_cycle_start`, `manual_pause`, and `sequence_cursor`.
- Safe-window evaluation is backend-owned. Frontend display may explain
  `safe_window_state`, but cannot authorize activation or rollback.
- Unsafe windows return `parameter_mutation_safe_window_denied`, persist the
  denied state, and emit `ParameterMutationSafeWindowDenied` as key evidence.
- Rollback targets must already exist in the mutation ledger for the same
  source and target pair.
- Rollback creates a new reverse mutation record with `rollback_of` and
  `rollback_target_parameter_version`. It must not delete or rewrite the
  original activation record.
- Historical event envelopes must preserve their original `parameter_version`.
  Activating or rolling back later versions must not repair valid prior
  envelopes into the new active version.

## Governed Events

All mutation events are retained key evidence:

- `ParameterMutationProposed`
- `ParameterMutationRejected`
- `ParameterMutationActivationScheduled`
- `ParameterMutationActivated`
- `ParameterMutationActivationFailed`
- `ParameterMutationSafeWindowDenied`
- `ParameterMutationRollbackScheduled`
- `ParameterMutationRolledBack`
- `ParameterMutationRollbackFailed`

Each event payload must include proposal identity, source identity, target
parameter, old/proposed parameter versions, actor, reason, governance, and any
available activation, safe-window, or rollback state.

## Health Metrics

`/api/runtime/evidence/health` exposes mutation counters alongside evidence
metrics:

- proposals created/rejected
- activations scheduled/applied/failed
- activation latency total and average
- safe-window denials
- rollback attempts/scheduled/applied/failed

These metrics are for operational visibility and alerting only.

## Update Checklist

When changing mutation behavior:

- update typed backend contracts first
- update event envelope classification for any new event type
- update frontend reader normalization and UI state labels
- update API/integration tests for proposal, activation, safe-window denial,
  rollback, timeline/replay/report evidence, and health metrics
- update `mutation_contract_snapshot.json`
- update this document and the v0.2.0 worklist
- verify all changed Markdown files decode as UTF-8

## Block 4 Handoff

Block 4 AI approval must build on this contract. AI may produce a proposal or
approval recommendation, but it must not bypass capability context validation,
permission boundary checks, safe-window evaluation, explicit activation
boundary, append-only ledger history, or governed evidence emission.
