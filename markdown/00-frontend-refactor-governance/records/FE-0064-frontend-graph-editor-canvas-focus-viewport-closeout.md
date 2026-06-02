# FE-0064 Frontend Graph Editor Canvas Focus Viewport Closeout

Status: closed.

## Child Node

`frontend.graph_editor.canvas_focus_viewport`

## Boundary

This leaf owns pure canvas focus, recommendation, target cycling, focus bounds, and visible-node viewport filtering helpers.

## Owned Files

- `frontend/src/components/strategyCanvasFocus.js`
- `frontend/src/components/strategyCanvasFocus.test.js`
- `frontend/src/components/strategyCanvasViewport.js`
- `frontend/src/components/strategyCanvasViewport.test.js`
- `frontend/src/components/StrategyCanvas.focus.test.jsx`

## Public Methods

- `CANVAS_FOCUS_MODES`
- `resolveCanvasRecommendations`
- `collectIssueNodeIds`
- `collectRecentNodeIds`
- `resolveCanvasFocusTargetIds`
- `resolveCanvasFocusAnchorId`
- `resolveCanvasActiveTargetId`
- `cycleCanvasFocusTarget`
- `buildCanvasFocusBounds`
- `FLOW_NODE_WIDTH`
- `FLOW_NODE_HEIGHT`
- `isNodeVisibleInViewport`
- `collectVisibleNodeIds`

## Preserved Behavior

- Selected, issue, and recent focus modes still produce the same target ids, active target, anchor target, and cycle behavior.
- Diagnostics recommendations still derive recommended nodes, repair-path nodes, repair-path edges, and issue nodes from the selected diagnostics lane context.
- Focus bounds still use the graph editor node dimensions and padding expected by the interaction shell.
- Viewport filtering still keeps intersecting nodes visible and filters nodes outside the viewport margin.
- `StrategyCanvas` still consumes these helpers through parent-level imports rather than submodule horizontal shortcuts.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; focus recommendation and viewport visibility have independent failure modes and direct tests.
- `leaf_split_positive_trigger`: `public_or_handler_boundary` and `independent_failure_mode`.
- `leaf_split_stop_condition`: reached; the focus helper and viewport helper are already small, pure, and independently tested.
- `leaf_split_decision_result`: no deeper split now. Splitting BFS/path recommendation internals or viewport constants would create tiny fragments without reducing current coupling.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/strategyCanvasFocus.test.js src/components/strategyCanvasViewport.test.js src/components/StrategyCanvas.focus.test.jsx`: passed, 3 files / 11 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.node_card_presentation`
