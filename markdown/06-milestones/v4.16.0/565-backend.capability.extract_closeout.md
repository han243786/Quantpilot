# v4.16.0 backend.capability actual extraction and closeout complete

> Batch: BE-001HN-02
> Node: `backend.capability`
> Parent: `backend`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.capability actual extraction and closeout complete.

`src/backend/capability/snapshot.rs` now owns capability response construction,
capability contract/hash/context helpers, runtime governance snapshot, and UI
capability projection. `src/capability_api.rs` remains only as a root
compatibility shim for existing crate-root imports.

---

## Movement

Moved into `src/backend/capability/snapshot.rs`:

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
unsupported_frontend_module_reasons
workspace_surface_capabilities
ui_action_capabilities
```

Compatibility marker kept:

```text
src/capability_api.rs re-exports backend.capability.snapshot for existing internal root imports.
```

---

## Equivalence Evidence

Capability hash/contract:

```text
cargo test -p quantpilot capability_contract --lib
```

Runtime guard:

```text
cargo test -p quantpilot runtime_write_rejects_missing_capability_context_without_creating_run
```

Frontend capability governance:

```text
cd frontend
npm.cmd test -- --run src/capabilities/capabilityGovernance.test.js src/capabilities/supportMatrix.test.js
```

Parent-child communication:

```text
HTTP route still flows app_router -> interface_boundary -> backend.capability -> snapshot.
Root capability_api is a compatibility shim only; it does not own capability behavior.
No sibling backend child was touched.
No release-transition optimization was introduced.
```

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `backend.capability.snapshot` now owns the capability snapshot/contract behavior. |
| parent_child_communication_kept | pass | Route flow stays parent-mediated; root shim is compatibility only. |
| equivalence_baseline_freezable | pass | BE-001HN-01 froze response/hash/context behavior before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | closed | `/api/capabilities` behavior now resolves through the backend capability child owner. |
| state_machine_phase | false | Capability remains a snapshot/contract domain. |
| strategy_branch | false | No strategy/runtime child behavior was moved. |
| independent_failure_mode | closed | Capability hash/context drift and unsupported-module reason behavior are isolated in the child. |
| reuse_pressure | closed | Crate-wide helper reuse is served through the root compatibility shim. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | true | Further split would need a new internal capability parent judgment. |
| communication_cost_rises | false | Current movement reduces old-owner delegation without adding sibling calls. |
| local_proof_missing | false | Rust capability tests, runtime guard test, and frontend capability tests passed. |
| line_count_only | true | Any further capability split must be contract-domain driven, not line-count driven. |

leaf_split_decision_result

`stop_split: true` for `backend.capability` at this level.

next_recursive_step

BE-001HO-01 backend parent residual judgment

## Boundary

**Real files**:
- `src/backend/capability.rs`
- `src/backend/capability/snapshot.rs`
- `src/capability_api.rs`

**Markers**:
- `capability actual_extraction_done`
- `capability closeout_done`
- `capability stop_split: true`
- `capability_api compatibility shim kept`

**Next step**:
BE-001HO-01 backend parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot capability_contract --lib`
- `cargo test -p quantpilot runtime_write_rejects_missing_capability_context_without_creating_run`
- `cd frontend && npm.cmd test -- --run src/capabilities/capabilityGovernance.test.js src/capabilities/supportMatrix.test.js`
