# v4.16.0 root.contracts.api_surface single leaf closeout

> Batch: BE-001PG-01
> Node: `root.contracts.api_surface`
> Parent: `root.contracts`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.api_surface` has been evaluated as the schema-only child selected by BE-001PF-01.

Decision:

`stop_split: false`

The node remains equivalent because no schema files or Rust source files changed. It should continue splitting because the current leaf contains two independent contract owners:

- HTTP API schema owner: `contracts/openapi/root.yaml`;
- runtime event stream schema owner: `contracts/asyncapi/runtime-events.yaml`.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Independent public/contract owners | Split. OpenAPI HTTP paths/tags and AsyncAPI runtime event channels/messages are separate contract surfaces. |
| Different consumers | Split. Backend route/API consumers differ from runtime event stream consumers. |
| Different schema standards | Split. OpenAPI and AsyncAPI have different validation and evolution rules. |
| Shared implementation owner only | Not enough to stop. Both files are schema contracts, but their future changes should not share one leaf gate. |
| Micro-module risk | Acceptable. This is documentation/schema splitting only; no runtime or build overhead is introduced. |

## Equivalence Evidence

No code or schema movement happened in this batch.

Observed physical leaves:

| Future child | Physical file |
| --- | --- |
| `contracts.api_surface.openapi_http` | `contracts/openapi/root.yaml` |
| `contracts.api_surface.asyncapi_runtime_events` | `contracts/asyncapi/runtime-events.yaml` |

## Non-Claims

This closeout does not claim:

- OpenAPI schema contents changed;
- AsyncAPI schema contents changed;
- backend routes or handlers changed;
- runtime event producers changed;
- QRPC/Core IR/compiler/runtime/QS behavior changed;
- executor session ownership moved;
- frontend extraction, E2E cleanup, test retirement, or release transition started.

## Next Step

BE-001PH-01 `root.contracts.api_surface` parent_residual_judgment selects `contracts.api_surface.openapi_http`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
