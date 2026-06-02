# v4.16.0 backend.storage_security.credential_vault_implementation.crypto_codec equivalence baseline and extraction plan

> Batch: BE-001JR-01
> Node: `backend.storage_security.credential_vault_implementation.crypto_codec`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.crypto_codec` equivalence baseline and extraction plan are frozen.

This child owns only the codec pocket inside `src/backend/storage_security/credential_vault/implementation.rs`:

- `NONCE_LEN = 12`
- `TAG_LEN = 16`
- `encrypt_with_machine_key`
- `decrypt_with_machine_key`

It may call the already closed `machine_key_management` derivation helpers, but it does not own their internals. It also does not own vault JSON persistence, backup restore, atomic save, service CRUD, secret pattern extraction, root compatibility shim behavior, or release transition.

BE-001JR-02 may move this owner pocket into a true child module under `src/backend/storage_security/credential_vault/implementation/` only if `implementation.rs` continues to mediate all calls and existing `CredentialVault` behavior remains equivalent.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Encrypt version | `encrypt_with_machine_key` must always emit version byte `2` before nonce and sealed payload. |
| Key derivation use | Encrypt uses `derive_key_pbkdf2_from_machine_key`; decrypt version `2` uses PBKDF2, version `1` uses SHA-256, and legacy unversioned payloads use SHA-256 with offset `0`. |
| Nonce/tag sizes | `NONCE_LEN` remains `12`; `TAG_LEN` remains `16`; payloads shorter than nonce + tag after offset fail before AES-GCM open. |
| AAD | AES-GCM seal/open must keep `Aad::from(".credentials".as_bytes())`. |
| Nonce generation | Encrypt generates random nonce bytes with `SystemRandom` and prepends them after the version byte. |
| Payload layout | v2 encrypted output remains `[2][12-byte nonce][ciphertext + tag]`. |
| Decrypt routing | Decrypt reads `ciphertext[0]` as version; offsets remain `1` for v1/v2 and `0` for legacy. |
| Error behavior | Empty ciphertext, short/corrupt payload, random failure, seal failure, and open failure continue to return `anyhow` errors with equivalent semantics. |
| Plaintext handling | Decrypt still truncates opened data to plaintext length and returns `Zeroizing<String>` with `String::from_utf8(data).unwrap_or_default()`. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/crypto_codec.rs`.
2. Move `NONCE_LEN`, `TAG_LEN`, `encrypt_with_machine_key`, and `decrypt_with_machine_key` into that child.
3. Keep codec functions `pub(super)` so only `implementation.rs` can call them.
4. Import `derive_key_from_machine_key` and `derive_key_pbkdf2_from_machine_key` from sibling `machine_key_management` only through the implementation parent boundary.
5. Keep vault load/save, backup restore, atomic persistence, service CRUD, secret pattern extraction, tests, and root shim in place.
6. Run `cargo fmt --check`, `cargo check -p quantpilot`, `cargo test -p quantpilot credential_vault --lib`, and `cargo test -p quantpilot credential --lib`.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/machine_key_management.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001JR-01`
- `crypto_codec baseline_frozen`
- `crypto_codec plan_frozen`
- `no code movement`
- `release_transition_guard`

**Next step**:
BE-001JR-02 backend.storage_security.credential_vault_implementation.crypto_codec extract_closeout

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
