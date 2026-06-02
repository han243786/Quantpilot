# FE-0063 Frontend Graph Editor Canvas Interaction Shell Closeout

Status: closed.

## Child Node

`frontend.graph_editor.canvas_interaction_shell`

## Boundary

This leaf extracts canvas interaction shell helpers from `StrategyCanvas.jsx` into `strategyCanvasInteractionShell.js`.

## Owned Files

- `frontend/src/components/StrategyCanvas.jsx`
- `frontend/src/components/strategyCanvasInteractionShell.js`
- `frontend/src/components/strategyCanvasInteractionShell.test.js`
- `frontend/src/components/StrategyCanvas.focus.test.jsx`
- `frontend/src/components/StrategyCanvas.interaction.test.jsx`

## Public Methods

- `resolveNodeCardMode`
- `scheduleAfterFirstPaint`
- `focusCanvasTargets`

## Preserved Behavior

- `StrategyCanvas.jsx` still owns the public React component and React Flow composition.
- Node-card staged/full query behavior remains unchanged.
- Canvas decoration, rich-node-card, deferred-flow, minimap, and focus viewport scheduling still happen after first paint.
- Focus actions still choose an anchor `setCenter`, single-target `setCenter`, or multi-target `fitBounds` using the same bounds helper.
- Node click, pane click, drag-stop, connect, and selected-node removal behavior remain covered by existing component anchors.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; `StrategyCanvas.jsx` is large and mixes React rendering, first-paint scheduling, focus viewport execution, interaction dispatch, lane summaries, and recommendation UI.
- `leaf_split_positive_trigger`: `public_or_handler_boundary`, `independent_failure_mode`, and `reuse_pressure`.
- `leaf_split_stop_condition`: not reached for the parent; this leaf closes only the interaction helper boundary.
- `leaf_split_decision_result`: no deeper split inside `frontend.graph_editor.canvas_interaction_shell` now. The extracted helpers are compact, public, tested, and delegate focus math to the already separate focus/viewport leaves.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/strategyCanvasInteractionShell.test.js src/components/StrategyCanvas.focus.test.jsx src/components/StrategyCanvas.interaction.test.jsx`: passed, 3 files / 13 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.canvas_focus_viewport`
