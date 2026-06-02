# v4.16.0 backend.storage_security.credential_vault_implementation parent residual judgment selects crypto_codec

> Batch: BE-001JQ-01
> Node: `backend.storage_security.credential_vault_implementation`
> Parent: `backend.storage_security`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation` parent residual judgment selects `backend.storage_security.credential_vault_implementation.crypto_codec`.

`machine_key_management` is now closed. The next cohesive residual is the crypto codec pocket in `src/backend/storage_security/credential_vault/implementation.rs`: nonce/tag constants, encrypt framing, decrypt version selection, AES-GCM seal/open, and versioned ciphertext payload handling.

Residual queue after this judgment:
- `crypto_codec` selected for BE-001JR-01 baseline.
- `vault_persistence_restore` remains open.
- `service_crud` remains open.
- `secret_pattern_extraction` remains open.
- implementation-local tests remain owned by the current implementation until child-specific migration is justified.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `crypto_codec` can be named around `NONCE_LEN`, `TAG_LEN`, `encrypt_with_machine_key`, and `decrypt_with_machine_key`. |
| parent_child_communication_kept | PASS | Future extraction can be a true child of `implementation.rs`, with callers still mediated by `CredentialVault`. |
| equivalence_baseline_freezable | PASS | BE-001JR-01 can freeze version headers, nonce/tag lengths, AAD, v1/v2 derivation selection, corrupt payload handling, and UTF-8 fallback before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PARTIAL | Public vault load/save behavior depends on it, but the codec remains internal. |
| state_machine_phase | PASS | Crypto encoding/decoding is a distinct phase between machine-key derivation and vault persistence/CRUD. |
| strategy_branch | PASS | Encrypt v2 output, decrypt v2, decrypt v1, legacy unversioned decrypt, short/corrupt payload, and open failure are separate branches. |
| independent_failure_mode | PASS | Nonce/tag framing, AAD, version offsets, and AES-GCM open/seal can regress independently from key bootstrap or service CRUD. |
| reuse_pressure | NO | Current use remains local to credential vault implementation. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child owns concrete codec symbols and security behavior. |
| communication_cost_rises | NO | A single child can own the codec without adding sibling horizontal links. |
| local_proof_missing | NO | Existing credential_vault tests cover load/save roundtrips and corrupt/missing branches indirectly; BE-001JR-01 will freeze child-specific evidence before movement. |
| line_count_only | NO | The selection is based on phase and failure-mode isolation, not file size. |

leaf_split_decision_result

`baseline_required`

Selected child: `backend.storage_security.credential_vault_implementation.crypto_codec`.

BE-001JR-01 must freeze the child boundary before any movement:
- Owns: nonce/tag constants, versioned ciphertext framing, AES-GCM seal/open, AAD, decrypt version routing, corrupt payload handling.
- Does not own: machine-key cache/init, key file IO, key derivation internals, vault JSON persistence, backup restore, atomic save, service CRUD, secret pattern extraction, root compatibility shim, or release transition.

next_recursive_step

BE-001JR-01 backend.storage_security.credential_vault_implementation.crypto_codec baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/machine_key_management.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001JQ-01`
- `leaf_split_decision_gate`
- `crypto_codec selected`
- `machine_key_management closed`
- `baseline_required`
- `release_transition_guard`

**Next step**:
BE-001JR-01 backend.storage_security.credential_vault_implementation.crypto_codec baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
