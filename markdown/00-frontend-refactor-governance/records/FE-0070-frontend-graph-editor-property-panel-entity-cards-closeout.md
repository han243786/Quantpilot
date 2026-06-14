# FE-0070 Frontend Graph Editor Property Panel Entity Cards Closeout

Status: closed.

## Child Node

`frontend.graph_editor.property_panel_views.entity_cards`

## Boundary

This leaf extracts node, edge, validation, runtime, metric, and node-source entity cards from `propertyPanelViews.jsx` into `propertyPanelEntityCards.jsx`, keeping `propertyPanelViews.jsx` as the parent compatibility facade and section composer.

## Owned Files

- `frontend/src/components/propertyPanelEntityCards.jsx`
- `frontend/src/components/propertyPanelEntityCards.test.jsx`
- `frontend/src/components/propertyPanelViews.jsx`
- `frontend/src/components/PropertyPanel.layout.test.jsx`
- `frontend/src/components/PropertyPanel.strategyIr.test.jsx`
- `frontend/src/components/CompilePanel.integration.test.jsx`
- `frontend/src/components/StrategyCodePanel.authoringView.test.jsx`

## Public Methods

- `NodeOverviewCard`
- `NodeConfigCard`
- `ConnectionsCard`
- `ValidationCard`
- `ActionableValidationCard`
- `NodeRuntimeCard`
- `NodeMetricsCard`
- `NodeQuantScriptCard`
- `EdgeOverviewCard`

## Preserved Behavior

- Existing consumers can still import entity cards from `propertyPanelViews`.
- Node name and config edits still call parent-owned mutation handlers.
- Node validation issues still route to config, connections, or validation cards through the same configure issue rules.
- Runtime state, runtime metrics, node source, and edge overview cards keep their display and action behavior.
- `NodeParamsSection`, `LaneAwareNodeParamsSection`, `NodeRuntimeSection`, and edge-mode panels now compose entity cards through a dedicated child boundary.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; entity cards have independent node/edge/runtime/config failure modes and are reused by multiple property-panel sections.
- `leaf_split_positive_trigger`: `public_or_handler_boundary`, `independent_failure_mode`, and `reuse_pressure`.
- `leaf_split_stop_condition`: reached for `entity_cards`; the card cluster is cohesive and has direct event-routing tests plus existing panel tests.
- `leaf_split_decision_result`: no deeper split now. Future splits should wait for a dedicated node-config, runtime-card, or edge-editor change.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/propertyPanelEntityCards.test.jsx src/components/PropertyPanel.layout.test.jsx src/components/PropertyPanel.strategyIr.test.jsx src/components/CompilePanel.integration.test.jsx src/components/StrategyCodePanel.authoringView.test.jsx`: passed, 5 files / 12 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.property_panel_views.section_composers`
