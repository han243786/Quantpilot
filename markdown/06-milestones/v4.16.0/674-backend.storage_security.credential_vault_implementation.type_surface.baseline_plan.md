# v4.16.0 backend.storage_security.credential_vault_implementation.type_surface equivalence baseline and extraction plan

> Batch: BE-001KJ-01
> Node: `backend.storage_security.credential_vault_implementation.type_surface`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.type_surface` equivalence baseline and extraction plan are frozen.

This child owns only the shared type/public facade surface currently embedded in `src/backend/storage_security/credential_vault/implementation.rs`:

- `SecretString` wrapper, plaintext serialization/deserialization, and `Drop` zeroize behavior
- `VaultData` persisted storage shape and `entries` map field
- public `CredentialFields = BTreeMap<String, String>` alias
- public `CredentialVault` owner struct and field layout
- `storage_root` environment fallback for `QUANTPILOT_STORAGE_ROOT`
- visibility required by closed children to access `SecretString`, `VaultData`, `CredentialVault`, and `CredentialFields` through the parent boundary

It does not own public method behavior (`load`, CRUD, persistence save/load, secret extraction), child module bodies, implementation-local tests, root compatibility shim, or release transition.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Public exports | `backend::storage_security::credential_vault` continues to export `CredentialFields` and `CredentialVault`. |
| Root shim | `src/credential_vault.rs` continues to re-export `CredentialFields` and `CredentialVault` through the backend facade. |
| Secret wrapper | `SecretString` serializes/deserializes as plaintext `String` and zeroizes its inner string on `Drop`. |
| Storage shape | `VaultData` remains `#[derive(Debug, Clone, Serialize, Deserialize, Default)]` with `#[serde(deny_unknown_fields)]` and `entries: BTreeMap<String, BTreeMap<String, SecretString>>`. |
| Credential fields | `CredentialFields` remains a public alias of `BTreeMap<String, String>`. |
| Vault fields | `CredentialVault` retains `path: PathBuf`, `machine_key: [u8; 32]`, and `data: Mutex<VaultData>`. |
| Storage root | `storage_root()` continues to read `QUANTPILOT_STORAGE_ROOT` and fall back to `"storage"`. |
| Parent mediation | Behavior methods stay in `implementation.rs`; closed children continue to communicate through the parent and shared type surface. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/type_surface.rs`.
2. Move only `storage_root`, `SecretString`, `VaultData`, `CredentialFields`, and `CredentialVault` into the child.
3. Keep `implementation.rs` as the parent behavior facade by re-exporting `CredentialFields` and `CredentialVault`, and by exposing `SecretString`, `VaultData`, and `storage_root` only to the parent/children as needed.
4. Preserve child-module access with explicit `pub(super)` visibility for tuple fields and struct fields that siblings already rely on.
5. Do not move `CredentialVault` method bodies, tests, child modules, `src/credential_vault.rs`, or any release-transition connection.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`

**Planned child file**:
- `src/backend/storage_security/credential_vault/implementation/type_surface.rs`

**Markers**:
- `BE-001KJ-01`
- `type_surface_baseline_frozen`
- `type_surface_plan_frozen`
- `visibility_boundary_frozen`
- `no_code_movement`
- `release_transition_guard`

**Next step**:
BE-001KJ-02 backend.storage_security.credential_vault_implementation.type_surface extract_closeout

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
