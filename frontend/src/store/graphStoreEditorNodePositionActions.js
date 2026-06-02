import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage
} from "./graphStoreHelpers";

export function createGraphStoreEditorNodePositionActions(set, get) {
  return {
    updateNodePosition(nodeId, position, persist = true) {
      const registry = get().registry;
      const graph = get().graph;
      const nextGraph = attachValidationWithRegistry(
        persist
          ? recordRecentNodeIds(
              {
                ...graph,
                metadata: { ...graph.metadata, updated_at: Date.now() },
                nodes: graph.nodes.map((node) => (node.id === nodeId ? { ...node, position } : node))
              },
              [nodeId]
            )
          : {
              ...graph,
              metadata: graph.metadata,
              nodes: graph.nodes.map((node) => (node.id === nodeId ? { ...node, position } : node))
            },
        registry
      );
      if (persist) saveGraphToStorage(nextGraph);
      set({
        graph: nextGraph,
        compileResult: null,
        quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || ""
      });
    },

    updateEditorViewport(viewport, persist = false) {
      const graph = get().graph;
      const nextGraph = {
        ...graph,
        metadata: {
          ...graph.metadata,
          updated_at: persist ? Date.now() : graph.metadata.updated_at,
          editor: {
            ...(graph.metadata?.editor || {}),
            viewport
          }
        }
      };
      if (persist) saveGraphToStorage(nextGraph);
      set({ graph: nextGraph });
    }
  };
}
