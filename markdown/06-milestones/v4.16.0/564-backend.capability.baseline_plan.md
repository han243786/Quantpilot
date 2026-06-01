# v4.16.0 backend.capability equivalence baseline and extraction plan

> Batch: BE-001HN-01
> Node: `backend.capability`
> Parent: `backend`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HN-02 `backend.capability` `extract_closeout`

---

## Summary

This baseline freezes the capability snapshot/contract residual before moving
it out of the old root-level `src/capability_api.rs` implementation owner.
The next movement may extract capability response, contract/hash/context,
versioning summary, permission boundary summary, runtime governance snapshot,
and UI capability projection into `src/backend/capability/snapshot.rs`.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001HN-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / capability snapshot boundary | capability residual freeze |
| Guidance matrix | `root.backend.capability` | planned real owner |
| Module tree | `backend -> capability -> snapshot` | capability child becomes real implementation owner |

---

## Equivalence Baseline

Frozen owner before extraction:

```text
src/capability_api.rs
```

Frozen child facade:

```text
src/backend/capability/snapshot.rs
```

Frozen public route behavior:

```text
GET /api/capabilities returns CapabilityResponse from build_capability_response.
```

Frozen helper cluster:

```text
CapabilityContract
get_capabilities
build_capability_response
current_capability_hash
current_capability_context
build_capability_contract
capability_contract_hash
capability_versioning_summary
capability_permission_boundary_summary
runtime_governance_snapshot
supported_named_capability
unsupported_frontend_module_reasons
workspace_surface_capabilities
ui_action_capabilities
```

Frozen behavior:

```text
Capability API/schema/runtime governance/versioning constants stay unchanged.
Declared/supported module keys and unsupported module reasons stay unchanged.
Canonical hash payload and stable JSON hashing stay unchanged.
Runtime governance snapshot shape and capability_context fields stay unchanged.
Workspace surface and UI action capability lists stay unchanged.
```

Frozen non-goals:

```text
No capability contract semantic changes.
No frontend static capability replacement.
No runtime/graph/strategy_config movement.
No public API path or response schema change.
No release-transition optimization.
```

---

## Extraction Plan

Planned real owner:

```text
src/backend/capability/snapshot.rs
```

Planned compatibility result:

```text
src/capability_api.rs remains as a private root compatibility shim re-exporting the child owner for existing crate-root imports.
```

Planned visibility:

```text
Capability helpers that are reused across the crate become pub(crate) in the child owner.
Public HTTP surface remains mediated by backend.capability -> snapshot.
```

The child owns only:

```text
capability response construction
capability contract construction
capability hash/context helpers
runtime governance snapshot helper
unsupported frontend module reason map
workspace surface capability list
UI action capability list
```

The parent/root keeps ownership of:

```text
backend.capability entrypoint wrapper
interface_boundary capability bridge
root compatibility shim for existing internal imports
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. Capability is a compact declarative snapshot/contract domain.
2. Existing backend child facade already names the owner.
3. Compatibility can be kept through a root shim while moving implementation.
4. Existing capability/governance tests can verify no response or hash drift.

## Boundary

**Real files**:
- `src/backend/capability.rs`
- `src/backend/capability/snapshot.rs`
- `src/capability_api.rs`
- `src/lib.rs`

**Markers**:
- `capability baseline_frozen`
- `capability plan_frozen`
- `capability_api compatibility shim planned`

**Next step**:
BE-001HN-02 backend.capability extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `npm.cmd test -- --run src/capabilities/capabilityGovernance.test.js src/capabilities/supportMatrix.test.js`
- `cargo test -p quantpilot runtime_write_rejects_missing_capability_context_without_creating_run`
