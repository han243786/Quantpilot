# QuantPilot v0.2.0 Upgrade Worklist

## Purpose

This document turns the v0.2.0 upgrade research into an execution checklist.
It is the active planning queue for the second-stage work after the first
governance seed has landed.

The implementation order is intentionally conservative:

1. Establish the runtime governance foundation.
2. Build the evidence surface on top of that foundation.
3. Add controlled runtime mutation.
4. Add AI-assisted proposal and approval flows.
5. Harden contract-first delivery, tests, and operations.

## Major Work Blocks

| Block | Scope | Primary outcome | Status |
|---|---|---|---|
| 1. Runtime governance foundation | capabilities, event envelope, version/revision, permission boundary | Every run/event can prove which capability, strategy version, parameter generation, deployment revision, and permission model it used | Completed |
| 2. Evidence surface | unified timeline, replay API, report lifecycle, retained key events | Users can inspect, replay, compact, and export a trustworthy evidence chain | Completed |
| 3. Controlled runtime mutation | parameter streaming, activation boundaries, rollback, safe window controller | Runtime changes can be proposed, activated, observed, and reverted without mixing states inside one decision chain | Detailed |
| 4. AI proposal and approval chain | AI proposal ledger, static checks, sandbox replay, approval, audit | AI can advise and submit candidates but cannot directly alter live runtime state | Detailed |
| 5. Contract-first delivery and operational hardening | OpenAPI/AsyncAPI, mock, Pact, e2e, metrics, health checks, alerts | Frontend, backend, docs, tests, and runtime stay aligned through machine-checkable contracts | Planned |

## Block 1: Runtime Governance Foundation

### Goal

Make capability boundaries, event identity, version identity, deployment identity,
and permission boundaries explicit runtime facts. This block is the prerequisite
for timeline, replay, report compaction, parameter hot update, safe windows, and
AI approval.

### Current Seed

The first seed is present:

- `/api/capabilities` exposes a schema hash, chain stages, versioning policy, and permission boundary.
- Runtime events expose an `envelope`.
- Run and backtest detail responses expose a `governance` snapshot.
- Backtest manifests retain the governance snapshot.
- Frontend diagnostics and backtest detail views read governance through a shared normalizer and display the governance identity without requiring raw JSON inspection.

The remaining work in this block is to harden that seed into a stable contract
instead of a best-effort response shape.

### Deliverables

| Deliverable | Description | Acceptance |
|---|---|---|
| Capability contract v1 | Stable capability schema with `schema_hash`, chain stages, supported sections, versioning model, and permission boundary | Backend fixture, frontend default capability, safe fallback, and runtime response all agree |
| Event envelope v1 | Stable event envelope attached to every runtime/backtest event | Every event has `event_id`, `event_type`, `stage`, `run_id`, `sequence_no`, timestamps, version ids, `capability_hash`, `deployment_revision`, and `retention_class` |
| Runtime governance snapshot v1 | Run/backtest-level snapshot describing capability hash, strategy version, parameter version, deployment revision, and permission model | Snapshot is persisted with saved runs/backtests and survives reload |
| Deployment revision model | Deterministic revision id from strategy version, compile id, parameter version, and capability hash | Same input produces same revision; changed capability/parameter produces a new revision |
| Permission boundary model | Explicit policy for execution ownership, live execution, AI writes, plugin network default, and non-execution order access | Runtime and frontend can fail closed when policy is missing or restrictive |
| Compatibility defaults | Safe deserialize defaults for old run/backtest records | Old persisted records still load with `unknown`/safe governance defaults |
| Contract tests | Focused tests for capability fixture, run event envelope, saved backtest manifest, and fallback capability | Tests fail if fields drift or disappear |

### Backend Tasks

- [x] Add capability governance fields to `/api/capabilities`.
- [x] Add runtime `governance` snapshot to run/backtest records and detail responses.
- [x] Add event `envelope` to projected frontend runtime events.
- [x] Persist governance snapshot in backtest reproducibility manifests.
- [x] Split capability schema construction into a small contract module so hashing, response building, and runtime snapshots cannot diverge.
- [x] Replace the temporary FNV-style schema hash with the repo's canonical JSON SHA-256 helper where dependency boundaries allow it.
- [x] Add a startup/runtime guard that rejects new runs when capability hash is absent or malformed.
- [x] Add an explicit `CapabilitySnapshotTaken` runtime/system event at run start.
- [x] Add an explicit `SecurityViolationDetected` event shape for future permission failures.
- [x] Add versioned enums for event `stage`, `retention_class`, and permission policies instead of free-form strings.
- [x] Backfill governance defaults in load paths with a helper that records whether a record was legacy/defaulted.

### Concrete Development Items And Acceptance Conditions

#### P0

P0 items are release blockers for the governance foundation. Without them, later
timeline, replay, mutation, and AI work would be built on unstable evidence.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 1.1 | Centralize capability contract construction | Done. Move capability schema assembly, governance sections, and hash generation behind one backend helper/module. `/api/capabilities` and runtime governance snapshots must call the same source. | Changing any supported mode/module/symbol/stage/policy changes one canonical hash. A unit test proves response hash and runtime snapshot hash are identical for the same capability payload. |
| 1.2 | Canonicalize capability hashing | Done. Replace the temporary FNV hash with `qrpc_core::canonical_json_sha256_digest` for capability hash and deployment revision inputs. Hash only stable contract/governance identity fields, not request time or display-only noise. Frontend backend fixture and local default capability now use the same canonical digest; safe fallback keeps the explicit `safe-fallback` sentinel because it is not a trusted capability boundary. | `schema_hash` uses `sha256:<digest>`. Reordered internal map construction does not change the digest. Fixture test fails if contract fields change without fixture update. |
| 1.3 | Define typed capability policy enums | Done. Backend capability permission policies now use typed enums for AI write policy and boundary access policy. Runtime event envelopes now use typed stage and retention-class enums that serialize to the existing contract strings. `PortfolioUpdated` is mapped to `fill` instead of the previous chain-external `position` stage. Frontend capability normalization validates permission values and downgrades unknown or missing policy values to `disabled`/`deny`/`false`. | Invalid policy/stage/retention strings cannot be produced by backend builders. Frontend tests verify unknown values fall back to safest behavior. |
| 1.5 | Harden event envelope generation | Done. Runtime/backtest creation now validates the generated event list before response. Run detail, run replay, backtest artifact event log, backtest replay, and SSE tests assert complete envelopes with sequence, run id, timestamps, governance ids, stage, and retention class. Legacy-loaded records repair missing envelope fields with safe governance defaults before exposure. | Tests cover detail response, replay response, and SSE stream. No exposed event has empty `event_id`, `run_id`, `stage`, `capability_hash`, or `deployment_revision`. |
| 1.6 | Validate sequence and stage invariants | Done. Added a backend event-envelope validator that rejects empty event identity, unknown event types, mismatched envelope ids/types/run ids, non-contiguous sequence numbers, mismatched typed stage, mismatched retention class, missing governance identity, timestamp drift, and governance hash/revision drift. New run and backtest creation call the validator after envelope attachment. | Validation unit tests reject duplicate sequence numbers, missing envelope ids, unknown event types, mismatched stage, and mismatched key retention. New run/backtest integration tests pass validation before response. |
| 1.7 | Persist governance across all artifact paths | Done. Saved run JSON, saved backtest manifests, transient backtest spill/reload, and fresh-app reload paths preserve the same governance snapshot. Backtest artifact normalization rebuilds legacy/defaulted artifact views when governance or event envelopes need repair so response digests describe the exposed artifact content. Legacy run/backtest records without governance or event envelopes load with safe defaults. | Save/reload tests assert `governance` equality across original detail, saved artifact, reloaded artifact, and manifest. Legacy records load with safe defaults. |
| 1.9 | Enforce fail-closed runtime guard | Done. Runtime write requests now carry `capability_context` with `schema_hash` and `permission_boundary`. `test-run`, `backtest`, and `backtest-sweep` validate the context before record/session creation; missing context, malformed/non-canonical hash, stale hash, or permission-boundary mismatch returns `capability_boundary_violation`. | `runtime_write_rejects_missing_capability_context_without_creating_run` proves a missing policy returns a structured error and creates no run; history/list endpoints remain readable. Guard unit tests cover missing context, malformed hash, and boundary mismatch. |
| 1.12 | Frontend fail-closed capability fallback | Done. Capability action gating now treats loading, safe fallback, cache fallback, missing capability snapshot, malformed `schema_hash`, missing `permission_boundary`, and boundary-policy mismatch as unsafe for compile/run/backtest/sweep actions. Runtime requests include the trusted capability context only after the frontend boundary check passes. | Store tests prove compile, simulation, and backtest actions are blocked before network calls in unsafe capability states and expose diagnostics reasons; valid capability state still sends `capability_context` in runtime start payloads. |

#### P1

P1 items make the foundation operationally useful and prepare the evidence
surface, but they can land after the P0 contract is stable.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 1.4 | Add capability snapshot event | Done. Run and backtest event streams now prepend a `CapabilitySnapshotTaken` event before business/runtime events. The event carries `capability_hash`, capability `schema_version`, `permission_boundary_model_version`, chain stages, and runtime mode, then receives the standard envelope as `stage=system` and `retention_class=key`. SSE completion counts now use the record event count so the snapshot is included consistently, and the SSE contract test asserts the first streamed runtime event is the snapshot with the expected envelope. | First event window for every new run/backtest contains a capability snapshot event before business decisions. Replay and diagnostics can locate it by event type. |
| 1.8 | Add legacy/default provenance | Done. Runtime governance snapshots now include `governance_source`, normalized as `current_runtime` for newly produced in-memory records, `loaded_manifest` for records materialized from saved/transient artifacts, and `legacy_default` when old records are backfilled with safe governance defaults. Run/backtest load paths centralize provenance normalization before repairing event envelopes or artifact manifests. | Loading old records does not fail. API responses distinguish real loaded governance from defaulted upgrade governance. Tests cover old run/backtest JSON without governance and saved reload paths that switch from `current_runtime` to `loaded_manifest`. |
| 1.10 | Introduce permission violation event shape | Done. Added a `SecurityViolationDetected` frontend runtime-event builder for future permission failures. The payload contains actor, attempted action, denied policy, module key, reason code, and trace id; the envelope is attached through the same runtime envelope helper as other events. | Builder test proves the event has `stage=system`, `severity=Error`, `retention_class=key`, governance identity, and `reason_code`. |
| 1.11 | Frontend governance reader utility | Done. Added `runtimeGovernance` utility to normalize runtime/backtest governance snapshots from runtime state, backtest manifests, or event envelopes. Missing or partial governance falls back to restrictive defaults: no live execution, AI writes disabled, plugin network denied, and non-execution order access denied. | Unit tests cover complete governance, partial governance, missing governance, manifest/event-envelope resolution, and display-row metadata. Components use this utility instead of open-coded response reads. |
| 1.13 | Surface governance in diagnostics | Done. Runtime diagnostics projection now carries governance rows, diagnostics panels show governance identity, and backtest detail manifest cards show capability hash, deployment revision, strategy/parameter versions, permission model, AI write policy, and governance source. Long hashes are shortened visually with full values retained in title metadata. | UI tests verify governance identity is visible in runtime diagnostics and backtest detail views without inspecting raw JSON. |

#### P2

P2 items reduce future drift and documentation debt. They should not block the
first runtime governance foundation, but they should be completed before the
next major work block is considered fully hardened.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 1.14 | Document contract ownership | Done. Added the active Runtime Governance Contract and linked it from runtime, governance, planning, and overview indexes. It identifies capability contract, event envelope, runtime governance snapshot, deployment revision, permission boundary, and diagnostics display ownership. | Docs index links to the active contract. The worklist now tracks progress, while the formalized field ownership and update checklist live in the active contract doc. |

### Block 1 Completion Gate

Block 1 can be marked complete only when these checks are all true:

- Capability response, frontend default capability, backend fixture, and safe fallback all include schema/hash/versioning/permission fields.
- New run, saved run, transient backtest, saved backtest, backtest manifest, replay response, and SSE stream expose governance and event envelopes consistently.
- Capability hash and deployment revision are deterministic and covered by tests.
- Missing or malformed permission boundary fails closed for new write/run actions.
- Legacy persisted records load successfully and are explicitly marked as defaulted/legacy.
- Diagnostics expose enough governance identity for a user to answer: which capability boundary, strategy version, parameter version, deployment revision, and permission model produced this event?

### Frontend Tasks

- [x] Extend `DEFAULT_CAPABILITIES` with schema, chain, versioning, and permission fields.
- [x] Extend safe fallback capabilities so failure mode is visibly restrictive.
- [x] Update backend capability fixture.
- [x] Surface capability hash and permission mode in diagnostics or runtime detail views.
- [x] Treat missing `permission_boundary` as unsafe fallback for run/compile actions.
- [ ] Use event `envelope.stage` and `envelope.retention_class` as the preferred timeline grouping inputs once the timeline block starts.
- [x] Add a frontend utility for reading governance snapshots without reaching into raw response objects in each component.
- [ ] Add UI-copy guardrails that prevent live execution or AI write support claims unless the capability policy allows them.

### Data Contract Sketch

Capability response:

```json
{
  "api_version": "quantpilot-capabilities/v1",
  "schema_version": "quantpilot/capabilities-schema/v1",
  "schema_hash": "sha256:<digest>",
  "chain_stages": ["data", "intent", "agent", "risk", "execution", "fill"],
  "versioning": {
    "model_version": "quantpilot/versioning-model/v1",
    "strategy_version_source": "frontend_runtime_config.metadata.version",
    "parameter_version_policy": "immutable_generation_pointer",
    "deployment_revision_policy": "strategy_version_plus_compile_id_plus_capability_hash"
  },
  "permission_boundary": {
    "model_version": "quantpilot/permission-boundary/v1",
    "execution_owner_module": "builtin.execution.paper",
    "live_execution_allowed": false,
    "ai_write_policy": "proposal_only",
    "plugin_network_default": "deny",
    "non_execution_order_access": "deny"
  }
}
```

Runtime write request capability context:

```json
{
  "capability_context": {
    "schema_hash": "sha256:<digest>",
    "permission_boundary": {
      "model_version": "quantpilot/permission-boundary/v1",
      "execution_owner_module": "builtin.execution.paper",
      "live_execution_allowed": false,
      "ai_write_policy": "proposal_only",
      "plugin_network_default": "deny",
      "non_execution_order_access": "deny"
    }
  },
  "runtime_config": {}
}
```

Runtime event envelope:

```json
{
  "event_id": "evt_001",
  "event_type": "RiskDecisionProduced",
  "stage": "risk",
  "run_id": "run_001",
  "sequence_no": 42,
  "occurred_at_ms": 1760000000000,
  "ingested_at_ms": 1760000000000,
  "trace_id": "trace_001",
  "module_key": "builtin.risk.global",
  "strategy_version": "1.0.0",
  "parameter_version": "config:<hash>",
  "deployment_revision": "sha256:<digest>",
  "capability_hash": "sha256:<digest>",
  "mode": "paper",
  "severity": "Info",
  "retention_class": "key",
  "reason_code": "APPROVED",
  "payload_version": 1
}
```

Runtime governance snapshot:

```json
{
  "schema_version": "quantpilot/runtime-governance/v1",
  "governance_source": "current_runtime",
  "capability_hash": "sha256:<digest>",
  "strategy_version": "1.0.0",
  "parameter_version": "config:<hash>",
  "deployment_revision": "sha256:<digest>",
  "permission_boundary": {
    "model_version": "quantpilot/permission-boundary/v1",
    "execution_owner_module": "builtin.execution.paper",
    "live_execution_allowed": false,
    "ai_write_policy": "proposal_only",
    "plugin_network_default": "deny",
    "non_execution_order_access": "deny"
  }
}
```

Security violation event payload:

```json
{
  "event_type": "SecurityViolationDetected",
  "severity": "Error",
  "payload": {
    "actor": {
      "actor_id": "actor_001",
      "display_name": "Operator"
    },
    "attempted_action": "runtime.start_live",
    "denied_policy": "live_execution_allowed",
    "module_key": "builtin.execution.paper",
    "reason_code": "LIVE_EXECUTION_DENIED",
    "trace_id": "trace_001"
  },
  "envelope": {
    "stage": "system",
    "retention_class": "key",
    "reason_code": "LIVE_EXECUTION_DENIED"
  }
}
```

### Test Checklist

- [x] Capability response serializes governance sections.
- [x] Backend fixture matches capability response.
- [x] Frontend support matrix accepts governance fields.
- [x] Runtime run detail includes governance and event envelope.
- [x] Backtest artifact projection still works with governance fields.
- [x] Full `api_backtest` integration suite passes.
- [x] Saved backtest reload asserts manifest governance equals detail governance.
- [x] Legacy run/backtest JSON without governance loads with safe defaults.
- [x] Capability hash drift test proves two different capability payloads do not share a hash.
- [x] Permission fallback test proves missing/failed capabilities disables write/run actions.
- [x] SSE stream test verifies event envelopes are present in streamed events.
- [x] Security violation builder test verifies system/key/error event shape.
- [x] Frontend governance reader tests verify complete, partial, missing, manifest, and envelope-derived governance.
- [x] Runtime diagnostics and backtest detail UI tests verify visible governance identity.
- [x] Runtime Governance Contract documents source-of-truth ownership for capability contract, event envelope, governance snapshot, deployment revision, permission boundary, and diagnostics display.

### Acceptance Criteria

Block 1 is complete when all of the following are true:

- Every new run and backtest has a governance snapshot.
- Every runtime/backtest event exposed to the frontend has an envelope.
- `schema_hash`, `capability_hash`, and `deployment_revision` are deterministic.
- Missing capability or missing permission policy fails closed for new runtime actions.
- Saved artifacts preserve governance data across reload.
- Frontend uses the governance fields for at least diagnostics and fallback decisions.
- Tests cover current records, saved records, streamed records, and legacy defaults.

### Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Hash construction diverges between response and runtime snapshot | Replay cannot prove the runtime matched the advertised capability boundary | Centralize capability hash construction |
| Free-form string policies drift | Frontend and backend disagree about safety meaning | Promote stages, retention classes, and policies to typed enums |
| Old persisted records lack governance | History pages or replay may fail after upgrade | Default safely and label legacy records as defaulted |
| Governance metadata becomes UI-only | Evidence chain remains incomplete | Persist snapshots in run records, backtest manifests, and event envelopes |

### Next Block Entry Criteria

Do not start the unified timeline/replay/report lifecycle block until Block 1 has
at least these stable inputs:

- canonical event envelope
- deterministic governance snapshot
- persisted capability hash
- typed retention class
- test fixture proving saved artifact reload preserves governance

## Block 2: Evidence Surface

### Goal

Build the user-facing evidence layer on top of the runtime governance foundation.
This block must let users inspect what happened, replay the event chain,
understand which facts are retained as key evidence, and produce/export reports
without reconstructing trust from raw runtime JSON.

Block 2 does not change runtime decisions. It organizes, filters, replays,
summarizes, and exports the evidence already produced by governed runs and
backtests.

### Inputs From Block 1

Block 2 assumes these facts are stable:

- every exposed event has an envelope
- envelope `stage` and `retention_class` are typed contract strings
- run/backtest detail has a normalized governance snapshot
- saved artifacts preserve governance and event envelopes across reload
- legacy/defaulted records are readable and explicitly marked by
  `governance_source`

If any of these inputs drift, Block 2 work must stop and the Runtime Governance
Contract must be repaired first.

### Deliverables

| Deliverable | Description | Acceptance |
|---|---|---|
| Unified timeline contract | A single normalized timeline item shape for live runs, saved runs, backtests, replay windows, and report inputs | Same event renders with the same stage, severity, retention class, governance identity, summary, and sequence metadata across live, detail, replay, and report surfaces |
| Replay window hardening | Replay APIs and frontend readers consistently expose ordered windows by sequence, stage, retention, and cursor/page metadata | Users can replay run/backtest windows without missing or duplicating sequence numbers |
| Retained key-event index | Backend and frontend can identify `retention_class=key` events and system governance events for compact evidence views | Capability snapshots, security violations, risk decisions, executions, fills, and portfolio updates remain available in compact mode |
| Evidence timeline UI | Workspace/detail pages show a scannable timeline grouped by envelope stage and filterable by severity, retention, and module | Users can move from summary to event detail without inspecting raw JSON |
| Report lifecycle model | Reports have explicit lifecycle state, source event window, governance identity, source artifact ids, and failure reason | Report state survives reload and never claims readiness before source evidence is complete |
| Report artifact persistence | Generated reports are saved as governed artifacts linked to run/backtest ids and source event ranges | Saved reports can be listed, opened, exported, and traced back to the exact evidence chain |
| Evidence compaction | Large event logs can produce a compact key-evidence view without losing governance/system facts | Compact view keeps all key events and records the compaction policy |
| Contract tests | Backend, frontend, and integration tests prove timeline/replay/report surfaces agree on the same governed evidence | Tests fail if sequence, stage, retention, governance identity, or report lifecycle fields drift |

### Backend Tasks

- [x] Define a `TimelineItem` response contract derived from `FrontendRuntimeEvent` plus envelope metadata.
- [x] Add a shared timeline projection helper for run detail, backtest detail, replay, and future report generation.
- [x] Harden replay pagination around `sequence_no`, `cursor`, `limit`, `stage`, `severity`, and `retention_class` filters.
- [x] Add a retained key-event index for runs and backtests.
- [x] Add compact evidence projection that preserves all key/system events.
- [x] Define report lifecycle states and persisted report metadata.
- [x] Add report create/list/detail endpoints for run/backtest evidence reports.
- [x] Add report export endpoints for generated evidence report artifacts.
- [x] Persist report artifacts with governance snapshot, source ids, source sequence range, and generation policy.
- [x] Detect saved report source drift before list/detail/export responses claim readiness.
- [x] Add API contract snapshot coverage for timeline, replay, compact evidence, and report lifecycle fields.
- [x] Add evidence health and cleanup endpoints for report/replay/compact observability and transient report output cleanup.
- [x] Add legacy-safe report/timeline defaults for old records that can only be repaired to governed defaults.
- [x] Add backend tests for sequence windows, filters, compact evidence, saved report reload, and lifecycle failures.

### Frontend Tasks

- [x] Add a timeline reader utility that normalizes live events, persisted events, and replay responses into one UI shape.
- [x] Replace ad hoc event-list grouping with envelope `stage` and `retention_class` where timeline views are used.
- [x] Add a timeline component with stage grouping, severity filter, retention filter, module filter, and sequence navigation.
- [x] Add replay controls that consume backend cursor/page metadata instead of slicing local arrays.
- [x] Surface retained key events as a compact evidence mode.
- [x] Add report lifecycle UI states: not requested, generating, ready, failed, expired, and source changed.
- [x] Add report save/export/reveal actions where report artifacts exist.
- [x] Show governance identity on report detail/export surfaces.
- [x] Add compact evidence summary cards for report review surfaces.
- [x] Add retention-aware report loading summary that prefers compact/key evidence over raw logs.
- [x] Add an E2E evidence walkthrough for governed timeline, replay paging, compact summary mode, and report lifecycle.
- [x] Add UI tests for run timeline, backtest timeline, replay pagination, compact mode, and report lifecycle.
- [x] Preserve UTF-8 Chinese labels in timeline/report UI and avoid raw JSON as the primary evidence surface.

### Concrete Development Items And Acceptance Conditions

#### P0

P0 items create the minimum trustworthy evidence chain. They must land before
parameter hot update, safe windows, or AI approval use timeline/replay/report
surfaces as audit evidence.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 2.1 | Define unified timeline item contract | Done. Added backend `RuntimeTimelineItem` / `RuntimeTimelineGovernanceIdentity` / `RuntimeTimelineCompactability` contract types plus a governed-event projection helper. Added frontend `runtimeTimeline` reader that normalizes governed events, replay wrappers, already shaped timeline items, and legacy event fallbacks into the same timeline shape. Required fields cover event id/type, sequence, occurred/ingested time, stage, retention class, severity, module key, node id, summary, reason code, governance identity, payload version, and compactability marker. | Backend serialization tests prove stage, retention, compactability, sequence, governance, and payload version are present. Frontend unit tests prove live/persisted event, replay wrapper, already-shaped item, and legacy fallback inputs normalize into one contract. |
| 2.2 | Centralize timeline projection | Done. `RuntimeTimelineItem` projection is centralized through `timeline_item_from_event` / `timeline_items_from_events` and is now reused by run detail, backtest detail, and replay responses. Replay keeps the legacy `events` wrapper for compatibility while adding the governed `timeline` window from the same source events. The frontend timeline reader now prefers backend `timeline` data on detail/replay responses and falls back to legacy events only when needed. | Run detail and backtest detail return timeline data that matches replay output for the same sequence window. Integration tests compare replay timeline items against detail timeline items by sequence, event id, and wrapper event metadata. |
| 2.3 | Harden replay pagination and filters | Done. Replay requests now accept `sequence_cursor`, `stage`, `severity`, `retention_class`, `module_key`, and `key_only`. Responses expose `source_event_count`, filtered `total_events`, `filters`, `sequence_cursor`, `previous_sequence_cursor`, and `next_sequence_cursor` while preserving old `cursor`, `next_cursor`, `checkpoints`, and `events` fields. Invalid sequence/offset cursors return `bad_replay_cursor`. | Integration tests prove ordered replay windows, detail/replay timeline equality, key/retention filters preserving sequence order, and structured errors for invalid sequence cursors. |
| 2.4 | Build retained key-event index | Done. Run and backtest detail responses now expose `retained_key_event_index`, built from the shared `RuntimeTimelineItem` projection rather than a second event DTO. The policy retains every `retention_class=key` item and explicitly keeps system governance events such as `CapabilitySnapshotTaken` and `SecurityViolationDetected`. Frontend timeline utilities can normalize backend indexes or build the same index from live/detail timeline data. | Backend unit tests prove key evidence and system governance events survive while debug-only items are excluded. Run/backtest integration tests prove detail responses expose the index and every retained entry maps back to the shared timeline. Frontend tests prove the reader builds the same index shape. |
| 2.5 | Add compact evidence projection | Done. Run and backtest detail responses now expose `compact_evidence` with `projection_version`, `policy_version`, source/retained/dropped counts, dropped counts by stage and retention, governance identity, and compact entries. The projection is derived from the shared timeline plus retained key-event index, so compact mode does not copy or reinterpret raw runtime logs. | Backend unit tests prove summary/debug events are dropped with stage/retention counts while retained key entries and governance identity survive. Run/backtest integration tests prove detail responses expose compact evidence whose retained entries match the key-event index and whose retained+dropped counts equal source events. |
| 2.6 | Add frontend timeline reader utility | Done. The frontend `runtimeTimeline` utility now normalizes live events, saved detail timeline items, replay windows, retained key indexes, and compact evidence projections into one UI shape. It prefers backend `timeline` / `compact_evidence` contracts and falls back to governed events only when older responses are missing projections. | Unit tests cover live/persisted events, replay responses, backend projected timeline, retained key index, compact evidence projection, and legacy fallbacks. |
| 2.7 | Add first governed timeline UI | Done. Added `GovernedTimelinePanel` and mounted it in runtime diagnostics and backtest detail surfaces. The panel groups by envelope stage, filters by severity/retention/module, shows compact retained/source counts, opens selected event detail, and displays governance identity without requiring raw JSON inspection. | UI tests verify stage grouping, key-event filtering, governance identity, selected event detail, and backtest detail integration. |
| 2.8 | Lock replay UI to backend windows | Done. `EventReplaySection` now requests replay windows with backend `sequence_cursor` metadata first and only falls back to legacy offset `cursor` when sequence metadata is absent. Load, next, previous, and checkpoint actions all use returned backend window metadata; the UI shows the sequence window start rather than deriving the primary window from local arrays. | UI tests prove load/next/previous window navigation follows returned `sequence_cursor` / `next_sequence_cursor` / `previous_sequence_cursor` metadata and keeps sequence numbers stable across page changes. |
| 2.9 | Define report lifecycle contract | Done. Added report lifecycle contract types with states `requested`, `generating`, `ready`, `failed`, `expired`, `source_changed`; source run/backtest id; source sequence range; governance identity; artifact metadata; generation policy; failure reason; and timestamps. Reports are marked `ready` only when compact source evidence has a sequence range, retained evidence, and non-legacy governance identity. | Contract tests prove reports without valid source evidence metadata and governance identity cannot become `ready`. Run/backtest report API tests prove ready records expose source kind/id, sequence range, governance identity, generation policy, and artifact metadata. |
| 2.10 | Persist report artifact metadata | Done. Added a dedicated report store under the runtime storage parent and persisted `RuntimeEvidenceReportRecord` JSON separately from run/backtest records. Report records link back to source evidence through source kind/id, graph id, sequence range, governance identity, policy, and artifact metadata; they do not copy raw event logs or compact entries. | Save/reload tests prove report detail survives app reload and still traces to the source run, source sequence range, governance identity, and metadata artifact. Backtest report tests prove the same report metadata contract works for backtest evidence. |

#### P1

P1 items turn the minimum evidence chain into a useful review workflow.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 2.11 | Add report generation endpoint | Done. `POST /api/runtime/reports` now acts as the deterministic compact-evidence report generation endpoint for run/backtest sources. The report id is derived from source kind/id, graph id, source sequence range, governance identity, and generation policy; repeated generation for the same source evidence and policy returns the existing record instead of creating timestamp drift. `GET /api/runtime/reports/{report_id}/export` returns a deterministic `RuntimeEvidenceReportArtifact` with schema version, source identity, lifecycle state, sequence range, governance identity, generation policy, evidence digest, and generated section summaries. | Backend integration tests create reports, verify stable repeated generation, load detail, export artifact JSON, and prove the exported artifact links to the same source id, lifecycle state, sequence range, governance identity, policy, and artifact metadata without copying raw events. |
| 2.12 | Add report list/detail/export UI | Done. Added `RuntimeReportPanel` and mounted it in runtime diagnostics and backtest detail. The panel loads reports for the selected run/backtest source, shows lifecycle states (`未生成`, `生成中`, `已就绪`, `失败`, `已过期`, `源已变化`), creates reports, opens detail, and exposes export/reveal links only when ready artifacts exist. The detail view shows source kind, sequence range, retained/source counts, capability hash, deployment revision, and policy without requiring raw JSON inspection. | UI tests verify users can load existing reports, generate a report for the selected source, open detail, and see export/reveal actions and lifecycle state. Backtest detail tests verify the report panel is present in the report lifecycle section. |
| 2.13 | Add evidence summary cards | Done. Added `runtimeEvidenceSummary` and `EvidenceSummaryCards` so report review surfaces summarize capability snapshots, data-quality warnings, risk decisions, execution outcomes, portfolio updates, and security violations from compact evidence/timeline contracts. Cards expose counts, latest event metadata, summaries, and original sequence numbers without using raw JSON as the primary surface. | Unit/UI tests verify cards are derived from compact evidence, preserve source sequence numbers, and render in the report panel with retained/key evidence context. |
| 2.14 | Add timeline compare hooks | Done. Added `compareRuntimeEvidenceSources` to compare two governed evidence sources by governance identity, key event counts, risk decisions, execution outcomes, and source event counts. The hook reuses retention-aware previews rather than introducing another event DTO shape. | Compare tests verify mismatched governance identity, changed risk decisions, changed execution outcomes, and event-count deltas are visible. |
| 2.15 | Add report source-change detection | Done. Report list/detail/export paths now materialize persisted report records against the current saved run/backtest source. Ready reports become `source_changed` when the source is missing or when graph id, sequence range, source/retained event count, governance identity, or generation policy no longer matches the saved source. Ready artifacts are cleared when readiness is revoked. | Backend integration tests mutate saved source evidence after report creation and verify detail returns `source_changed` with structured retryable failure metadata instead of a ready artifact. |
| 2.16 | Add retention-aware loading strategy | Done. Frontend evidence preview prefers compact evidence and retained key windows before detail windows, and exported report artifacts now include `loading_strategy` with primary source, source/retained counts, and whether a detailed window is required. Report UI displays compact-first strategy metadata next to summary cards. | Unit/UI tests verify compact evidence is preferred, detail windows are not required when compact entries are present, and report export carries `loading_strategy.primary_source=compact_evidence`. |
| 2.17 | Add report failure observability | Done. Report records now persist structured `failure` metadata with reason code, user-facing message, and retry eligibility while keeping legacy `failure_reason` populated for compatibility. Failed generation and source-change paths expose retryable reasons, and report UI renders reason code plus retry state. | Backend tests verify failed/source-changed report records expose structured retryable failure metadata; report panel tests verify failure detail is visible without inspecting raw JSON. |

#### P2

P2 items reduce drift and prepare Block 5 contract-first hardening.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 2.18 | Document evidence contract ownership | Done. Added the active Runtime Evidence Contract and linked it from the runtime index and overview docs index. The contract defines ownership, update rules, and field-level expectations for timeline items, replay windows, retained key-event indexes, compact evidence, report lifecycle records, and report artifact export. | Docs index links to the evidence contract. The contract states backend/frontend owner, persisted status, update rule, active fields, report source-change behavior, and the change checklist for snapshot/UI/doc updates. |
| 2.19 | Add API contract snapshots | Done. Added a fixture-style API contract snapshot for timeline, replay, compact evidence, retained key-event index, report record, report artifact metadata, export sections, and loading strategy fields. The test builds a real runtime run, detail response, replay window, report record, and export artifact, then compares public field sets and stable enum/schema values against `tests/fixtures/runtime/evidence_contract_snapshot.json`. | `runtime_evidence_contract_snapshot_matches_fixture` fails when public evidence response fields or stable schema/enum values drift without an intentional fixture update. |
| 2.20 | Add E2E evidence walkthrough | Done. Added `evidence-contract-walkthrough.spec.js` covering backtest detail governed timeline, replay pagination via backend sequence cursors, compact evidence summary mode, report generation lifecycle, and export link visibility in one deterministic browser flow. | Playwright E2E passes on deterministic fixtures, verifies the main evidence path does not require raw JSON text, and covers timeline/replay/compact/report lifecycle together. |
| 2.21 | Add evidence metrics and health checks | Done. Added in-memory evidence metrics and `GET /api/runtime/evidence/health`. The endpoint exposes report generation/failure/source-changed counters, replay page count and latency totals/average, compact projection source/retained totals, detail-window fallback count, persisted report count, report status counts, and the active cleanup policy. | Integration test creates a run, loads replay, generates a report, then verifies the health response exposes report/replay/compact counters without changing normal detail/replay/report behavior. |
| 2.22 | Add retention cleanup policy | Done. Added `quantpilot/evidence-cleanup/v1` and `POST /api/runtime/evidence/cleanup`. Cleanup removes only transient report-generation outputs named with `report-generation-tmp-` or `report-generation-partial-` after the TTL, while persisted report JSON records and saved runtime artifacts remain protected. Runtime Artifact Retention docs now define the evidence report tier. | Integration test creates transient report-generation outputs next to a persisted report record, runs cleanup with deterministic `max_age_ms`, verifies only transient outputs are removed, and verifies the saved report record still loads as `ready`. |

### Block 2 Completion Gate

Block 2 can be marked complete only when these checks are all true:

- Timeline items are the shared evidence shape for live detail, saved detail,
  replay, compact evidence, and report generation.
- Replay windows are sequence-safe, page-safe, and filterable by governed
  envelope fields.
- Compact evidence preserves all key/system events and records its compaction
  policy.
- Report lifecycle states are explicit, persisted, and linked to source
  governance identity and source sequence range.
- Frontend timeline and report surfaces let users inspect evidence without raw
  JSON.
- Tests cover current, saved, replayed, compacted, reported, and legacy/defaulted
  evidence paths.

### Block 2 Data Contract Sketch

Timeline item:

```json
{
  "timeline_item_version": 1,
  "event_id": "evt_001",
  "event_type": "RiskDecisionProduced",
  "sequence_no": 42,
  "stage": "risk",
  "retention_class": "key",
  "severity": "Info",
  "module_key": "builtin.risk.global",
  "node_id": "risk_node",
  "occurred_at_ms": 1760000000000,
  "ingested_at_ms": 1760000000000,
  "summary": "Risk approved",
  "reason_code": "APPROVED",
  "governance": {
    "capability_hash": "sha256:<digest>",
    "deployment_revision": "sha256:<digest>",
    "strategy_version": "1.0.0",
    "parameter_version": "config:<hash>"
  },
  "payload_version": 1,
  "compactability": "retain"
}
```

Replay window:

```json
{
  "kind": "backtest",
  "record_id": "backtest_001",
  "graph_id": "graph_001",
  "source_event_count": 25000,
  "total_events": 420,
  "cursor": 0,
  "sequence_cursor": 1,
  "limit": 100,
  "window_end": 100,
  "previous_cursor": null,
  "next_cursor": 100,
  "previous_sequence_cursor": null,
  "next_sequence_cursor": 101,
  "filters": {
    "stage": "risk",
    "severity": "Warn",
    "retention_class": "key",
    "module_key": "builtin.risk.global",
    "key_only": false
  },
  "events": [],
  "timeline": []
}
```

Compact evidence metadata:

```json
{
  "policy_version": "quantpilot/evidence-compaction/v1",
  "source_event_count": 25000,
  "retained_event_count": 420,
  "dropped_by_retention": {
    "debug": 22000,
    "summary": 2580
  },
  "key_event_count": 420,
  "system_event_count": 2,
  "dropped_by_stage": {
    "agent": 500,
    "system": 20
  },
  "governance": {
    "capability_hash": "sha256:<digest>",
    "deployment_revision": "sha256:<digest>",
    "strategy_version": "1.0.0",
    "parameter_version": "config:<hash>"
  },
  "entries": []
}
```

Retained key-event index:

```json
{
  "index_version": 1,
  "policy_version": "quantpilot/key-event-index/v1",
  "source_event_count": 25000,
  "retained_event_count": 420,
  "key_event_count": 418,
  "system_event_count": 2,
  "entries": [
    {
      "timeline_item_version": 1,
      "event_id": "evt_001",
      "event_type": "CapabilitySnapshotTaken",
      "sequence_no": 1,
      "stage": "system",
      "retention_class": "key",
      "compactability": "retain"
    }
  ]
}
```

Report lifecycle:

```json
{
  "report_id": "report_001",
  "source_kind": "backtest",
  "source_id": "backtest_001",
  "status": "ready",
  "source_sequence_range": {
    "from": 1,
    "to": 420
  },
  "governance": {
    "capability_hash": "sha256:<digest>",
    "deployment_revision": "sha256:<digest>"
  },
  "generation_policy": "quantpilot/report-policy/v1",
  "artifacts": [
    {
      "kind": "html",
      "artifact_id": "report_html_001",
      "file_name": "report.html"
    }
  ],
  "failure_reason": null,
  "failure": null,
  "loading_strategy": {
    "primary_source": "compact_evidence",
    "source_event_count": 25000,
    "retained_event_count": 420,
    "requires_detail_window": false
  }
}
```

### Block 2 Test Checklist

- [x] Timeline item contract includes envelope stage, retention class, sequence, and governance identity.
- [x] Run detail timeline matches replay window for the same sequence range.
- [x] Backtest detail timeline matches replay window for the same sequence range.
- [x] Replay pagination has no duplicate or missing sequence numbers.
- [x] Replay filters preserve ordering and return structured errors for invalid cursors.
- [x] Retained key-event index preserves key evidence and system governance events.
- [x] Compact evidence retains all key/system events.
- [x] Report lifecycle cannot become ready without source evidence metadata.
- [x] Saved report reload preserves source id, source sequence range, governance, and artifacts.
- [x] Frontend timeline reader normalizes live, saved, replayed, retained key-index, and compact evidence.
- [x] Timeline UI groups by envelope stage and filters by severity/retention/module.
- [x] Report UI shows lifecycle, source governance, and export/reveal actions.
- [x] Large-log UI path loads compact/key evidence before detailed windows.
- [x] UTF-8 and user-facing text checks remain green for timeline/report labels.
- [x] Runtime Evidence Contract is linked from active docs indexes.
- [x] API contract snapshot covers timeline, replay, compact evidence, and report lifecycle fields.
- [x] Browser-level evidence walkthrough covers detail timeline, replay paging, compact mode, and report lifecycle.
- [x] Evidence health exposes report/replay/compact counters and cleanup policy.
- [x] Evidence cleanup removes transient generation outputs without deleting saved reports.

### Block 2 Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Timeline introduces a second event DTO family | Live, replay, and report views drift | Derive timeline items from governed runtime events and centralize projection |
| Replay pagination uses wall-clock time instead of sequence | Missing or duplicated evidence windows | Page by envelope `sequence_no` and test cursor boundaries |
| Compact evidence drops key/system events | Reports lose audit-critical facts | Retain all `retention_class=key` and system governance events |
| Reports copy raw logs without source linkage | Reports cannot prove what evidence they summarize | Persist source id, sequence range, governance, and generation policy |
| UI hides governance behind raw JSON | Users cannot audit evidence chain | Show normalized governance rows on timeline/report surfaces |

### Next Block Entry Criteria

Do not start controlled runtime mutation until Block 2 has at least these stable
inputs:

- sequence-safe replay window
- governed timeline item contract
- retained key-event compaction
- report lifecycle states
- source-linked report artifacts

## Block 3: Controlled Runtime Mutation

### Goal

Allow runtime parameters to change while a strategy is running without mixing
old and new state inside one decision chain. Every proposed mutation must have a
versioned parameter identity, an activation boundary, rollback metadata,
permission checks, safe-window evaluation, and auditable events that connect
back to the Block 1 governance foundation and Block 2 evidence surface.

Block 3 does not grant AI direct write access and does not introduce live order
execution. It creates the controlled mutation substrate that Block 4 can later
use through an approval chain.

### Inputs From Block 2

Block 3 assumes these facts are stable:

- capability boundary and runtime governance snapshot are fail-closed
- event envelopes carry sequence, stage, retention class, capability hash, and
  deployment revision
- timeline/replay/report surfaces can audit key evidence without raw JSON
- report lifecycle and source-change detection can prove which evidence a
  review consumed
- evidence health and cleanup endpoints can observe report/replay behavior

If these inputs drift, runtime mutation work must stop until the governance and
evidence contracts are repaired.

### Deliverables

| Deliverable | Description | Acceptance |
|---|---|---|
| Parameter mutation contract | A typed request/proposal/activation contract for parameter changes | Every mutation has proposal id, actor, target scope, old/new parameter version, reason, permission context, and activation boundary |
| Parameter version identity | Deterministic version id for the active parameter set | Same parameter payload yields the same version; any accepted change yields a new version visible in governance and events |
| Runtime mutation ledger | Append-only persisted record of proposals, activations, rejections, and rollbacks | Reloaded runs/backtests can explain which parameter set was active at each boundary |
| Activation boundary controller | Runtime applies accepted changes only at explicit cycle or sequence boundaries | A decision chain never contains mixed parameter versions |
| Safe-window controller | Optional guard that evaluates whether the current state allows activation | Unsafe windows block activation with a structured reason and key event |
| Rollback path | Accepted mutation can revert to a prior parameter version through the same boundary rules | Rollback emits governed events and restores the previous version without deleting history |
| Frontend mutation UI | Users can inspect current parameters, propose changes, see pending/active/rejected/rolled-back state, and request rollback | UI shows version, boundary, safety result, and audit evidence without raw JSON |
| Evidence integration | Mutation events appear in timeline, retained key index, replay, compact evidence, and reports | Reports and replay can prove who changed what, when it activated, and whether it was rolled back |

### Backend Tasks

- [x] Define typed parameter mutation request, proposal, and rejection response contracts.
- [x] Add deterministic parameter-version hashing for normalized runtime parameter payloads.
- [x] Add a mutation ledger store linked to run id, graph id, governance identity, actor, and source capability context.
- [x] Add a runtime mutation permission guard that rejects missing/stale capability boundary or unsupported live mutation scope.
- [x] Add proposal create/list/detail endpoints.
- [x] Add activation endpoint that schedules accepted mutation at a safe boundary instead of applying immediately.
- [x] Add activation boundary controller for cycle start, sequence cursor, and manual pause boundary.
- [x] Add safe-window evaluator with structured deny reasons.
- [x] Add rollback endpoint that schedules a prior parameter version through the same boundary controller.
- [x] Emit governed key events for proposal created and proposal rejected.
- [x] Emit governed key events for activation scheduled, activation applied, and activation failed.
- [x] Emit governed key events for rollback scheduled, rollback applied, and safe-window denied.
- [x] Persist active parameter version in runtime governance snapshots and saved artifacts.
- [x] Add replay/report integration so mutation events and active parameter versions are included in timeline, compact evidence, and reports.
- [x] Add backend contract and integration tests for proposal, activation, boundary isolation, rollback, permission denial, safe-window denial, persistence, and legacy/defaulted records.

### Frontend Tasks

- [x] Add a parameter mutation reader utility that normalizes mutation proposals and governed mutation events.
- [x] Extend the mutation reader utility for active parameter version and activation state.
- [x] Extend the mutation reader utility for rollback state and pending safe-window state.
- [x] Add a mutation panel for runtime diagnostics and strategy workspace runtime surfaces.
- [x] Show current parameter version, pending proposal, target boundary, and active version.
- [x] Show safe-window status.
- [x] Provide activate action with explicit disabled states.
- [x] Provide rollback action with explicit disabled states.
- [ ] Provide propose and cancel-pending actions with explicit disabled states.
- [x] Fail closed when capability context is missing, stale, safe fallback, or permission boundary disallows mutation.
- [x] Surface mutation events in governed timeline filters and evidence summary cards.
- [x] Add report UI rows for mutation lifecycle evidence.
- [x] Add report UI rows for rollback evidence through the mutation lifecycle report section.
- [x] Add UI tests for activation pending state, disabled unsafe states, and evidence visibility.
- [x] Add UI tests for safe-window denial and rollback.
- [x] Preserve UTF-8 Chinese labels in mutation UI and avoid raw JSON as the primary mutation surface.

### Concrete Development Items And Acceptance Conditions

#### P0

P0 items establish the fail-closed mutation contract. No runtime parameter
change can be applied until these are complete.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 3.1 | Done - Define parameter mutation contract | Added typed backend/frontend contracts for proposal id, source run id, graph id, target module/node/parameter path, old value, new value, actor, reason, capability context, requested boundary, status, timestamps, and governance identity. | API tests prove proposal/list/detail payloads expose stable fields. Missing actor, target parameter path, or capability context is rejected before ledger write. |
| 3.2 | Done - Canonicalize parameter version identity | Added deterministic `sha256:<digest>` parameter-version hashing from canonical target + value payloads. Runtime timestamps and display labels are excluded from the version payload. | Integration tests prove reordered parameter maps keep the same version and changed parameter values produce different old/proposed versions. |
| 3.3 | Done - Add mutation permission guard | Proposal creation validates current capability hash, permission boundary, source kind, target module support, actor, reason, and mutation scope before ledger write. | Missing capability context returns structured `parameter_mutation_boundary_violation` and creates no ledger entry. Unsupported source kind or module scope is rejected before persistence. |
| 3.4 | Done - Add append-only mutation ledger | Persisted mutation proposal records under the runtime mutation store, linked to run id, graph id, actor, governance identity, and parameter versions. | Reload test proves proposal detail survives app state reload with identical proposal/source/governance metadata. Activation and rollback transitions remain P1/P2 work. |
| 3.5 | Done - Emit mutation proposal events | Emits governed `ParameterMutationProposed` / `ParameterMutationRejected` key events with envelope stage `system`, retention `key`, reason code, actor, proposal id, and target parameter path. | Runtime detail tests prove proposal/rejection events enter run evidence and carry governance identity through the envelope. |
| 3.6 | Done - Add proposal create/list/detail endpoints | Added `POST /api/runtime/mutations`, `GET /api/runtime/mutations`, and `GET /api/runtime/mutations/:proposal_id`. List/detail read persisted ledger records. | Integration tests create a proposal, list it, reload app state, and open the same proposal with identical source/run/governance metadata. |
| 3.7 | Done - Frontend mutation reader utility | Added a frontend utility that normalizes mutation proposals, governed mutation events, legacy/defaulted fields, and proposal/rejection counts into one reader shape. | Unit tests cover current proposal payloads, legacy missing fields, rejected states, wrapped list shapes, and mutation event payload fallback. |

#### P0 Implementation Notes

- Backend contract lives in `src/frontend_api_types.rs` and uses `RuntimeParameterMutationRecord`, `CreateRuntimeParameterMutationRequest`, `RuntimeParameterMutationTarget`, `RuntimeParameterMutationBoundary`, and `RuntimeParameterMutationGovernance`.
- Ledger persistence lives in `src/runtime_persistence.rs` and stores proposal records in the `mutations` store next to reports/backtests/runs.
- API endpoints live in `src/runtime_api.rs`:
  - `POST /api/runtime/mutations`
  - `GET /api/runtime/mutations?source_kind=run&source_id=<run_id>`
  - `GET /api/runtime/mutations/:proposal_id`
- P0 intentionally supports proposal/rejection only. Activation, active parameter switching, safe-window checks, rollback, and AI approval remain blocked until P1/P2/P3 items are implemented.
- Mutation proposal/rejection events are registered in the event envelope and timeline retention contracts as `system` + `key`, so they flow through runtime detail, replay windows, retained key index, compact evidence, and later report surfaces through the existing evidence path.

#### P1

P1 items make accepted mutations safely activatable at explicit boundaries.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 3.8 | Done - Add activation boundary model | Added supported activation boundaries: `next_cycle_start`, `sequence_cursor`, and `manual_pause`; `immediate` is explicitly rejected. Requested and resolved boundaries are stored in activation state. | Contract/integration tests prove immediate mutation is disabled and activation state includes requested/resolved boundary metadata. |
| 3.9 | Done - Add activation scheduling endpoint | Added `POST /api/runtime/mutations/:proposal_id/activate`, guarded by current capability context. It schedules accepted proposals before any active version change. | Integration tests schedule activation and prove manual-pause proposals remain pending without changing the active parameter version. |
| 3.10 | Done - Apply mutation at boundary | Added deterministic boundary controller: `next_cycle_start` emits a scheduled event, then applies the new parameter version at the resolved next sequence. | Integration tests prove scheduled events use the old parameter version and activated events use the new version, with ordered sequence numbers. |
| 3.11 | Done - Emit activation lifecycle events | Added `ParameterMutationActivationScheduled`, `ParameterMutationActivated`, and `ParameterMutationActivationFailed` as governed key events. | Timeline/replay/compact tests prove activation events are retained and linked to proposal id plus old/new parameter versions. |
| 3.12 | Done - Persist active parameter version in governance | Runtime run governance updates to the active parameter version after boundary-applied activation; saved run persistence preserves this when the source has been saved. | Run detail and report export tests show the active parameter version after activation. |
| 3.13 | Done - Build mutation UI panel | Added a pure `RuntimeMutationPanel` for current version, proposal list, pending activation, resolved boundary, and activation status. API helpers are available for page/store integration. | UI tests verify users can inspect proposed/active versions, see pending activation, and stay blocked without capability context, without raw JSON. |
| 3.14 | Done - Add mutation timeline/report integration | Mutation lifecycle events are registered as system/key events and included in governed timeline, evidence summary cards, compact evidence, replay, and report sections. | Integration tests prove mutation proposal and activation are auditable through timeline, replay, compact summary, and report export. |

#### P1 Implementation Notes

- Activation scheduling is additive to the P0 ledger: proposal records now carry `activation_state` and append-only `lifecycle` entries.
- The runtime event repair path now preserves already-valid historical envelopes, so boundary-before events keep their old `parameter_version` after activation.
- `next_cycle_start` resolves to the next lifecycle boundary in deterministic tests and emits both scheduled and activated events. `manual_pause` stays pending for later operator/runtime action. `sequence_cursor` can be scheduled when a future resolved sequence is provided.
- P1 intentionally does not add safe-window policy, rollback execution, or AI approval. Those remain P2/P3 work.
- Report records and exports now expose `mutation_lifecycle_event_count`; exports add a `mutation_lifecycle` section when retained mutation lifecycle evidence exists.

#### P2

P2 items add safety windows and rollback without changing the core boundary
model.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 3.15 | Done - Define safe-window policy | Added typed safe-window snapshot/state contracts for runtime status, open order count, outstanding risk violation, data freshness, portfolio exposure, cooldown window, retryability, and policy version. | Backend and frontend tests cover allowed default windows and denied windows with structured reason codes such as `SAFE_WINDOW_RUNTIME_ACTIVE` and `SAFE_WINDOW_OPEN_ORDERS`. |
| 3.16 | Done - Add safe-window evaluator | Activation and rollback requests evaluate the safe-window snapshot before scheduling or applying a parameter version. Missing context defaults to the conservative deterministic safe test window; explicit unsafe context fails closed. | Unsafe activation returns `parameter_mutation_safe_window_denied`, records no active parameter change, and keeps the proposal retryable through the persisted safe-window-denied state. |
| 3.17 | Done - Emit safe-window events | Added governed key event `ParameterMutationSafeWindowDenied` with proposal id, target, denied policy result, retryability, and current state snapshot in payload. | Integration tests prove denial events are appended to run evidence as `system` + `key` events and the active parameter version does not change. |
| 3.18 | Done - Add rollback contract | Added `POST /api/runtime/mutations/:proposal_id/rollback` and rollback request fields for target prior parameter version, boundary, actor, reason, capability context, and safe-window context. | Contract tests reject unknown target versions and accept ledger-backed prior parameter versions. |
| 3.19 | Done - Add rollback execution | Rollback creates an append-only reverse mutation record, schedules/applies through the same boundary controller, updates active runtime governance, and emits rollback scheduled/applied/failed lifecycle events. | Integration test activates a parameter mutation, rolls back to the original ledger-backed version, and verifies final active parameter version equals the original version. |
| 3.20 | Done - Add rollback UI | Extended the mutation reader and `RuntimeMutationPanel` for safe-window state, rollback linkage, pending/applied/failed rollback counts, denied state, and disabled rollback actions. | UI tests verify safe-window denial display, rollback disabled/available states, and callback payloads without raw JSON. |

#### P2 Implementation Notes

- Safe-window evaluation is backend-owned. Frontend display mirrors `safe_window_state`, but activation/rollback safety does not depend on UI-only checks.
- `ParameterMutationSafeWindowDenied`, `ParameterMutationRollbackScheduled`, `ParameterMutationRolledBack`, and `ParameterMutationRollbackFailed` are registered as governed `system` + `key` events, so they flow through timeline, replay, compact evidence, and reports.
- Rollback is append-only: it creates a new mutation record with `rollback_of` and `rollback_target_parameter_version`; it never deletes or mutates the original activation evidence.
- Rollback target versions must already be present in the mutation ledger for the same source/target pair. Unknown versions are rejected before any event or active version change.
- P2 intentionally leaves propose/cancel-pending page integration, mutation health metrics, contract snapshots, and E2E walkthroughs to P3/follow-up work.

#### P3

P3 items harden operations and prepare the AI approval chain.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 3.21 | Done - Add mutation health metrics | Extended `/api/runtime/evidence/health` with proposal created/rejected, activation scheduled/applied/failed, activation latency total/average, safe-window denial, rollback attempt/scheduled/applied/failed counters. | API tests prove counters update for safe-window denial, activation, unknown rollback attempt, and successful rollback without changing user-visible mutation behavior. |
| 3.22 | Done - Add mutation contract snapshots | Added `tests/fixtures/runtime/mutation_contract_snapshot.json` covering proposal, activation, safe-window denial, rollback, lifecycle entries, health metrics, timeline/replay mutation events, and report mutation sections. | `runtime_parameter_mutation_contract_snapshot_matches_fixture` fails when public mutation fields or stable mutation values drift without an intentional fixture update. |
| 3.23 | Done - Add mutation E2E walkthrough | Added `runtime-mutation-walkthrough.spec.js` with deterministic mocked run history and mutation ledger states for proposal, safe-window denial, activation, and rollback in the runtime research surface. | E2E verifies the mutation panel exposes the main mutation lifecycle without raw JSON. |
| 3.24 | Done - Document mutation contract ownership | Added active Runtime Mutation Contract and linked it from runtime, planning, and overview docs indexes. | Docs identify field owners, update checklist, safety rules, governed events, metrics, and Block 4 AI approval handoff constraints. |

#### P3 Implementation Notes

- Mutation metrics are part of the existing runtime evidence health endpoint so operational checks can correlate mutation activity with replay/report evidence health.
- Mutation contract snapshots intentionally check public field sets and stable enum/event/report values, not timestamps or generated ids.
- The P3 E2E walkthrough is a deterministic frontend path over mocked API fixtures. It verifies the user-facing mutation panel and raw-JSON avoidance; backend mutation state transitions remain covered by API integration tests.
- The active Runtime Mutation Contract owns future AI approval handoff rules. Block 4 may add approval states, but must not bypass capability context, permission boundary, safe-window evaluation, explicit activation boundaries, append-only ledger records, or governed evidence events.

### Block 3 Completion Gate

Block 3 can be marked complete only when these checks are all true:

- Parameter mutation proposals are typed, persisted, actor-linked, and
  governed by the current capability boundary.
- Parameter version identity is deterministic and visible in governance,
  mutation events, timeline, replay, compact evidence, and reports.
- Runtime activation happens only at explicit boundaries; immediate mixed-chain
  mutation is not possible.
- Safe-window denial is structured, auditable, and retained as key evidence.
- Rollback follows the same proposal/boundary/event path as activation.
- Frontend mutation surfaces show current, pending, denied, active, and
  rolled-back states without raw JSON.
- Tests cover proposal, permission denial, activation boundary isolation,
  safe-window denial, rollback, persistence/reload, replay, compact evidence,
  report export, and legacy/defaulted states.

### Block 3 Data Contract Sketch

Mutation proposal:

```json
{
  "proposal_id": "mutation_001",
  "source_kind": "run",
  "source_id": "run_001",
  "graph_id": "graph_001",
  "target": {
    "node_id": "risk_node",
    "module_key": "builtin.risk.global",
    "parameter_path": "max_position"
  },
  "old_value": 0.25,
  "new_value": 0.18,
  "old_parameter_version": "sha256:<digest>",
  "proposed_parameter_version": "sha256:<digest>",
  "status": "proposed",
  "activation_boundary": {
    "requested": "next_cycle_start",
    "resolved_sequence_no": null
  },
  "actor": {
    "actor_id": "user_001",
    "display_name": "User 001"
  },
  "reason": "Reduce exposure after volatility spike.",
  "governance": {
    "capability_hash": "sha256:<digest>",
    "deployment_revision": "sha256:<digest>",
    "permission_boundary_model_version": "quantpilot/permission-boundary/v1"
  },
  "created_at_ms": 1760000000000,
  "updated_at_ms": 1760000000000
}
```

Activation state:

```json
{
  "proposal_id": "mutation_001",
  "status": "activation_scheduled",
  "old_parameter_version": "sha256:<digest>",
  "active_parameter_version": "sha256:<digest>",
  "target_parameter_version": "sha256:<digest>",
  "requested_boundary": "next_cycle_start",
  "resolved_boundary": {
    "sequence_no": 88,
    "cycle_id": "slow_cycle_12"
  },
  "safe_window": {
    "status": "allowed",
    "policy_version": "quantpilot/mutation-safe-window/v1",
    "reason_code": "SAFE_WINDOW_OPEN"
  }
}
```

Mutation event:

```json
{
  "event_id": "evt_mutation_001",
  "event_type": "ParameterMutationActivated",
  "summary": "Parameter max_position changed from 0.25 to 0.18 at sequence 88.",
  "reason_code": "PARAMETER_MUTATION_ACTIVATED",
  "payload": {
    "proposal_id": "mutation_001",
    "target_parameter_path": "max_position",
    "old_parameter_version": "sha256:<digest>",
    "new_parameter_version": "sha256:<digest>",
    "activation_sequence_no": 88
  },
  "envelope": {
    "stage": "system",
    "retention_class": "key",
    "sequence_no": 88,
    "capability_hash": "sha256:<digest>",
    "deployment_revision": "sha256:<digest>"
  }
}
```

### Block 3 Test Checklist

- [x] Mutation contract rejects missing target, actor, parameter path, and capability context.
- [x] Parameter version hashing is deterministic and excludes timestamps/display labels.
- [x] Permission guard rejects stale capability hash and unsupported mutation scope.
- [x] Proposal create/list/detail survives reload.
- [x] Activation scheduling does not change active parameter version before boundary.
- [x] Boundary application never mixes parameter versions inside one decision chain.
- [x] Mutation lifecycle events are envelope-complete and retained as key evidence.
- [x] Safe-window denial is structured, retry-aware, and reportable.
- [x] Rollback restores a ledger-backed prior parameter version at a boundary.
- [x] Frontend mutation reader normalizes current, pending, denied, active, rollback, and legacy states.
- [x] Mutation UI disables unsafe actions and exposes boundary/safe-window state.
- [x] Timeline, replay, compact evidence, and reports show mutation evidence without raw JSON.
- [x] Contract snapshot and E2E walkthrough cover the main mutation path.
- [x] UTF-8 and user-facing text checks remain green for mutation labels.

### Block 3 Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Immediate parameter apply bypasses boundary | One decision chain mixes old and new runtime state | Only schedule activation through explicit boundary controller; no direct write path |
| Parameter hash includes display noise | Governance changes without real parameter change | Canonicalize normalized parameter payload and exclude timestamps/labels |
| Rollback deletes history | Audit trail cannot prove what happened | Use append-only rollback events and retain every prior proposal/activation |
| Safe-window policy becomes UI-only | Backend can activate unsafe changes | Evaluate safe windows backend-side before scheduling/applying activation |
| Mutation events are not retained | Reports cannot audit hot updates | Mark mutation lifecycle and denial events `retention_class=key` |
| Block 4 AI path writes directly | AI bypasses human approval and runtime guard | Block 3 exposes proposal substrate only; Block 4 must use same guarded proposal/approval path |

### Next Block Entry Criteria

Do not start AI proposal and approval chain until Block 3 has at least these
stable inputs:

- parameter mutation proposal contract
- deterministic parameter-version identity
- append-only mutation ledger
- activation boundary controller
- safe-window denial contract
- rollback through boundary controller
- mutation evidence visible in timeline/replay/report surfaces

## Block 4: AI Proposal And Approval Chain

### Goal

Allow AI to assist with runtime parameter changes without giving it a direct
write path into live runtime state. AI may draft candidates, attach evidence,
run static checks, request sandbox replay, and recommend approval decisions.
Human approval and the Block 3 mutation boundary controller remain the only
path to live activation.

### Boundary Rules

- AI output is advisory until converted into a guarded mutation proposal.
- AI cannot activate, rollback, or directly change the active parameter version.
- Every AI candidate must carry capability context, permission boundary context,
  source evidence, model identity, prompt identity, and deterministic hashes.
- Missing or unsupported `ai_write_policy` fails closed to proposal read-only
  behavior.
- Static checks and sandbox replay must complete before approval can be granted.
- Human approval cannot be replaced by AI self-approval.
- Approved AI candidates still flow through Block 3 safe-window and activation
  boundaries before runtime state changes.
- Timeline, replay, compact evidence, and reports must expose the AI approval
  lifecycle as governed evidence rather than raw JSON.

### Deliverables

| Deliverable | Description | Acceptance |
|---|---|---|
| AI proposal contract v1 | Typed candidate, recommendation, static-check, sandbox-replay, and approval decision payloads | Backend, frontend, fixtures, and docs use the same status and reason-code vocabulary |
| AI proposal ledger | Append-only storage for AI candidates and lifecycle transitions | Candidate creation, static-check result, sandbox replay, approval, rejection, and conversion are auditable |
| Permission boundary enforcement | Guard `ai_write_policy` and capability context before accepting AI-authored candidates | Missing, stale, malformed, or unsupported boundary context returns structured denial and creates no live mutation |
| Static validation pipeline | Check target module, parameter path, value schema, allowed scope, source evidence, and stale governance before replay/approval | Invalid candidates cannot reach sandbox replay or approval |
| Sandbox replay evidence | Dry-run replay contract that compares candidate behavior against the baseline without mutating live state | Replay produces retained evidence and proves active parameter version remains unchanged |
| Human approval workflow | Approve, reject, request changes, expire, and convert approved candidates into Block 3 mutation proposals | AI cannot approve itself; approved candidates still require Block 3 activation |
| Evidence integration | Timeline, replay, compact evidence, and report lifecycle include AI proposal and approval events | Users can audit why AI suggested a change, who approved it, and whether it became a mutation proposal |
| Contract and e2e coverage | Snapshot tests, API tests, frontend tests, and an end-to-end walkthrough | Contract drift fails tests before the UI and backend disagree |

### Backend Tasks

- [x] Define versioned AI proposal, recommendation, and static-check contracts for P0 candidate intake. Sandbox-replay and approval decision contracts remain P1/P2.
- [x] Add append-only AI proposal ledger storage and reload-safe serialization.
- [x] Enforce `ai_write_policy` and capability context before candidate creation.
- [x] Add static validation for target module, parameter path, value schema, governance freshness, source evidence, and actor identity.
- [x] Add create/list/detail/status endpoints for AI candidates.
- [ ] Add sandbox replay endpoints that never mutate active runtime state.
- [ ] Add approval decision endpoints for approve, reject, request changes, expire, and convert-to-mutation-proposal actions.
- [x] Emit governed lifecycle events for AI proposal creation and static-check result. Sandbox replay, approval decision, expiration, and conversion remain P1/P2.
- [ ] Link approved candidates to Block 3 mutation proposal ids without bypassing safe-window or activation boundaries.
- [ ] Add metrics for proposal volume, static-check failures, replay pass/fail, approval latency, and conversion rate.
- [ ] Add contract snapshots and regression tests for denial, replay isolation, approval, and conversion paths.

### Frontend Tasks

- [x] Add an AI proposal reader that normalizes candidate and static-check status. Sandbox-replay and approval status remain P1/P2.
- [ ] Add an AI proposal queue/review surface with capability and permission denial reasons.
- [ ] Show model identity, prompt/evidence hashes, source report/run links, target parameter, and proposed value diff.
- [ ] Show static-check failures before sandbox replay controls are enabled.
- [ ] Show sandbox replay comparison against baseline runtime/report evidence.
- [ ] Require explicit human approval/rejection actions and disable self-approval or stale-governance candidates.
- [ ] Convert approved candidates into Block 3 mutation proposals through existing guarded mutation flows.
- [ ] Surface AI proposal lifecycle in timeline, replay, compact evidence, and report panels.
- [ ] Add UI tests for disabled states, denial reasons, replay evidence, approval decisions, and conversion links.

### Concrete Development Items And Acceptance Conditions

#### P0

P0 establishes the fail-closed AI proposal substrate. It must land before any
sandbox replay or approval action exists.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 4.1 | Done - Define AI proposal contract | Added typed AI candidate, source evidence, model identity, prompt hash, evidence hash, target mutation, lifecycle status, actor, static-check, and governance fields. P0 status values include `draft`, `submitted`, `static_check_failed`, `static_check_passed`, `denied`, and `expired`. | Backend tests reject missing contract fields, actor, prompt/evidence hash, capability context, and invalid hash identities before candidate intake. |
| 4.2 | Done - Add append-only AI proposal ledger | Added AI proposal storage under `ai-proposals`, separate from the Block 3 mutation ledger. Records include source run id, graph id, target module/path, proposed value, normalized hashes, actor identity, lifecycle, and timestamps. | Create/list/detail survives persisted reload paths. Candidate lifecycle is retained in record history and does not create Block 3 mutation records. |
| 4.3 | Done - Enforce AI permission boundary | Candidate creation now validates capability context and requires `ai_write_policy=proposal_only`. Missing, malformed, stale, or unsupported boundary context returns `ai_proposal_denied`. | Tests prove denied AI candidates create no AI proposal record, no mutation proposal, no activation schedule, and no active parameter-version change. |
| 4.4 | Done - Add static validation pipeline | Static validation checks module target, value presence, source evidence availability, actor identity, governance context, prompt/evidence identity, and deterministic proposed parameter version. No-op values stop as `static_check_failed`. | Invalid static candidates are auditable with structured reason codes and cannot enter sandbox replay or approval because those routes do not exist in P0. |
| 4.5 | Done - Add proposal read APIs | Added `POST /api/runtime/ai-proposals`, `GET /api/runtime/ai-proposals`, and `GET /api/runtime/ai-proposals/:ai_proposal_id` with source and status filters. | Backend and frontend tests read the same normalized proposal after creation and static-check failure. |
| 4.6 | Done - Emit governed proposal events | Added `AIProposalCreated`, `AIProposalDenied`, `AIProposalStaticCheckPassed`, and `AIProposalStaticCheckFailed` as known system/key event types. P0 create path emits created plus static-check pass/fail events. | Run detail and replay tests assert envelope completeness, `stage=system`, `retention_class=key`, capability hash linkage, and key replay visibility. |
| 4.7 | Done - Add frontend AI proposal normalizer | Added `runtimeAiProposal` utility for proposal status, target, source evidence, governance, model metadata, static-check result, denial reason, and disabled/actionable state. Unknown statuses fall back to disabled presentation. | Unit tests cover complete, partial, unknown-status, static-check-failed, wrapped list, and event-payload proposal shapes without raw JSON reads in components. |

#### P1

P1 adds sandbox replay and evidence comparison. It must prove replay isolation
before any human approval path can convert a candidate into a mutation proposal.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 4.8 | Define sandbox replay contract | Add request/response shape for replaying an AI candidate against baseline capability, deployment revision, parameter version, and source run/report evidence. | Snapshot tests cover replay request, accepted response, static-denied response, and stale-governance response. |
| 4.9 | Implement replay isolation | Execute candidate replay in a dry-run context that never writes active runtime state, activation schedules, or Block 3 mutation ledger entries. | Tests assert active parameter version, mutation ledger, and activation queue remain unchanged after sandbox replay. |
| 4.10 | Compare candidate against baseline | Produce deterministic comparison rows for key metrics, risk limits, rejected signals, safe-window eligibility, and report deltas. | Replay output can explain what changed, what stayed unchanged, and which evidence supports the recommendation. |
| 4.11 | Emit sandbox replay events | Emit `AIProposalSandboxReplayStarted`, `AIProposalSandboxReplayCompleted`, and `AIProposalSandboxReplayFailed` as governed key events. | Timeline and replay tests can locate replay events and link them to proposal id, source evidence, and replay result id. |
| 4.12 | Add sandbox review UI | Show replay status, baseline/candidate comparison, static-check result, denied reasons, and evidence links in the AI proposal review surface. | UI tests show replay controls disabled until static checks pass and show replay evidence without raw JSON inspection. |
| 4.13 | Add replay report integration | Attach sandbox replay evidence to report lifecycle and compact evidence retention. | Report tests include replay summary, candidate/baseline deltas, and proposal ids in retained key evidence. |

#### P2

P2 adds human approval and conversion into the existing Block 3 mutation flow.
Approval does not activate runtime state by itself.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 4.14 | Define approval decision contract | Add approve, reject, request-changes, and expire decisions with approver identity, rationale, governance snapshot, replay evidence id, and decision timestamp. | Contract tests reject missing human approver, missing rationale, missing replay result, stale governance, and AI self-approval. |
| 4.15 | Add approval transition endpoints | Add guarded endpoints for approval decisions. Enforce state transitions from static/replay-passed candidates only. | Invalid transitions return structured denials and append audit events without mutating runtime state. |
| 4.16 | Convert approved candidate to mutation proposal | Create or link a Block 3 mutation proposal from an approved AI candidate. Preserve source AI proposal id, approval id, replay evidence id, and governance identity. | Conversion creates a normal Block 3 proposal in `proposed` state and does not schedule activation automatically. |
| 4.17 | Emit approval lifecycle events | Emit `AIProposalApprovalRequested`, `AIProposalApproved`, `AIProposalRejected`, `AIProposalChangesRequested`, `AIProposalExpired`, and `AIProposalConvertedToMutationProposal`. | Event tests assert envelope completeness, key retention, approver identity, proposal linkage, and mutation proposal linkage. |
| 4.18 | Add approval queue UI | Show pending, approved, rejected, changes-requested, expired, and converted candidates with explicit disabled reasons. | UI tests prove stale, replay-missing, denied, and self-approval candidates cannot be approved. |
| 4.19 | Integrate approval evidence into timeline/replay/report | Display approval chain, approver rationale, replay evidence, and mutation proposal linkage in evidence surfaces. | A user can trace AI candidate to approval decision to Block 3 mutation proposal without opening raw JSON. |

#### P3

P3 hardens operations and prepares the next contract-first delivery block.

| ID | Development item | Concrete content | Acceptance condition |
|---|---|---|---|
| 4.20 | Add AI approval metrics | Track candidate counts, denial counts, static-check failures, replay pass/fail, approval latency, approval outcomes, and conversion counts. | Health/metrics tests expose stable metric names and safe zero defaults. |
| 4.21 | Add contract snapshots | Add backend/frontend snapshots for AI proposal, static check, sandbox replay, approval decision, lifecycle events, and converted mutation proposal links. | Snapshot drift forces deliberate fixture updates and docs review. |
| 4.22 | Add end-to-end walkthrough | Cover AI candidate creation, static check, sandbox replay, human approval, conversion to Block 3 mutation proposal, and evidence review. | E2E test proves no active parameter change occurs until the converted mutation proposal follows Block 3 activation rules. |
| 4.23 | Expand active AI approval contract | P0 created the active Runtime AI Approval Contract for candidate intake and linked it from planning, runtime, and overview indexes. P3 must expand it for sandbox replay, human approval, conversion, and Block 5 handoff. | Docs identify field ownership, safety rules, lifecycle events, test ownership, and Block 5 contract handoff. |
| 4.24 | Prepare Block 5 handoff fixtures | Prepare OpenAPI/AsyncAPI candidate shapes, mock payloads, and Pact scenario notes for Block 5. | Block 5 can start from stable AI proposal, replay, approval, and conversion fixtures instead of reverse-engineering runtime code. |

### Block 4 Completion Gate

Block 4 can be marked complete only when these checks are all true:

- AI cannot directly mutate active parameters, activate changes, or perform rollback.
- AI candidate creation is blocked by missing, stale, malformed, or unsupported capability/permission context.
- AI proposals are typed, append-only, reload-safe, and include model, prompt, evidence, actor, capability, and deployment identity.
- Static checks run before sandbox replay and produce structured pass/fail reason codes.
- Sandbox replay is isolated from live runtime state and produces retained comparison evidence.
- Human approval is required before conversion and AI cannot approve itself.
- Approved AI candidates convert only into normal Block 3 mutation proposals; activation still depends on Block 3 safe-window and boundary rules.
- Timeline, replay, compact evidence, and reports expose AI proposal, replay, approval, and conversion lifecycle events.
- Frontend surfaces disabled states and denial reasons without requiring raw JSON inspection.
- Contract snapshots, unit tests, API tests, frontend tests, and e2e walkthrough cover the main path and denial paths.
- UTF-8 and user-facing text checks remain green for AI approval labels.

### Block 4 Test Checklist

- [x] AI proposal contract rejects missing target, actor, model identity, prompt hash, evidence hash, and governance identity.
- [x] AI permission guard rejects missing, stale, malformed, or unsupported `ai_write_policy`.
- [x] Denied AI candidates create no mutation proposal, activation schedule, rollback, or active parameter-version change.
- [x] AI proposal ledger survives reload and preserves append-only transition history.
- [x] Static validation blocks invalid module/path/schema/scope/source-evidence/governance candidates.
- [ ] Sandbox replay cannot write active runtime state or Block 3 mutation ledger entries.
- [ ] Sandbox replay comparison includes baseline, candidate, key metrics, risk results, and evidence links.
- [ ] Approval endpoints reject invalid transitions, stale governance, missing replay evidence, and AI self-approval.
- [ ] Approved candidates convert into Block 3 mutation proposals without auto-activation.
- [x] AI proposal creation and static-check events are envelope-complete and retained as key evidence. Replay, approval, and conversion events remain P1/P2.
- [ ] Timeline, replay, compact evidence, and reports show AI approval evidence without raw JSON.
- [x] Frontend normalizer marks non-actionable P0 AI proposal states as disabled and exposes denial/static-check reasons. Approval-action UI remains P2.
- [ ] Contract snapshots and e2e walkthrough cover the main approval path and denial paths.
- [ ] UTF-8 and user-facing text checks remain green for AI approval labels.

### Block 4 Risks

| Risk | Impact | Mitigation |
|---|---|---|
| AI writes directly to runtime state | Approval chain becomes cosmetic and bypasses Block 3 safety | No AI endpoint may call activation, rollback, or active parameter update paths; conversion creates only a normal mutation proposal |
| AI proposal lacks reproducible identity | Reports cannot prove which model/prompt/evidence produced the recommendation | Persist model identity, prompt hash, evidence hash, source ids, capability hash, and deployment revision |
| Sandbox replay mutates live state | Dry-run evidence changes runtime behavior before approval | Run replay in isolated context and assert active parameter version, mutation ledger, and activation queue are unchanged |
| AI approves itself | Human approval boundary is lost | Require human approver identity and reject actor/model self-approval |
| Capability changes after replay | Approval is based on stale governance | Revalidate capability hash, deployment revision, and permission boundary before approval and conversion |
| Rejected AI proposals disappear | Audit trail hides unsafe recommendations | Retain denied, rejected, expired, and changes-requested events as key evidence |
| UI implies AI autonomy | Users believe AI can execute live writes | UI copy and disabled states must reflect proposal-only behavior unless future capability policy explicitly changes |

### Next Block Entry Criteria

Do not start contract-first delivery and operational hardening until Block 4 has
at least these stable inputs:

- typed AI proposal and approval contracts
- append-only AI proposal ledger
- permission-boundary denial contract for AI-authored candidates
- static-check result contract
- sandbox replay evidence contract
- human approval lifecycle events
- approved-candidate to Block 3 mutation-proposal linkage
- timeline/replay/report evidence integration
- contract snapshot and e2e baseline for the AI approval path
