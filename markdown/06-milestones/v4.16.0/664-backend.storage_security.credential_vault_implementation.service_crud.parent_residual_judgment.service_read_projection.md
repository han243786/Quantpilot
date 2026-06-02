# v4.16.0 backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_read_projection

> Batch: BE-001KD-01
> Node: `backend.storage_security.credential_vault_implementation.service_crud`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` is selected as the next child.

After `service_mutation_commit` closeout, `service_crud` still owns the read-only projection pocket:

- `get_service`
- `list_services`
- `VaultData.entries` lookup
- cloned `BTreeMap<String, Zeroizing<String>>` projection
- cloned service key listing
- poisoned mutex recovery for read paths

`service_mutation_commit` remains closed. This step only selects the read projection residual and does not move code.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `service_read_projection` maps directly to `get_service`, `list_services`, lookup, zeroizing clone wrapping, and key listing. |
| parent_child_communication_kept | PASS | The future child can stay below `service_crud`; `implementation.rs` remains mediated by the parent `service_crud` helper layer. |
| equivalence_baseline_freezable | PASS | Existing credential vault tests cover missing get, set/get roundtrip, list empty, and list with services. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | The child backs public `CredentialVault::get_service` and `CredentialVault::list_services` through parent delegation. |
| state_machine_phase | PASS | Read-only projection is a separate CRUD phase from mutation/save commit. |
| strategy_branch | PASS | Missing lookup, hit projection, zeroizing wrapping, empty list, and non-empty key listing are distinct branches. |
| independent_failure_mode | PASS | Read projection can regress independently from mutation validation, remove, insert, and save handoff. |
| reuse_pressure | PARTIAL | The split improves white-box review and test targeting; generic reuse is secondary. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child owns a coherent read/list projection behavior pocket. |
| communication_cost_rises | NO | A single child below `service_crud` avoids sibling horizontal links and keeps the parent facade stable. |
| local_proof_missing | NO | BE-001KC-03 passed `cargo check -p quantpilot`, `credential_vault`, and `credential` filtered tests. |
| line_count_only | NO | Selection is based on public read surface, phase boundary, and failure isolation rather than size. |

leaf_split_decision_result

`select_child: backend.storage_security.credential_vault_implementation.service_crud.service_read_projection`

Next step freezes the child baseline before code movement. `service_mutation_commit` remains closed and must not be edited in the read projection baseline.

next_recursive_step

BE-001KE-01 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_mutation_commit.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`

**Markers**:
- `BE-001KD-01`
- `parent_residual_judgment`
- `service_read_projection_selected`
- `service_mutation_commit_remains_closed`
- `no_code_movement`
- `release_transition_guard`

**Next step**:
BE-001KE-01 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection baseline_plan

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
