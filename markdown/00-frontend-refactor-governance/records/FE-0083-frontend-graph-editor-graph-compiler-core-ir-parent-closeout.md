# FE-0083 Frontend Graph Editor Graph Compiler Core IR Parent Closeout

Status: closed.

## Parent Node

`frontend.graph_editor.graph_compiler_core_ir`

## Boundary

This parent owns graph compilation from editor graph state into compile output, Core IR, runtime config, topology diagnostics, and local compile diagnostics. `compileGraph.js` remains the public facade; implementation ownership now lives in split white-box children.

## Closed Children

- `frontend.graph_editor.graph_compiler_core_ir.compile_support`
- `frontend.graph_editor.graph_compiler_core_ir.core_ir_lowering`
- `frontend.graph_editor.graph_compiler_core_ir.runtime_config_lowering`
- `frontend.graph_editor.graph_compiler_core_ir.topology_diagnostics`

## Public Methods

- `compileGraph`
- `buildLocalCompileDiagnostics`
- `buildCoreIr`
- `buildRuntimeConfig`
- `buildTopology`
- `appendGraphCompileDiagnostics`

## Preserved Behavior

- Compile graph consumers keep calling `compileGraph`.
- Local compile diagnostics, Core IR lowering, runtime config projection, deterministic topology ordering, cycle blockers, and validation blockers remain covered by child closeout tests.
- The parent has no direct child-to-child calls beyond `compileGraph.js` facade orchestration, preserving the development-time parent communication rule.

## Recursive Decision

- `parent_closeout_gate`: passed; every planned child under `graph_compiler_core_ir` has a closeout record and is represented in the frontend-local module tree.
- `leaf_split_decision_result`: no deeper split for this parent now. Continue the active graph editor parent through `frontend.graph_editor.quantscript_bridge`.

## Verification

- No code changed in this closeout.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.

## Next Leaf

`frontend.graph_editor.quantscript_bridge`
