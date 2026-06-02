# v4.16.0 backend.storage_security.credential_vault_implementation.service_crud equivalence baseline and extraction plan

> Batch: BE-001KA-01
> Node: `backend.storage_security.credential_vault_implementation.service_crud`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.service_crud` equivalence baseline and extraction plan are frozen.

This child owns only the service CRUD behavior inside `src/backend/storage_security/credential_vault/implementation.rs`:

- `set_service`
- `get_service`
- `delete_service`
- `list_services`
- empty-field validation
- `CredentialFields` to `BTreeMap<String, SecretString>` conversion
- `VaultData.entries` insert/overwrite
- `VaultData.entries` lookup and clone into `Zeroizing<String>`
- missing-service delete error
- service key listing
- save handoff after set/delete mutation

It does not own `SecretString`, `VaultData`, `CredentialFields`, `CredentialVault` field layout, `load`, `load_from_storage_root`, `save_inner`, persistence children, secret pattern extraction, implementation-local tests, root shim behavior, or release transition.

BE-001KA-02 may move CRUD implementation bodies into a true child module under `src/backend/storage_security/credential_vault/implementation/` only if the parent `CredentialVault` public methods remain the public facade and existing behavior remains equivalent.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| set_service empty fields | Empty `CredentialFields` returns an error and does not mutate vault data. |
| set_service mutation | Non-empty fields are converted into `SecretString`, inserted under `service.to_string()`, and overwrite any existing service entry. |
| set_service persistence | Successful mutation calls parent `save_inner` with the locked `VaultData`. |
| get_service missing | Missing service returns `None`. |
| get_service hit | Existing service returns a cloned `BTreeMap<String, Zeroizing<String>>`, preserving field keys and values. |
| get_service safety wrapper | Returned values remain wrapped in `Zeroizing<String>` so caller-owned plaintext clears on drop. |
| delete_service missing | Missing service returns an error with the current message semantics and does not save. |
| delete_service hit | Existing service is removed and then persisted through `save_inner`. |
| list_services | Returns cloned service names from `VaultData.entries.keys()` without sorting or mutation. |
| Mutex behavior | Poisoned vault data locks recover through `unwrap_or_else(|e| e.into_inner())`. |
| Parent mediation | Public methods stay in `implementation.rs`; the child exposes only `pub(super)` helpers unless a later baseline explicitly moves public methods. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/service_crud.rs`.
2. Move CRUD method bodies into parent-only helper functions:
   - `set_service(vault: &CredentialVault, service: &str, fields: CredentialFields) -> Result<()>`
   - `get_service(vault: &CredentialVault, service: &str) -> Option<BTreeMap<String, Zeroizing<String>>>`
   - `delete_service(vault: &CredentialVault, service: &str) -> Result<()>`
   - `list_services(vault: &CredentialVault) -> Vec<String>`
3. Keep `CredentialVault` public methods in `implementation.rs` as stable facade methods delegating to the child.
4. Keep `SecretString`, `VaultData`, `CredentialFields`, `CredentialVault` fields, load/persistence, secret pattern extraction, tests, and root shim in place.
5. Do not change release transition behavior or introduce horizontal links to persistence children.
6. Run `cargo fmt --check`, `cargo check -p quantpilot`, `cargo test -p quantpilot credential_vault --lib`, and `cargo test -p quantpilot credential --lib`.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001KA-01`
- `service_crud baseline_frozen`
- `service_crud plan_frozen`
- `public facade retained`
- `no code movement`
- `release_transition_guard`

**Next step**:
BE-001KA-02 backend.storage_security.credential_vault_implementation.service_crud extract_closeout

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
