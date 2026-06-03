# v4.16.0 root.contracts.api_surface.openapi_http single leaf closeout

> Batch: BE-001PI-01
> Node: `root.contracts.api_surface.openapi_http`
> Parent: `root.contracts.api_surface`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.api_surface.openapi_http` has been evaluated as the OpenAPI HTTP contract owner selected by BE-001PH-01.

Decision:

`stop_split: true`

The node remains equivalent because `contracts/openapi/root.yaml` was not edited.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Independent physical owner | Stop. The complete HTTP API contract is one canonical OpenAPI root file. |
| Public contract surface | Covered. The leaf owns OpenAPI version, API info, servers, tags, paths, operation ids, chain-stage metadata, request/response schemas, and component schemas. |
| Different consumers inside the file | Not enough to split now. Tag/path consumers differ, but there is no separate physical schema owner in this batch. |
| Verification cost | Stop. Splitting by tag or path would create virtual leaves with duplicated schema gates and higher drift risk. |
| Future reopen rule | Allowed only when a concrete schema change targets a path/tag group and the proposal names that sub-surface explicitly. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| HTTP API schema maintenance proposal | `contracts.api_surface.openapi_http` | Updated or verified `contracts/openapi/root.yaml` contract |

The leaf may describe and guard:

- OpenAPI version and API metadata;
- local server declaration;
- tags and `x-chain-stage` metadata;
- HTTP paths and operation ids;
- request/response shapes;
- shared component schemas.

## Non-Claims

This closeout does not claim:

- OpenAPI content changed;
- AsyncAPI runtime events were handled;
- backend handlers, route registration, AppState, executor behavior, or frontend callers changed;
- QRPC/Core IR/compiler/runtime/QS behavior changed;
- release transition was opened.

## Next Step

BE-001PJ-01 `root.contracts.api_surface` parent_residual_judgment selects `contracts.api_surface.asyncapi_runtime_events`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
