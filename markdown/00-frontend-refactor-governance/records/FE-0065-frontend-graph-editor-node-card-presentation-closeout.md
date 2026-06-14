# FE-0065 Frontend Graph Editor Node Card Presentation Closeout

Status: closed.

## Child Node

`frontend.graph_editor.node_card_presentation`

## Boundary

This leaf owns graph node card rendering, node card data projection, metric label formatting, quick-field presentation, recommendation role projection, and the data-node live price overlay.

## Owned Files

- `frontend/src/nodes/BaseNodeCard.jsx`
- `frontend/src/nodes/BaseNodeCard.test.jsx`
- `frontend/src/nodes/NodePriceOverlay.jsx`
- `frontend/src/nodes/NodePriceOverlay.test.jsx`
- `frontend/src/nodes/nodeCardPresentation.js`
- `frontend/src/nodes/nodeCardPresentation.test.js`
- `frontend/src/components/StrategyCanvas.interaction.test.jsx`

## Public Methods

- `BaseNodeCard`
- `NodePriceOverlay`
- `buildNodeCardData`
- `formatNodeMetricLabel`

## Preserved Behavior

- `BaseNodeCard` still renders handles, node header, summary chips, quick fields, runtime line, issue text, focus classes, and recommendation classes.
- Quick-field controls still stop canvas bubbling and dispatch `updateNodeConfig`.
- Data nodes still show a live ticker price overlay.
- The price overlay still reads the current price on mount and subscribes directly to graph node changes without requiring a parent card render.
- `StrategyCanvas` still receives node card data from `buildNodeCardData` and maps all node types to `BaseNodeCard`.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; node card rendering mixes UI rendering, presentation projection, and a store-subscription price overlay.
- `leaf_split_positive_trigger`: `public_or_handler_boundary`, `independent_failure_mode`, and `reuse_pressure`.
- `leaf_split_stop_condition`: reached for this leaf after `NodePriceOverlay` extraction. The remaining `BaseNodeCard` rendering and `nodeCardPresentation` projection are each covered by direct anchors and are not yet large enough to justify a deeper split.
- `leaf_split_decision_result`: no deeper split now. Future splits should wait for a concrete card-layout change, new node-card variant, or a separate live-market overlay feature request.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/nodes/NodePriceOverlay.test.jsx src/nodes/BaseNodeCard.test.jsx src/nodes/nodeCardPresentation.test.js src/components/StrategyCanvas.interaction.test.jsx`: passed, 4 files / 12 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.property_panel_model`
