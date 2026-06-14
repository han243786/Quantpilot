# FE-0096 Frontend Graph Editor Parent Closeout

Status: closed.

## Parent Node

`frontend.graph_editor`

## Closed Children

- `frontend.graph_editor.canvas_interaction_shell`
- `frontend.graph_editor.canvas_focus_viewport`
- `frontend.graph_editor.node_card_presentation`
- `frontend.graph_editor.property_panel_model`
- `frontend.graph_editor.property_panel_views`
- `frontend.graph_editor.module_palette`
- `frontend.graph_editor.graph_factory_validation`
- `frontend.graph_editor.graph_compiler_core_ir`
- `frontend.graph_editor.quantscript_bridge`
- `frontend.graph_editor.editor_store_actions`
- `frontend.graph_editor.legacy_editor_page_shell`

## Final Parent Boundary

`frontend.graph_editor` now owns graph rendering, graph editor interaction shells, node-card presentation, property-panel model/view boundaries, module palette projection, graph factory and validation helpers, graph compiler lowering, QuantScript bridge helpers, editor store action facade leaves, and the legacy editor page shell equivalence baseline.

Runtime session panels, backtest detail/compare views, global store migration, API transport, routing contracts, and shared style system ownership remain outside this parent.

## Whitebox Contract

### Public Inputs

- Graph store graph state, registry, selected node/edge/diagnostic targets, source drafts, editor viewport, and runtime highlights.
- Capability-aware module registry output used by graph factories, validation, module palette, and compiler projections.
- React Flow node, edge, connection, viewport, and rendering contracts.
- Router helpers consumed only by the legacy editor shell backtest-detail bridge.

### Public Outputs

- Strategy canvas rendering, interaction dispatch, focus controls, lane/focus status, and mini-map overlay.
- Base node-card presentation data, handles, runtime status display, quick-field presentation, and metric labels.
- Property panel shell, model, cards, layout primitives, and section composer boundaries.
- Graph factory helpers, validation rules/support, compiler projections, topology diagnostics, and graph source artifacts.
- Module palette model and node insertion surface.
- Editor store public action facade for selection, drafts, templates, node mutation, edge creation/removal, and selected deletion.
- Legacy editor page shell baseline for top toolbar, module sidebar, canvas, property panel, deferred event stream mount, and backtest navigation bridge.

## Preserved Behavior

- Existing graph editor public imports and store action names remain stable for workspace, canvas, panel, and toolbar consumers.
- Parent-child communication still flows through public component props, graph helpers, store action facades, and route/store boundaries.
- No child-to-child shortcut was introduced during this parent.
- Runtime panels and backtest result views were not absorbed into graph editor ownership.

## Further-Split Decision

No further split is useful inside `frontend.graph_editor` now. Large helper clusters were split into smaller white-box children where semantic boundaries justified it, and compact compatibility shells were closed with equivalent baseline coverage instead of being over-filed.

## Verification

- From `frontend/`, graph editor parent anchor test set: passed, 45 files / 112 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Parent Candidate

`frontend.runtime_panels`
