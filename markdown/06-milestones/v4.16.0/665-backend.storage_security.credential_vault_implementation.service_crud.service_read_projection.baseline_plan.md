# v4.16.0 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection equivalence baseline and extraction plan

> Batch: BE-001KE-01
> Node: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection`
> Parent: `backend.storage_security.credential_vault_implementation.service_crud`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` equivalence baseline and extraction plan are frozen.

This child owns only the read-only service projection behavior inside `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs`:

- `get_service`
- `list_services`
- `VaultData.entries` lookup by service label
- missing-service `None` result
- cloned `BTreeMap<String, Zeroizing<String>>` projection for existing entries
- service key listing through cloned `VaultData.entries.keys()`
- poisoned mutex recovery through `unwrap_or_else(|e| e.into_inner())`

It does not own `set_service`, `delete_service`, mutation/save behavior, `CredentialVault` public facade methods, parent-owned types, tests, root shim behavior, or release transition.

BE-001KE-02 may move only read projection helpers into `src/backend/storage_security/credential_vault/implementation/service_crud/service_read_projection.rs`. `service_crud/mod.rs` must remain the parent child that mediates calls from `implementation.rs`.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| get_service input | Receives `&CredentialVault` and service label by `&str`. |
| get_service lock recovery | Vault data mutex poisoning is recovered through `unwrap_or_else(|e| e.into_inner())`. |
| get_service missing | Missing service returns `None`. |
| get_service hit | Existing service returns a cloned `BTreeMap` preserving field keys and values. |
| get_service safety wrapper | Returned plaintext values remain wrapped in caller-owned `Zeroizing<String>`. |
| get_service mutation isolation | Read projection does not mutate entries and does not call `save_inner`. |
| list_services input | Receives `&CredentialVault` only. |
| list_services lock recovery | List uses the same poisoned-lock recovery behavior as get. |
| list_services output | Returns cloned service names from `VaultData.entries.keys()` without sorting or mutation. |
| Parent mediation | `service_crud` remains the parent module; `implementation.rs` public methods keep delegating to `service_crud`. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/service_crud/service_read_projection.rs`.
2. Move only the bodies of `get_service` and `list_services` into child helpers with the same signatures.
3. Keep parent `service_crud::get_service` and `service_crud::list_services` as delegating helpers so `implementation.rs` remains unchanged.
4. Keep `service_mutation_commit.rs` closed and unchanged.
5. Keep parent-owned types, tests, root shim, persistence children, and release transition unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_read_projection.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_mutation_commit.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`

**Markers**:
- `BE-001KE-01`
- `service_read_projection_baseline_frozen`
- `service_read_projection_plan_frozen`
- `service_mutation_commit_remains_closed`
- `no_code_movement`
- `release_transition_guard`

**Next step**:
BE-001KE-02 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection extract_closeout

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
