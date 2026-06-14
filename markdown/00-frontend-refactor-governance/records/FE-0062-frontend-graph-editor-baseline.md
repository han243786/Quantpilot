# FE-0062 Frontend Graph Editor Baseline

Status: baseline established.

## Parent Node

`frontend.graph_editor`

## Current Scope

The graph editor parent owns graph editing UI, graph node presentation, canvas focus and viewport helpers, property-panel inspection/editing surfaces, graph creation/validation/compile semantics, QuantScript graph source bridging, module-palette node insertion, and the editor action facade that mutates graph structure.

Runtime panels, backtest result views, persistent graph storage, backend API contracts, route contracts, and capability catalog truth remain external parent inputs. Store actions that are runtime, compile, persistence, capability refresh, or global app startup concerns must stay in their owning parents unless a later store leaf explicitly moves them.

`EditorPage.jsx` is treated as a legacy/compat graph-editor shell candidate because it still composes `ModuleSidebar`, `StrategyCanvas`, and `PropertyPanel`, even though current routed workspace entry commonly reaches the editor through strategy workspace surfaces.

## Initial Child Queue

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

## Current Owned And Split-Target Files

- `frontend/src/pages/EditorPage.jsx`
- `frontend/src/components/StrategyCanvas.jsx`
- `frontend/src/components/StrategyCanvas.focus.test.jsx`
- `frontend/src/components/StrategyCanvas.interaction.test.jsx`
- `frontend/src/components/StrategyCanvasMiniMap.jsx`
- `frontend/src/components/strategyCanvasFocus.js`
- `frontend/src/components/strategyCanvasFocus.test.js`
- `frontend/src/components/strategyCanvasViewport.js`
- `frontend/src/components/strategyCanvasViewport.test.js`
- `frontend/src/components/PropertyPanel.jsx`
- `frontend/src/components/PropertyPanel.layout.test.jsx`
- `frontend/src/components/PropertyPanel.compileSummary.test.jsx`
- `frontend/src/components/PropertyPanel.strategyIr.test.jsx`
- `frontend/src/components/propertyPanelViews.jsx`
- `frontend/src/components/CompilePanel.integration.test.jsx`
- `frontend/src/components/ModuleSidebar.jsx`
- `frontend/src/components/ModuleSidebar.test.jsx`
- `frontend/src/hooks/usePropertyPanelModel.js`
- `frontend/src/hooks/usePropertyPanelActions.js`
- `frontend/src/hooks/propertyPanelSelectors.js`
- `frontend/src/hooks/propertyPanelShared.js`
- `frontend/src/nodes/BaseNodeCard.jsx`
- `frontend/src/nodes/BaseNodeCard.test.jsx`
- `frontend/src/nodes/nodeCardPresentation.js`
- `frontend/src/nodes/nodeCardPresentation.test.js`
- `frontend/src/graph/createGraph.js`
- `frontend/src/graph/createNode.js`
- `frontend/src/graph/validation.js`
- `frontend/src/graph/compileGraph.js`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/graph/compileGraph.multiSymbol.test.js`
- `frontend/src/graph/quantscript.js`
- `frontend/src/graph/quantscript.test.js`
- `frontend/src/graph/spread.test.js`
- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStore.editorActions.test.js`

## Important Consumers

- `frontend/src/pages/StrategyWorkspaceCodeTab.jsx`
- `frontend/src/pages/StrategyWorkspacePage.codeMode.test.jsx`
- `frontend/src/components/StrategyCodePanel.jsx`
- `frontend/src/components/StrategyDiagnosticsPanel.jsx`
- `frontend/src/components/StrategyParamsPanel.jsx`
- `frontend/src/components/EventStreamPanel.nodeFocus.test.jsx`
- `frontend/src/store/graphStore.js`
- `frontend/src/store/graphStoreCompileActions.js`
- `frontend/src/store/graphStoreCompileOutcomeMapping.js`
- `frontend/src/store/graphStorePersistenceActions.js`
- `frontend/src/store/graphStorePersistenceHelpers.js`
- `frontend/src/templates/strategyTemplates.js`
- `frontend/src/test/fixtures/runtime/buildValidatedSampleGraph.js`
- `frontend/src/capabilities/capabilityGovernanceCore.js`

## Whitebox Contract

### Public Inputs

- Graph store graph state, registry, runtime highlights, selected node/edge/diagnostic targets, and editor viewport.
- Store editor actions for node selection, edge selection, node position, edge creation, node config/name updates, selected removal, and source draft application.
- Capability-aware registry and module definitions used by graph factories, validation, module palette, and compiler projections.
- Current graph artifacts and source drafts for QuantScript, formal QuantScript, and Strategy IR.
- React Flow viewport, connection, and rendering contracts.

### Public Outputs

- Strategy canvas rendering, node/edge interaction dispatch, focus controls, lane/focus status, and mini-map overlay.
- Base node card presentation data, handles, runtime status display, quick-field presentation, and metric labels.
- Property panel shell for graph overview, selected node, selected edge, compile diagnostics, source editing, runtime metrics, and Strategy IR focus handoff.
- Graph factory helpers: `createSampleGraph`, `createEmptyGraph`, and `createNodeFromModule`.
- Graph semantic helpers: `isValidConnection`, `validateGraph`, `compileGraph`, QuantScript generation/parsing, and graph artifacts attachment.
- Module palette node insertion UI and editor action facade for graph structure mutations.

## Equivalence Anchors

- `frontend/src/components/StrategyCanvas.focus.test.jsx`
- `frontend/src/components/StrategyCanvas.interaction.test.jsx`
- `frontend/src/components/strategyCanvasFocus.test.js`
- `frontend/src/components/strategyCanvasViewport.test.js`
- `frontend/src/nodes/BaseNodeCard.test.jsx`
- `frontend/src/nodes/nodeCardPresentation.test.js`
- `frontend/src/components/PropertyPanel.layout.test.jsx`
- `frontend/src/components/PropertyPanel.compileSummary.test.jsx`
- `frontend/src/components/PropertyPanel.strategyIr.test.jsx`
- `frontend/src/components/CompilePanel.integration.test.jsx`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/graph/compileGraph.multiSymbol.test.js`
- `frontend/src/graph/spread.test.js`
- `frontend/src/graph/quantscript.test.js`
- `frontend/src/store/graphStore.editorActions.test.js`
- `frontend/src/components/ModuleSidebar.test.jsx`
- Frontend build.

## Baseline Verification

- From `frontend/`, graph editor anchor test set: passed, 16 files / 44 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Split Rules

- Keep parent-child communication through graph editor public imports, store public actions, and existing route/store boundaries; do not add submodule-to-submodule horizontal shortcuts.
- Do not move runtime panel rendering, runtime session transport, backtest result inspection, or persistence API behavior into this parent.
- Do not change route contracts or workspace tab availability while closing graph editor leaves.
- Keep capability catalog truth owned by `frontend.capabilities`; graph editor leaves may consume capability-aware registry output.
- Keep store-wide shape migration for the later `frontend.store` parent; graph editor may isolate editor action facade behavior but must not rewrite runtime/compile/persistence store branches.
- Treat `propertyPanelViews.jsx`, `compileGraph.js`, `quantscript.js`, `validation.js`, and `StrategyCanvas.jsx` as high-value split candidates because each exceeds a compact leaf size or exposes multiple public concerns.

## First Leaf

`frontend.graph_editor.canvas_interaction_shell`
