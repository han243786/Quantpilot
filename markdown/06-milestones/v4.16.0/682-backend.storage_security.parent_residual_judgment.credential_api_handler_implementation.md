# v4.16.0 backend.storage_security parent residual judgment selects credential_api_handler_implementation

> Batch: BE-001KN-01
> Node: `backend.storage_security`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation` is selected as the next child.

`backend.storage_security.credential_api`, `backend.storage_security.credential_vault`, and `backend.storage_security.credential_vault_implementation` are now closed. The remaining credential security residual is the paused handler owner in `src/credential_api.rs`.

This selection does not move handler code yet. BE-001KO-01 must first freeze the handler-level safety semantics:

- route paths and HTTP methods
- `UserId` extraction and user-scoped credential keys
- service-name validation
- empty-field rejection
- vault load/error handling
- audit logging calls
- status-code and JSON response shape

The planned physical direction is parent-mediated: keep the route facade visible under `backend.storage_security.credential_api`, introduce a handler implementation child under `backend.storage_security`, and make the storage-security parent mediate any route-facade-to-handler call. No child-to-child shortcut is allowed.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | The selected child is `backend.storage_security.credential_api_handler_implementation`. |
| parent_child_communication_kept | PASS | The next plan must route through the `backend.storage_security` parent bridge instead of a credential_api facade child calling a handler sibling directly. |
| equivalence_baseline_freezable | PASS | The root handler owner has a compact surface that can freeze route paths, auth scoping, validation, audit, vault calls, status codes, and response JSON before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | `src/credential_api.rs` owns the `/api/credentials` list/set/delete handler boundary. |
| state_machine_phase | PARTIAL | The handlers execute a request lifecycle: authenticate user, validate input, scope service key, call vault, return JSON/status. |
| strategy_branch | PASS | List, set, and delete are separate security branches with distinct validation and failure modes. |
| independent_failure_mode | PASS | Credential handler regressions can occur independently from vault crypto/persistence and from the route facade. |
| reuse_pressure | PARTIAL | The handlers are not broadly reused, but their boundary is reused through app route composition and interface boundary registration. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | This is a real handler owner, not a facade/import micro leaf. |
| communication_cost_rises | NO | Parent-mediated extraction can reduce root-level security sprawl while preserving explicit route ownership. |
| local_proof_missing | NO | The next baseline can use `cargo check` and credential route/vault focused proof before any movement. |
| line_count_only | NO | Selection is driven by handler safety ownership, not file size. |

leaf_split_decision_result

`select_child: backend.storage_security.credential_api_handler_implementation`

`backend.storage_security stop_split: false`.

The route facade closeout from BE-001JH-01 remains valid for facade splitting only. The handler implementation was deliberately paused outside that facade leaf and is now selected as a separate safety-baselined child.

next_recursive_step

BE-001KO-01 backend.storage_security.credential_api_handler_implementation baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security.rs`
- `src/backend/storage_security/credential_api.rs`
- `src/credential_api.rs`

**Markers**:
- `BE-001KN-01`
- `select_credential_api_handler_implementation`
- `handler_safety_baseline_required`
- `parent_mediated_credential_route_bridge`
- `release_transition_guard`

**Next step**:
BE-001KO-01 backend.storage_security.credential_api_handler_implementation baseline_plan

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
