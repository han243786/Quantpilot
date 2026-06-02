import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage
} from "./graphStoreHelpers";
import { createGraphStoreEditorNodeConfigActions } from "./graphStoreEditorNodeConfigActions";
import { createGraphStoreEditorNodeCreationActions } from "./graphStoreEditorNodeCreationActions";
import { createGraphStoreEditorNodePositionActions } from "./graphStoreEditorNodePositionActions";

export function createGraphStoreEditorNodeActions(set, get) {
  return {
    ...createGraphStoreEditorNodeCreationActions(set, get),
    ...createGraphStoreEditorNodePositionActions(set, get),
    ...createGraphStoreEditorNodeConfigActions(set, get),

    toggleNodeCollapse(nodeId) {
      const registry = get().registry;
      const graph = get().graph;
      const nextGraph = attachValidationWithRegistry(
        recordRecentNodeIds(
          {
            ...graph,
            nodes: graph.nodes.map((node) =>
              node.id === nodeId
                ? { ...node, ui_state: { ...node.ui_state, collapsed: !node.ui_state.collapsed } }
                : node
            )
          },
          [nodeId]
        ),
        registry
      );
      saveGraphToStorage(nextGraph);
      set({ graph: nextGraph, compileResult: null, quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
    }
  };
}
