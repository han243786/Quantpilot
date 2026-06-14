# FE-0066 Frontend Graph Editor Property Panel Model Closeout

Status: closed.

## Child Node

`frontend.graph_editor.property_panel_model`

## Boundary

This leaf owns the property panel model hook, store action facade, selector projection, Strategy IR source and diagnostic targeting helpers, and source-apply/reset error handling.

## Owned Files

- `frontend/src/hooks/usePropertyPanelModel.js`
- `frontend/src/hooks/usePropertyPanelActions.js`
- `frontend/src/hooks/propertyPanelSelectors.js`
- `frontend/src/hooks/propertyPanelShared.js`
- `frontend/src/hooks/propertyPanelShared.test.js`
- `frontend/src/components/PropertyPanel.layout.test.jsx`
- `frontend/src/components/PropertyPanel.compileSummary.test.jsx`
- `frontend/src/components/PropertyPanel.strategyIr.test.jsx`
- `frontend/src/components/CompilePanel.integration.test.jsx`

## Public Methods

- `usePropertyPanelModel`
- `usePropertyPanelActions`
- `usePropertyPanelSelectors`
- `formatValue`
- `stringifyJson`
- `compileOutputsText`
- `strategyIrSourceFromGraph`
- `findTargetRangeInSource`
- `diagnosticSeverityCounts`
- `booleanStatusTone`
- `booleanStatusText`
- `strategyIrRoleText`
- `runtimeSourceText`
- `runtimeSourceOfTruthText`

## Preserved Behavior

- Property panel consumers still receive selectors, actions, local apply errors, and `strategyIrEditorRef` through `usePropertyPanelModel`.
- QuantScript, formal QuantScript, and Strategy IR apply/reset handlers still clear or set local errors through the same callback shape.
- Selected node, edge, source node, target node, module definition, diagnostics, runtime metrics, and graph/source projections remain sourced from `useGraphStore`.
- Strategy IR diagnostic targeting still focuses and selects source ranges through `findTargetRangeInSource`.
- Shared formatting and diagnostic helpers now have direct unit coverage.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; the model boundary contains store selectors, action facades, local error state, and source helper logic.
- `leaf_split_positive_trigger`: `public_or_handler_boundary` and `independent_failure_mode`.
- `leaf_split_stop_condition`: reached. The hook, selector, action, and shared helper files are already compact and parent-facing.
- `leaf_split_decision_result`: no deeper split now. Deeper action-handler extraction would add indirection without reducing current risk; the larger remaining risk is in `propertyPanelViews.jsx`, which is queued as the next leaf.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/hooks/propertyPanelShared.test.js src/components/PropertyPanel.layout.test.jsx src/components/PropertyPanel.compileSummary.test.jsx src/components/PropertyPanel.strategyIr.test.jsx src/components/CompilePanel.integration.test.jsx`: passed, 5 files / 9 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.property_panel_views`
