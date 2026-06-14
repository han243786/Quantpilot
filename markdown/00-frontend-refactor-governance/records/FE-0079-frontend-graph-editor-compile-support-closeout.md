# FE-0079 Frontend Graph Editor Compile Support Closeout

Status: closed.

## Child Node

`frontend.graph_editor.graph_compiler_core_ir.compile_support`

## Boundary

This leaf owns pure support helpers for local graph compilation: capability support lookups, JSON and CSV normalization, portfolio rebalance option normalization, and local compile diagnostics. `compileGraph.js` remains the public compiler facade.

## Owned Files

- `frontend/src/graph/compileGraph.js`
- `frontend/src/graph/compileGraphSupport.js`
- `frontend/src/graph/compileGraphSupport.test.js`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/graph/compileGraph.multiSymbol.test.js`
- `frontend/src/graph/spread.test.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.test.js`

## Public Methods

- `compileGraph`
- `capabilitySet`
- `supportMap`
- `capabilityEntryStatus`
- `capabilityReason`
- `jsonValue`
- `parseCsvStrings`
- `parseCsvNumbers`
- `normalizeRebalanceSchedule`
- `normalizeRebalanceAllocationKind`
- `normalizeRebalanceRankMethod`
- `normalizeRebalanceScoreNormalize`
- `agentUsesPortfolioRebalance`
- `buildLocalCompileDiagnostics`

## Preserved Behavior

- `compileGraph` stays the only public compile entry imported by store and graph tests.
- Capability fallback behavior still supports runtime modes, execution modules, exchanges, symbols, and frontend module support.
- Portfolio rebalance CSV parsing and invalid option detection still drive local compile errors.
- Local compile diagnostics still emit deterministic graph-sourced error and warning records.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; support helpers, Core IR lowering, runtime config lowering, and topology diagnostics have different failure modes.
- `leaf_split_positive_trigger`: `testability_gain`, `independent_failure_mode`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `compile_support`; helpers are pure and covered directly.
- `leaf_split_decision_result`: continue splitting `frontend.graph_editor.graph_compiler_core_ir` through `core_ir_lowering`, `runtime_config_lowering`, and `topology_diagnostics`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/graph/compileGraphSupport.test.js src/graph/compileGraph.diagnostics.test.js src/graph/compileGraph.multiSymbol.test.js src/graph/spread.test.js src/store/graphStore.strategyIrCompile.test.js src/store/graphStoreCompileOutcomeProjection.test.js`: passed, 6 files / 15 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.graph_compiler_core_ir.core_ir_lowering`
