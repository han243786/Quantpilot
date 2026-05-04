# Runtime Governance Contract

This file is the active source-of-truth contract for runtime governance identity.
Use it when changing capability contracts, event envelopes, runtime/backtest
governance snapshots, deployment revisions, or permission-boundary enforcement.

The v0.2.0 worklist tracks implementation progress, but this document owns the
field contract after Block 1.

## Contract Owners

Ownership is assigned by role, not by person name.

| Contract area | Source of truth | Owner role | Required update when changed |
|---|---|---|---|
| Capability contract | `/api/capabilities` response and backend capability contract builder | backend capability owner | backend fixture, frontend default capability, support matrix tests, docs index |
| Event envelope | `FrontendRuntimeEvent.envelope` on run, replay, SSE, backtest, and artifact events | backend runtime owner | envelope validator tests, run/backtest integration tests, replay/SSE tests |
| Runtime governance snapshot | `governance` on run/backtest detail and backtest manifest | backend runtime persistence owner | saved/reloaded artifact tests, legacy default tests, frontend reader tests |
| Deployment revision | deterministic digest from strategy version, compile id, parameter version, and capability hash | backend compile/runtime owner | canonical hash tests and saved artifact reload tests |
| Permission boundary | capability `permission_boundary` plus runtime write guard | backend runtime owner and frontend capability-gate owner | backend guard tests, frontend fail-closed action tests, UI wording checks |
| Governance diagnostics display | normalized governance rows in diagnostics/detail panels | frontend runtime owner | diagnostics/detail UI tests and governance reader tests |

## Capability Contract

The backend capability contract is authoritative. Frontend defaults, fixtures,
and support-matrix rules must follow it.

Required fields:

- `api_version`
- `schema_version`
- `schema_hash`
- `chain_stages`
- `versioning`
- `permission_boundary`

Rules:

- `schema_hash` must be `sha256:<digest>`.
- The digest must use canonical JSON and exclude request-time or display-only
  noise.
- Safe fallback capabilities are not trusted runtime capability boundaries and
  must remain more restrictive than normal capability mode.
- If frontend and backend capability facts disagree, update frontend capability
  normalization and tests to match backend truth.

Implementation references:

- [src/capability_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/capability_api.rs)
- [frontend/src/capabilities/supportMatrix.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/capabilities/supportMatrix.js)
- [frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json)

## Runtime Governance Snapshot

Every new run and backtest must expose a `governance` snapshot. Saved backtest
manifests must retain the same governance identity.

Required fields:

- `schema_version`
- `governance_source`
- `capability_hash`
- `strategy_version`
- `parameter_version`
- `deployment_revision`
- `permission_boundary`

Allowed provenance values currently used by the runtime are:

- `current_runtime`: produced by a new in-memory runtime/backtest record.
- `loaded_manifest`: materialized from a saved or transient artifact manifest.
- `legacy_default`: backfilled from an old record that did not contain
  governance data.

Rules:

- Missing old governance must load with safe defaults, not fail the read path.
- Defaulted legacy governance must be explicitly visible through
  `governance_source`.
- New runtime write paths must not rely on legacy/defaulted governance to start a
  new run.
- Frontend code must read governance through
  `frontend/src/utils/runtimeGovernance.js`, not ad hoc nested property reads in
  each component.

Implementation references:

- [src/runtime_response_mapping.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_response_mapping.rs)
- [src/runtime_persistence.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_persistence.rs)
- [frontend/src/utils/runtimeGovernance.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/runtimeGovernance.js)

## Event Envelope

Every runtime/backtest event exposed to the frontend must carry an envelope.
This applies to live SSE events, saved run detail, saved backtest detail,
backtest artifact event logs, and replay responses.

Required fields:

- `event_id`
- `event_type`
- `stage`
- `run_id`
- `sequence_no`
- `occurred_at_ms`
- `ingested_at_ms`
- `trace_id`
- `module_key`
- `strategy_version`
- `parameter_version`
- `deployment_revision`
- `capability_hash`
- `mode`
- `severity`
- `retention_class`
- `reason_code`
- `payload_version`

Rules:

- `sequence_no` must be contiguous inside one exposed event list.
- `stage` and `retention_class` are typed backend enums serialized to contract
  strings.
- Envelope `capability_hash` and `deployment_revision` must match the enclosing
  run/backtest governance snapshot.
- Legacy events may be repaired on load before exposure, but exposed events must
  satisfy the validator.
- `CapabilitySnapshotTaken` and `SecurityViolationDetected` are system events
  with `retention_class=key`.

Implementation references:

- [src/runtime_event_projection.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_event_projection.rs)
- [frontend/src/utils/runtimeDiagnosticsProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/runtimeDiagnosticsProjection.js)

## Permission Boundary

The permission boundary defines runtime safety behavior and frontend action
gating.

Required fields:

- `model_version`
- `execution_owner_module`
- `live_execution_allowed`
- `ai_write_policy`
- `plugin_network_default`
- `non_execution_order_access`

Current safe defaults:

- `live_execution_allowed=false`
- `ai_write_policy=disabled`
- `plugin_network_default=deny`
- `non_execution_order_access=deny`

Rules:

- Missing or malformed runtime capability context must reject runtime write
  requests before creating a run/backtest record.
- Missing frontend permission boundary must block compile/run/backtest/sweep
  actions.
- Unknown frontend permission values must normalize to the restrictive default.
- User-facing UI and docs must not claim live execution or AI write support
  unless the capability permission policy allows it.

Implementation references:

- [src/runtime_validation.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_validation.rs)
- [frontend/src/capabilities/supportMatrix.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/capabilities/supportMatrix.js)
- [frontend/src/store/graphStoreRuntimeSessionActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeSessionActions.js)

## Diagnostics Surface

Runtime diagnostics and backtest detail views must expose enough governance
identity for a user to answer which capability boundary, strategy version,
parameter version, deployment revision, and permission model produced the
event/result.

Rules:

- Long hashes may be visually shortened.
- Full values must remain available through tooltip or copy metadata.
- UI must show normalized governance rows, not raw JSON.
- Diagnostics should use `event.envelope.stage` and
  `event.envelope.retention_class` as timeline grouping inputs when the timeline
  block starts.

Implementation references:

- [frontend/src/components/RuntimeDiagnosticsPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/RuntimeDiagnosticsPanel.jsx)
- [frontend/src/pages/BacktestDetailPage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/BacktestDetailPage.jsx)

## Change Checklist

Any change to this contract must update all affected layers in the same batch:

- backend capability or runtime builders
- persisted run/backtest artifact shape
- frontend normalizers and capability gates
- diagnostics/detail display when user-visible governance facts change
- tests for current, saved, streamed, replayed, and legacy/defaulted records
- this document and the docs index

The worklist item is not complete until the active contract docs and
implementation progress agree.
