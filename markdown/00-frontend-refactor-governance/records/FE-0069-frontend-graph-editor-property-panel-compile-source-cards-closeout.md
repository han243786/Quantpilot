# FE-0069 Frontend Graph Editor Property Panel Compile Source Cards Closeout

Status: closed.

## Child Node

`frontend.graph_editor.property_panel_views.compile_source_cards`

## Boundary

This leaf extracts compile/source cards from `propertyPanelViews.jsx` into `propertyPanelCompileSourceCards.jsx`, keeping `propertyPanelViews.jsx` as the parent compatibility facade and section composer.

## Owned Files

- `frontend/src/components/propertyPanelCompileSourceCards.jsx`
- `frontend/src/components/propertyPanelCompileSourceCards.test.jsx`
- `frontend/src/components/propertyPanelViews.jsx`
- `frontend/src/components/PropertyPanel.compileSummary.test.jsx`
- `frontend/src/components/StrategyCodePanel.authoringView.test.jsx`
- `frontend/src/components/PropertyPanel.layout.test.jsx`
- `frontend/src/components/CompilePanel.integration.test.jsx`

## Public Methods

- `RepairPathContextPanel`
- `GraphOverviewCard`
- `CompileSummaryCard`
- `QuantScriptEditorCard`
- `FormalQuantScriptEditorCard`
- `StrategyIrEditorCard`

## Preserved Behavior

- Existing consumers can still import compile/source cards from `propertyPanelViews`.
- Graph overview still reports graph identity, node/edge counts, validation counts, and compile status.
- Compile summary still separates final runtime compile output from Strategy IR semantic preflight and shows conflict guidance when runtime and semantic checks disagree.
- Formal QuantScript, graph-source, and Strategy IR editor cards keep their draft edit, reset, apply, focus, and error display behaviors.
- `SourceSection` still composes authoring cards plus compile/source editor cards through the parent boundary.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; compile/source cards have distinct compile-contract dependencies, source editor focus behavior, and user-facing runtime source-of-truth semantics.
- `leaf_split_positive_trigger`: `public_or_handler_boundary`, `independent_failure_mode`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `compile_source_cards`; the child owns a cohesive card cluster with direct card tests and existing integration anchors.
- `leaf_split_decision_result`: no deeper split now. Future splits should wait for a dedicated compile-summary, formal-source editor, or Strategy IR editor change.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/propertyPanelCompileSourceCards.test.jsx src/components/PropertyPanel.compileSummary.test.jsx src/components/StrategyCodePanel.authoringView.test.jsx src/components/PropertyPanel.layout.test.jsx src/components/CompilePanel.integration.test.jsx`: passed, 5 files / 11 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.property_panel_views.entity_cards`
