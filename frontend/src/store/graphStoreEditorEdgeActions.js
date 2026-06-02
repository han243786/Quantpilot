import {
  attachValidationWithRegistry,
  saveGraphToStorage,
  withRecentNodeIds
} from "./graphStoreHelpers";
import { createGraphStoreEditorEdgeCreationActions } from "./graphStoreEditorEdgeCreationActions";

export function createGraphStoreEditorEdgeActions(set, get) {
  return {
    ...createGraphStoreEditorEdgeCreationActions(set, get),

    removeSelected() {
      const registry = get().registry;
      const { selectedNodeId, selectedEdgeId, graph } = get();
      if (!selectedNodeId && !selectedEdgeId) return;
      let nextGraph = graph;
      if (selectedNodeId) {
        nextGraph = {
          ...graph,
          nodes: graph.nodes.filter((node) => node.id !== selectedNodeId),
          edges: graph.edges.filter((edge) => edge.source_node_id !== selectedNodeId && edge.target_node_id !== selectedNodeId)
        };
      }
      if (selectedEdgeId) {
        nextGraph = {
          ...graph,
          edges: graph.edges.filter((edge) => edge.id !== selectedEdgeId)
        };
      }
      const finalGraph = attachValidationWithRegistry(
        withRecentNodeIds(nextGraph, nextGraph.metadata?.editor?.recent_node_ids),
        registry
      );
      saveGraphToStorage(finalGraph);
      set((state) => ({
        graph: finalGraph,
        compileResult: null,
        selectedNodeId: null,
        selectedEdgeId: null,
        quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "",
        runtime: {
          ...state.runtime
        }
      }));
    }
  };
}
