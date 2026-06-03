# v4.16.0 root.contracts.api_surface parent residual judgment selects asyncapi_runtime_events

> Batch: BE-001PJ-01
> Node: `root.contracts.api_surface`
> Selected child: `root.contracts.api_surface.asyncapi_runtime_events`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.api_surface` returns to its remaining schema residual after `contracts.api_surface.openapi_http` closeout.

Decision:

`next_child: root.contracts.api_surface.asyncapi_runtime_events`

## Closed Api Surface Children

| Child | Result |
| --- | --- |
| `contracts.api_surface.openapi_http` | Closed with `stop_split: true`; owns `contracts/openapi/root.yaml`. |

## Open Api Surface Residuals

| Residual | Status |
| --- | --- |
| `contracts.api_surface.asyncapi_runtime_events` | Selected next. Owns the AsyncAPI runtime event stream contract file. |

## Selection Rationale

`contracts.api_surface.asyncapi_runtime_events` is selected because it is the only remaining `api_surface` child:

- physical file: `contracts/asyncapi/runtime-events.yaml`;
- declares AsyncAPI version, runtime event stream server/channel, receive operation, owner metadata, stability metadata, and the `RuntimeEvent` message payload contract;
- does not own HTTP API schema, which is already closed under `contracts.api_surface.openapi_http`;
- does not own backend SSE handler behavior or runtime event producer behavior.

## Hard Boundaries

The next `root.contracts.api_surface.asyncapi_runtime_events` closeout must not:

- edit `contracts/asyncapi/runtime-events.yaml`;
- change channel address, server info, action, owner metadata, stability metadata, message fields, required fields, enum values, or payload examples;
- change backend event stream handler behavior or runtime event producers;
- change OpenAPI, QRPC/Core IR/compiler/runtime/QS behavior;
- introduce release transition sibling links.

## Next Step

BE-001PK-01 `root.contracts.api_surface.asyncapi_runtime_events` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
