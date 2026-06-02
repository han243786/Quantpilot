# v4.16.0 backend.storage_security.credential_vault_implementation.implementation_test_harness equivalence baseline and extraction plan

> Batch: BE-001KL-01
> Node: `backend.storage_security.credential_vault_implementation.implementation_test_harness`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.implementation_test_harness` equivalence baseline and extraction plan are frozen.

This child owns only the inline `#[cfg(test)] mod tests` harness currently embedded in `src/backend/storage_security/credential_vault/implementation.rs`:

- `VAULT_TEST_LOCK` and `vault_lock`
- `VaultTestEnv` temp storage setup, `.machine_key` seed, `clean_credentials`, `load_vault`, and cleanup
- `run_vault_test` serialization guard
- 15 unit tests covering load, CRUD, persistence, list, and secret extraction behavior

It does not own production method bodies, child module bodies, shared type surface, root compatibility shim, or release transition.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Harness module | The test owner remains `#[cfg(test)] mod tests` under the credential vault implementation parent. |
| Parent access | The harness continues to use `super::*` and access parent-visible test helpers/types through the parent boundary. |
| Serialization guard | `VAULT_TEST_LOCK` serializes CWD/storage-sensitive vault tests. |
| Test environment | `VaultTestEnv` creates a temp storage root, seeds `.machine_key`, and cleans `.credentials`, `.tmp`, and `.bak` files. |
| Load tests | Fresh vault creates credentials file; existing vault reload preserves service list. |
| CRUD tests | Set/get roundtrip, empty-field rejection, overwrite, missing get, delete success/error, and delete persistence remain covered. |
| List tests | Empty and multi-service list behavior remain covered. |
| Extraction tests | Long value extraction, 3-char skip, 4-char retain, and `Zeroizing<String>` output remain covered. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/tests.rs`.
2. Move the existing inline `#[cfg(test)] mod tests` body into that child module without changing assertions.
3. Replace the inline block in `implementation.rs` with `#[cfg(test)] mod tests;`.
4. Keep production method bodies, child module bodies, `type_surface.rs`, root shim, and release transition unchanged.
5. Re-run both credential filtered test sets after movement.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`

**Planned child file**:
- `src/backend/storage_security/credential_vault/implementation/tests.rs`

**Markers**:
- `BE-001KL-01`
- `implementation_test_harness_baseline_frozen`
- `test_harness_plan_frozen`
- `no_code_movement`
- `release_transition_guard`

**Next step**:
BE-001KL-02 backend.storage_security.credential_vault_implementation.implementation_test_harness extract_closeout

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
