# FE-0081 Frontend Graph Editor Runtime Config Lowering Closeout

Status: closed.

## Child Node

`frontend.graph_editor.graph_compiler_core_ir.runtime_config_lowering`

## Boundary

This leaf owns local runtime config generation and compile-time graph error collection for node sections. `compileGraph.js` remains the public compiler facade, while `compileGraphRuntimeConfig.js` owns capability-derived support gates, source ID mappings, runtime metadata, data sources, intent refs, agent refs, risk refs, execution refs, and portfolio rebalance validation errors.

## Owned Files

- `frontend/src/graph/compileGraph.js`
- `frontend/src/graph/compileGraphRuntimeConfig.js`
- `frontend/src/graph/compileGraphRuntimeConfig.test.js`
- `frontend/src/graph/compileGraphCoreIr.js`
- `frontend/src/graph/compileGraphSupport.js`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/graph/compileGraph.multiSymbol.test.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`

## Public Methods

- `compileGraph`
- `buildRuntimeConfig`

## Preserved Behavior

- `compileGraph` still returns `runtime_config`, `compile_summary`, `core_ir`, QuantScript artifacts, and diagnostics.
- Runtime config output keeps the same metadata, node sections, source mappings, and ref wiring.
- Unsupported symbols, exchanges, execution modules, runtime modes, and invalid rebalance values still produce local compile errors.
- Strategy IR compile integration still receives the same local compile result shape.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; runtime config generation and topology diagnostics have separate failure modes.
- `leaf_split_positive_trigger`: `testability_gain`, `independent_failure_mode`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `runtime_config_lowering`; remaining compiler facade concern is topology diagnostics.
- `leaf_split_decision_result`: continue `frontend.graph_editor.graph_compiler_core_ir` through `topology_diagnostics`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/graph/compileGraphRuntimeConfig.test.js src/graph/compileGraphCoreIr.test.js src/graph/compileGraphSupport.test.js src/graph/compileGraph.diagnostics.test.js src/graph/compileGraph.multiSymbol.test.js src/store/graphStore.strategyIrCompile.test.js`: passed, 6 files / 14 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.graph_compiler_core_ir.topology_diagnostics`
