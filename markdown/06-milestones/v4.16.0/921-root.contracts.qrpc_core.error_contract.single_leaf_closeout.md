# v4.16.0 root.contracts.qrpc_core.error_contract single leaf closeout

> Batch: BE-001PP-01
> Node: `root.contracts.qrpc_core.error_contract`
> Parent: `root.contracts.qrpc_core`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.error_contract` has been evaluated as the typed core error owner selected by BE-001PO-01.

Decision:

`stop_split: true`

The node remains equivalent because `qrpc_core/src/error.rs` was not edited.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Independent physical owner | Stop. The typed core error contract is already one compact file. |
| Public contract surface | Covered. The leaf owns `QuantPilotError`, display formatting, source behavior, and IO conversion. |
| Different behavior families | Not enough to split. Enum variants and formatting are one caller-facing error contract. |
| Micro-module risk | Stop. Splitting variants, display, source, and conversion would fragment one coherent error owner. |
| Future reopen rule | Allowed only when a concrete error variant, formatting, source, or conversion change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Core error contract maintenance proposal | `contracts.qrpc_core.error_contract` | Updated or verified `qrpc_core/src/error.rs` typed error contract |

The leaf may describe and guard:

- `QuantPilotError` variants;
- `Display` output contract;
- `std::error::Error::source` behavior;
- `From<std::io::Error>` conversion;
- caller matching assumptions around typed errors.

## Non-Claims

This closeout does not claim:

- error variants or display strings changed;
- Strategy IR, plugin contract, event proto, or `lib.rs` protocol contracts changed;
- compiler, runtime, backend, executor, frontend, or E2E behavior changed;
- release transition was opened.

## Next Step

BE-001PQ-01 `root.contracts.qrpc_core` parent_residual_judgment selects `contracts.qrpc_core.event_envelope_proto`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
