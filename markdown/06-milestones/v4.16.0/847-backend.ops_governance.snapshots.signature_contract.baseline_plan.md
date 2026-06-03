# v4.16.0 backend.ops_governance.snapshots.signature_contract equivalence baseline and extraction plan

> Batch: BE-001NT-01
> Node: `backend.ops_governance.snapshots.signature_contract`
> Parent: `backend.ops_governance.snapshots`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots.signature_contract` is frozen as the shared snapshot signature input construction child.

BE-001NT-01 does not move code. It defines the exact baseline and allowed movement for BE-001NT-02.

## Current Owner

Current implementation is still in `src/backend/ops_governance/snapshots/handlers.rs`.

The child boundary is:

- `build_signature_input`.

The parent bridge must remain:

- `build_signature_input`.

Create and restore children must continue to call the parent bridge, not the child directly.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Return type | Still returns `serde_json::Value`. |
| Capability field | Still writes `"capability_hash"` from the provided capability hash. |
| Strategy field | Still writes `"strategy_version"` from the provided strategy version. |
| Parameter field | Still writes `"parameter_version"` from the provided parameter version. |
| Core IR field | Still writes `"core_ir_digest"` from the provided core IR digest. |
| Event bounds | Still nests from/to event IDs, from/to sequence, and event count under `"event_slice_bounds"`. |
| Created timestamp | Still writes `"created_at_ms"` from the provided timestamp. |
| Caller contract | Create and restore flows still pass the same arguments through the snapshots parent bridge. |

## Allowed BE-001NT-02 Movement

BE-001NT-02 may:

- create `src/backend/ops_governance/snapshots/handlers/signature_contract.rs`;
- move only the implementation body of `build_signature_input` into that private child module;
- add a private `mod signature_contract;` declaration in `src/backend/ops_governance/snapshots/handlers.rs`;
- keep a parent bridge named `build_signature_input` with the same signature that delegates to the child;
- add a direct child unit test for signature input field shape;
- keep all existing create/restore child call sites parent-mediated.

## Forbidden BE-001NT-02 Movement

BE-001NT-02 must not move or rewrite:

- create flow child;
- read routes child;
- restore flow child;
- persistence child;
- snapshot ID validation child;
- route facade;
- canonical digest implementation;
- snapshot persistence or disk load implementation;
- restore audit persistence implementation;
- AppState memory insert/read/cleanup behavior;
- storage lifecycle or runtime persistence internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

## Parent-Child Rule

The child must stay private under the current snapshots handler implementation owner.

Allowed call paths:

- snapshots handler parent bridge -> private `handlers::signature_contract::build_signature_input`;
- create/restore children -> snapshots handler parent bridge.

Forbidden call path:

Any create/restore child importing or calling `handlers::signature_contract` directly.

## Proof

BE-001NT-02 must prove equivalence with:

- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `cargo check -p quantpilot`

## Next Step

BE-001NT-02 backend.ops_governance.snapshots.signature_contract extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
