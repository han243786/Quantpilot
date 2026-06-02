# v4.16.0 backend.storage_security.credential_vault_implementation.secret_pattern_extraction equivalence baseline and extraction plan

> Batch: BE-001KH-01
> Node: `backend.storage_security.credential_vault_implementation.secret_pattern_extraction`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.secret_pattern_extraction` equivalence baseline and extraction plan are frozen.

This child owns only the safe-log redaction pattern extraction behavior inside `src/backend/storage_security/credential_vault/implementation.rs`:

- public `CredentialVault::extract_secret_patterns`
- lock recovery on parent `VaultData`
- traversal of all service entries and all credential field values
- cloning each stored `SecretString` into caller-owned `Zeroizing<String>`
- filtering cloned values by the current real threshold `len() >= 4`
- returning collected redaction patterns without mutation or persistence

It does not own `SecretString`, `VaultData`, `CredentialFields`, `CredentialVault` field layout, `load`, `save_inner`, service CRUD children, persistence children, implementation-local test harness, root shim behavior, or release transition.

BE-001KH-02 may move only this method body into a true child module under `src/backend/storage_security/credential_vault/implementation/`. It must also add a focused threshold guard proving a 4-character secret is extracted, because current tests cover long values and 3-character skipping but do not distinguish `len() >= 4` from wider thresholds.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Public facade | `CredentialVault::extract_secret_patterns(&self) -> Vec<Zeroizing<String>>` remains the public method. |
| Lock recovery | The vault data mutex recovers poisoning through `unwrap_or_else(|e| e.into_inner())`. |
| Traversal | Iterates all `VaultData.entries` service maps and all field values. |
| Clone wrapping | Each selected stored secret is cloned into caller-owned `Zeroizing<String>`. |
| Threshold | Values are retained when `len() >= 4` and skipped when `len() < 4`. |
| Empty result | If no retained value exists, returns an empty `Vec`. |
| No mutation | Extraction does not mutate entries and does not call `save_inner`. |
| Parent mediation | Public method stays in `implementation.rs` and delegates to a child helper after extraction. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/secret_pattern_extraction.rs`.
2. Move only the extraction body into a `pub(super)` helper accepting `&CredentialVault`.
3. Keep public `CredentialVault::extract_secret_patterns` in `implementation.rs` as a facade delegating to the child.
4. Add or tighten a unit test so a 4-character secret is included and a 3-character secret remains skipped.
5. Keep service CRUD, persistence, machine key, crypto codec, parent-owned types, root shim, and release transition unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`

**Markers**:
- `BE-001KH-01`
- `secret_pattern_extraction_baseline_frozen`
- `secret_pattern_extraction_plan_frozen`
- `threshold_len_4_frozen`
- `no_code_movement`
- `release_transition_guard`

**Next step**:
BE-001KH-02 backend.storage_security.credential_vault_implementation.secret_pattern_extraction extract_closeout

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
