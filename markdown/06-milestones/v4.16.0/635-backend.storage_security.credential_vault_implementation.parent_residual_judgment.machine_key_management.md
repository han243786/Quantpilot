# v4.16.0 backend.storage_security.credential_vault_implementation parent residual judgment selects machine_key_management

> Batch: BE-001JO-01
> Node: `backend.storage_security.credential_vault_implementation`
> Parent: `backend.storage_security`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation` remains open and selects `backend.storage_security.credential_vault_implementation.machine_key_management` as the next child.

Selection reason: machine key cache/init and key derivation are upstream of every vault encryption/decryption path, but they can be frozen without moving AES-GCM nonce/tag handling, vault JSON persistence, backup restore, service CRUD, or secret pattern extraction. This makes it the safest first internal child after the implementation owner extraction.

Residual queue after this judgment:
- `machine_key_management` selected for BE-001JP-01 baseline.
- `crypto_codec` remains open.
- `vault_persistence_restore` remains open.
- `service_crud` remains open.
- `secret_pattern_extraction` remains open.
- implementation-local tests remain owned by the current implementation until a child-specific test migration is justified.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `machine_key_management` has a stable owner pocket: `MACHINE_KEYS`, `MACHINE_KEY_INIT_LOCK`, `get_machine_key_for_path`, `derive_key_from_machine_key`, and `derive_key_pbkdf2_from_machine_key`. |
| parent_child_communication_kept | PASS | Future extraction remains under `backend.storage_security.credential_vault`; callers continue through `CredentialVault` and parent helpers. |
| equivalence_baseline_freezable | PASS | BE-001JP-01 can freeze key file path behavior, cache behavior, key creation/load, PBKDF2 parameters, and failure propagation before any code movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PARTIAL | The child is internal, but all public `CredentialVault` load/mutation paths depend on its key material. |
| state_machine_phase | PASS | Machine-key initialization precedes encryption/decryption and vault persistence. |
| strategy_branch | PASS | Existing key load, new key generation, cache hit, parent directory creation, and PBKDF2 derivation are distinct branches. |
| independent_failure_mode | PASS | Key file IO, cache poisoning, random key generation, and key derivation failures can regress independently from CRUD or JSON persistence. |
| reuse_pressure | PARTIAL | Key derivation helpers may remain reused by crypto codec, but reuse pressure is secondary to safety isolation. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child owns concrete key-management symbols and security behavior. |
| communication_cost_rises | NO | Parent-owned helper calls can stay local to the credential vault module. |
| local_proof_missing | NO | Prior BE-001JM-02 gates passed `cargo check -p quantpilot`, `cargo test -p quantpilot credential_vault --lib`, and `cargo test -p quantpilot credential --lib`; BE-001JP-01 will freeze child-specific proof. |
| line_count_only | NO | The selection is based on initialization order, security failure mode, and branch isolation. |

leaf_split_decision_result

`baseline_required`

Selected child: `backend.storage_security.credential_vault_implementation.machine_key_management`.

BE-001JP-01 must freeze the child boundary before any movement:
- Owns: machine-key cache, key init lock, key file load/create, key material derivation.
- Does not own: AES-GCM encrypt/decrypt, nonce/tag handling, vault JSON schema, backup restore, atomic save, service CRUD, secret pattern extraction, or release transition.

next_recursive_step

BE-001JP-01 backend.storage_security.credential_vault_implementation.machine_key_management baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001JO-01`
- `leaf_split_decision_gate`
- `machine_key_management selected`
- `stop_split false`
- `baseline_required`
- `release_transition_guard`

**Next step**:
BE-001JP-01 backend.storage_security.credential_vault_implementation.machine_key_management baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
