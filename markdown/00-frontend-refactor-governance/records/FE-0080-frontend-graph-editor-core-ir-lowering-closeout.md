# FE-0080 Frontend Graph Editor Core IR Lowering Closeout

Status: closed.

## Child Node

`frontend.graph_editor.graph_compiler_core_ir.core_ir_lowering`

## Boundary

This leaf owns conversion from validated frontend graph structures into QuantPilot Core IR sections. `compileGraph.js` remains the public compiler facade, while `compileGraphCoreIr.js` owns data bindings, indicators, signal rules, agent policies, risk policies, execution config projection, intent condition lowering, spread specs, and portfolio rebalance Core IR policy output.

## Owned Files

- `frontend/src/graph/compileGraph.js`
- `frontend/src/graph/compileGraphCoreIr.js`
- `frontend/src/graph/compileGraphCoreIr.test.js`
- `frontend/src/graph/compileGraphSupport.js`
- `frontend/src/graph/compileGraphSupport.test.js`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/graph/compileGraph.multiSymbol.test.js`
- `frontend/src/graph/spread.test.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`

## Public Methods

- `compileGraph`
- `buildCoreIr`

## Preserved Behavior

- Store and UI callers still call `compileGraph` from `compileGraph.js`.
- Core IR metadata, data bindings, indicators, signal rules, agent policies, risk policies, and execution projection remain attached to compile results.
- Portfolio rebalance agent config still lowers to `portfolio_rebalance` policy output.
- Spread observer lowering and formal Strategy IR fallback coverage remain intact.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; Core IR projection is independently testable from runtime config compile validation.
- `leaf_split_positive_trigger`: `testability_gain`, `independent_failure_mode`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `core_ir_lowering`; remaining compiler facade concerns belong to runtime config lowering and topology diagnostics.
- `leaf_split_decision_result`: continue `frontend.graph_editor.graph_compiler_core_ir` through `runtime_config_lowering`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/graph/compileGraphCoreIr.test.js src/graph/compileGraphSupport.test.js src/graph/compileGraph.diagnostics.test.js src/graph/compileGraph.multiSymbol.test.js src/graph/spread.test.js src/store/graphStore.strategyIrCompile.test.js`: passed, 6 files / 14 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.graph_compiler_core_ir.runtime_config_lowering`
