# v4.16.0 backend.storage_security safety equivalence baseline and extraction plan

> Batch: BE-001JD-01
> Node: `backend.storage_security`
> Parent: `backend`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.storage_security safety equivalence baseline and extraction plan

Frozen current boundary:

- `src/backend/storage_security.rs` owns the parent facade, `register_credential_routes`, and `CredentialVault` re-export.
- `src/backend/storage_security/credential_api.rs` owns only the credential route facade that delegates to `src/credential_api.rs`.
- `src/backend/storage_security/credential_vault.rs` owns only the vault type re-export facade that delegates to `src/credential_vault.rs`.

Safety pause remains active for:

- `src/credential_api.rs`
- `src/credential_vault.rs`
- `src/storage_lifecycle.rs`
- `src/safe_log.rs`
- `src/auth/mod.rs`
- `src/auth_middleware.rs`
- `src/rate_limiter.rs`
- `src/backup.rs`

Allowed next movement:

- Confirm the already-existing storage/security parent facade as an extraction closeout.
- Do not move sensitive implementation yet; after closeout, choose children only through explicit safety baseline gates.

Forbidden next movement:

- Do not change secret persistence, encryption, auth middleware, quota checks, atomic writes, TTL cleanup, backup behavior, or log redaction.
- Do not expose new credential routes or bypass `backend.interface_boundary`.
- Do not create sibling horizontal links from unrelated backend leaves into storage/security internals.

## Boundary

**Real files**:
- `src/backend/storage_security.rs`
- `src/backend/storage_security/credential_api.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_api.rs`
- `src/credential_vault.rs`
- `src/storage_lifecycle.rs`
- `src/safe_log.rs`
- `src/auth/mod.rs`
- `src/auth_middleware.rs`
- `src/rate_limiter.rs`
- `src/backup.rs`

**Markers**:
- `BE-001JD-01`
- `safety_baseline_frozen`
- `credential_api facade`
- `credential_vault facade`
- `auth storage safe_log backup paused`
- `release_transition_guard`

**Next step**:
BE-001JD-02 backend.storage_security extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
