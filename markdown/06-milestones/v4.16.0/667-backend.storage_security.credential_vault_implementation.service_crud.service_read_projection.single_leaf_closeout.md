# v4.16.0 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single leaf closeout stops further split

> Batch: BE-001KE-03
> Node: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection`
> Parent: `backend.storage_security.credential_vault_implementation.service_crud`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` is closed as a terminal child.

The child now owns the complete read-only projection pocket:

- `get_service` missing lookup and hit projection
- cloned `BTreeMap<String, Zeroizing<String>>` values for caller-owned plaintext cleanup
- `list_services` cloned key listing
- shared poisoned-lock recovery

Splitting again into `get_service` and `list_services` micro leaves would duplicate lock/read scaffolding and add another delegation hop without producing a stronger parent boundary. `service_crud` now has both known children closed and can move to parent closeout.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | The node has a stable name and owns the complete service read/list projection pocket. |
| parent_child_communication_kept | PASS | It remains below `service_crud`; `implementation.rs` reaches it only through the parent `service_crud` mediation layer. |
| equivalence_baseline_freezable | PASS | Existing credential vault tests cover missing get, set/get roundtrip, empty list, and non-empty list after BE-001KE-02 extraction. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | The child backs `CredentialVault::get_service` and `CredentialVault::list_services` through parent delegation. |
| state_machine_phase | PASS | It owns the read-only CRUD projection phase, separate from mutation/save commit. |
| strategy_branch | PASS | Missing lookup, hit projection, zeroizing wrapping, empty list, and non-empty key listing are covered branches. |
| independent_failure_mode | PASS | Read/list projection failures are isolated from mutation validation, insert/remove, and save handoff failures. |
| reuse_pressure | PARTIAL | The current split improves review and test targeting; further reuse pressure is not present. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | YES | Further split would create per-method read micro leaves that mostly repeat lock/read scaffolding. |
| communication_cost_rises | YES | Adding grandchildren below read projection would add a delegation hop without a new parent-child contract. |
| local_proof_missing | NO | BE-001KE-02 passed `cargo check -p quantpilot`, `credential_vault`, and `credential` filtered tests. |
| line_count_only | NO | Stop decision is based on ownership and communication cost, not file size. |

leaf_split_decision_result

`stop_split_true`

`backend.storage_security.credential_vault_implementation.service_crud.service_read_projection stop_split: true`.

The next recursive step closes the `service_crud` parent because both known children, `service_mutation_commit` and `service_read_projection`, are now terminal.

next_recursive_step

BE-001KF-01 backend.storage_security.credential_vault_implementation.service_crud parent_closeout
## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_read_projection.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`

**Markers**:
- `BE-001KE-03`
- `leaf_split_decision_gate`
- `service_read_projection_stop_split_true`
- `service_crud_children_closed`
- `return_parent_closeout`
- `release_transition_guard`

**Next step**:
BE-001KF-01 backend.storage_security.credential_vault_implementation.service_crud parent_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential_vault --lib`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
