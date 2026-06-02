# v4.16.0 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit equivalence baseline and extraction plan

> Batch: BE-001KC-01
> Node: `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit`
> Parent: `backend.storage_security.credential_vault_implementation.service_crud`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` equivalence baseline and extraction plan are frozen.

This child owns only the mutating service CRUD behavior inside `src/backend/storage_security/credential_vault/implementation/service_crud.rs`:

- `set_service`
- `delete_service`
- empty-field validation before mutation
- `CredentialFields` to `BTreeMap<String, SecretString>` conversion
- `VaultData.entries` insert/overwrite under `service.to_string()`
- `VaultData.entries` remove
- missing-service delete error
- save handoff through parent `CredentialVault::save_inner` after successful set/delete
- poisoned mutex recovery through `unwrap_or_else(|e| e.into_inner())`

It does not own `get_service`, `list_services`, read projection, `Zeroizing<String>` wrapping, `CredentialVault` public facade methods, parent-owned types, tests, root shim behavior, or release transition.

BE-001KC-02 may convert `service_crud.rs` into a true parent module directory and move only mutation/save helpers into `src/backend/storage_security/credential_vault/implementation/service_crud/service_mutation_commit.rs`. `service_crud` must remain the parent child that mediates calls from `implementation.rs`.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| set_service input | Receives `&CredentialVault`, service label, and owned `CredentialFields`. |
| set_service empty fields | Empty fields return the current error message and do not mutate or save. |
| set_service lock recovery | Vault data mutex poisoning is recovered through `unwrap_or_else(|e| e.into_inner())`. |
| set_service conversion | Non-empty fields are converted into `BTreeMap<String, SecretString>` without filtering or sorting. |
| set_service insert | Entry is inserted under `service.to_string()` and overwrites any existing entry. |
| set_service persistence | Successful mutation calls `vault.save_inner(&data)` before returning. |
| delete_service lock recovery | Delete uses the same poisoned-lock recovery behavior as set. |
| delete_service missing | Missing service returns the current missing-label error and does not save. |
| delete_service hit | Existing service is removed and then persisted through `vault.save_inner(&data)`. |
| Parent mediation | `service_crud` remains the parent module; `implementation.rs` public methods keep delegating to `service_crud`. |

## Extraction Plan

1. Replace `src/backend/storage_security/credential_vault/implementation/service_crud.rs` with directory parent module `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs`.
2. Create `src/backend/storage_security/credential_vault/implementation/service_crud/service_mutation_commit.rs`.
3. Move only the bodies of `set_service` and `delete_service` into child helpers with the same signatures.
4. Keep parent `service_crud::set_service` and `service_crud::delete_service` as delegating helpers so `implementation.rs` remains unchanged.
5. Keep `get_service` and `list_services` in `service_crud/mod.rs`; they remain residual for `service_read_projection`.
6. Keep parent-owned types, tests, root shim, persistence children, and release transition unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation/service_crud.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_mutation_commit.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`

**Markers**:
- `BE-001KC-01`
- `service_mutation_commit_baseline_frozen`
- `service_mutation_commit_plan_frozen`
- `service_read_projection_remains_residual`
- `no_code_movement`
- `release_transition_guard`

**Next step**:
BE-001KC-02 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit extract_closeout

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
