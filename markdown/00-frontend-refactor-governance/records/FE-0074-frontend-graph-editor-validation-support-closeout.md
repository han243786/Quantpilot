# FE-0074 Frontend Graph Editor Validation Support Closeout

Status: closed.

## Child Node

`frontend.graph_editor.graph_factory_validation.validation_support`

## Boundary

This leaf extracts graph validation infrastructure from `validation.js` into `validationSupport.js`. The public validation API remains at `validation.js`, while capability indexing, fallback support maps, comparison helpers, and issue construction are now white-box utilities.

## Owned Files

- `frontend/src/graph/validation.js`
- `frontend/src/graph/validationSupport.js`
- `frontend/src/graph/validationSupport.test.js`
- `frontend/src/graph/spread.test.js`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/graph/compileGraph.multiSymbol.test.js`

## Public Methods

- `allowedChain`
- `typeLabels`
- `capabilitySet`
- `supportMap`
- `capabilityEntryStatus`
- `capabilityReason`
- `compareValues`
- `buildIssue`
- `buildCapabilityIndex`
- `isValidConnection`
- `validateGraph`

## Preserved Behavior

- `isValidConnection` keeps the existing graph connection API and edge rule behavior.
- `validateGraph` keeps node, edge, graph issue outputs and issue count semantics.
- Capability support entries still override fallback sets before unsupported runtime, execution, exchange, symbol, or frontend module issues are emitted.
- Field comparison and issue id construction remain deterministic.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; validation infrastructure has independent capability-index and issue-construction failure modes.
- `leaf_split_positive_trigger`: `public_or_handler_boundary`, `independent_failure_mode`, and `testability_gain`.
- `leaf_split_stop_condition`: reached for `validation_support`; it is a pure helper leaf with direct tests and existing graph validation integration tests.
- `leaf_split_decision_result`: continue splitting the parent `frontend.graph_editor.graph_factory_validation` through node factory, seed graph factory, and validation rules.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/graph/validationSupport.test.js src/graph/spread.test.js src/graph/compileGraph.diagnostics.test.js src/graph/compileGraph.multiSymbol.test.js`: passed, 4 files / 8 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.graph_factory_validation.node_factory`
