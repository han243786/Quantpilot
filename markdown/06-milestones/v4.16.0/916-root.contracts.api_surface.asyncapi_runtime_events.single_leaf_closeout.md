# v4.16.0 root.contracts.api_surface.asyncapi_runtime_events single leaf closeout

> Batch: BE-001PK-01
> Node: `root.contracts.api_surface.asyncapi_runtime_events`
> Parent: `root.contracts.api_surface`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.api_surface.asyncapi_runtime_events` has been evaluated as the AsyncAPI runtime event stream contract owner selected by BE-001PJ-01.

Decision:

`stop_split: true`

The node remains equivalent because `contracts/asyncapi/runtime-events.yaml` was not edited.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Independent physical owner | Stop. The runtime event stream contract is one AsyncAPI root file. |
| Public contract surface | Covered. The leaf owns AsyncAPI version, server/channel, receive operation, owner metadata, stability metadata, and `RuntimeEvent` message payload. |
| Multiple message families | Not present in current file. Current observed contract is one `RuntimeEvent` message payload. |
| Producer/consumer behavior | Outside this leaf. Backend SSE handler and runtime event producers stay behavior owners, not schema owners. |
| Future reopen rule | Allowed only when a concrete AsyncAPI schema change targets event channels, message fields, required fields, enum values, or stability metadata. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Runtime event stream schema maintenance proposal | `contracts.api_surface.asyncapi_runtime_events` | Updated or verified `contracts/asyncapi/runtime-events.yaml` contract |

The leaf may describe and guard:

- AsyncAPI version and metadata;
- browser server/channel address;
- receive operation metadata;
- `RuntimeEvent` message and payload fields;
- required fields and stage enum values;
- schema version, trace, module, strategy, parameter, and payload metadata.

## Non-Claims

This closeout does not claim:

- AsyncAPI content changed;
- OpenAPI HTTP schema changed;
- backend SSE handler behavior changed;
- runtime event producers changed;
- QRPC/Core IR/compiler/runtime/QS behavior changed;
- release transition was opened.

## Next Step

BE-001PL-01 `root.contracts.api_surface` parent_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
