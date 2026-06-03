# v4.16.0 root.contracts.api_surface parent residual judgment selects openapi_http

> Batch: BE-001PH-01
> Node: `root.contracts.api_surface`
> Selected child: `root.contracts.api_surface.openapi_http`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.api_surface` continues after BE-001PG-01 set `stop_split: false`.

Decision:

`next_child: root.contracts.api_surface.openapi_http`

## Closed Api Surface Children

No `root.contracts.api_surface` children are closed yet.

## Open Api Surface Residuals

| Residual | Status |
| --- | --- |
| `contracts.api_surface.openapi_http` | Selected next. Owns the OpenAPI HTTP contract file. |
| `contracts.api_surface.asyncapi_runtime_events` | Queued. Owns the AsyncAPI runtime event stream contract file. |

## Selection Rationale

`contracts.api_surface.openapi_http` is selected first because it is the broadest externally visible schema surface:

- physical file: `contracts/openapi/root.yaml`;
- declares OpenAPI version, API info, servers, tags, HTTP paths, operation ids, chain stage metadata, request/response schemas, and component schemas;
- does not own runtime event stream message shape, which remains queued under `contracts.api_surface.asyncapi_runtime_events`;
- does not own backend route handler behavior, which remains in already-closed backend modules unless a future explicit schema-to-handler change reopens the relevant owner.

## Hard Boundaries

The next `root.contracts.api_surface.openapi_http` closeout must not:

- edit `contracts/openapi/root.yaml`;
- change paths, operation ids, tags, versions, examples, request schemas, response schemas, or component schemas;
- change backend handlers, route registration, AppState, executor behavior, event producers, or frontend callers;
- change QRPC/Core IR/compiler/runtime/QS behavior;
- introduce release transition sibling links.

## Next Step

BE-001PI-01 `root.contracts.api_surface.openapi_http` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
