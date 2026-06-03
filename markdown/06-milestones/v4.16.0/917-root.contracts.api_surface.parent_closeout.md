# v4.16.0 root.contracts.api_surface parent closeout

> Batch: BE-001PL-01
> Node: `root.contracts.api_surface`
> Parent: `root.contracts`
> Stage: `parent_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.api_surface` is closed for the current recursive contracts extraction scope.

Decision:

`close_parent: true`

The parent remains equivalent because neither schema file changed:

- `contracts/openapi/root.yaml`;
- `contracts/asyncapi/runtime-events.yaml`.

## Closed Children

| Child | Result |
| --- | --- |
| `contracts.api_surface.openapi_http` | Closed with `stop_split: true`; owns the canonical OpenAPI HTTP schema root. |
| `contracts.api_surface.asyncapi_runtime_events` | Closed with `stop_split: true`; owns the canonical AsyncAPI runtime event stream schema root. |

## Parent Boundary

`root.contracts.api_surface` now owns the schema-surface parent boundary:

- OpenAPI and AsyncAPI are separate schema leaves;
- schema content remained frozen during this closeout;
- backend route handlers, SSE handlers, runtime event producers, AppState, executor behavior, frontend callers, and release transition logic remain outside this parent;
- any future schema change must name either `contracts.api_surface.openapi_http` or `contracts.api_surface.asyncapi_runtime_events` explicitly before editing schema content.

## Non-Claims

This closeout does not claim:

- HTTP or event schema content changed;
- backend behavior changed;
- QRPC/Core IR/compiler/runtime/QS behavior changed;
- executor extraction is complete;
- release transition optimization is allowed.

## Root.Contracts Return

Return to `root.contracts` residual judgment.

Recommended next child:

`root.contracts.qrpc_core`

Rationale: after schema surfaces are closed, the next highest-risk contracts owner is `qrpc_core`, which contains runtime protocol structs, artifact/version constants, digest helpers, Strategy IR, plugin metadata re-exports, event envelope proto, and core errors.

## Next Step

BE-001PM-01 `root.contracts` parent_residual_judgment selects `contracts.qrpc_core`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
