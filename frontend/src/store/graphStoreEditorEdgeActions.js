import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage,
  withRecentNodeIds
} from "./graphStoreHelpers";

export function createGraphStoreEditorEdgeActions(set, get) {
  return {
    addEdge(connection) {
      const registry = get().registry;
      const graph = get().graph;
      const edge = {
        id: `edge_${Date.now()}`,
        source_node_id: connection.source,
        source_port: connection.sourceHandle,
        target_node_id: connection.target,
        target_port: connection.targetHandle,
        edge_type: `${connection.source}-${connection.target}`
      };
      const finalGraph = attachValidationWithRegistry(
        recordRecentNodeIds(
          {
            ...graph,
            metadata: { ...graph.metadata, updated_at: Date.now() },
            edges: [...graph.edges, edge]
          },
          [connection.source, connection.target]
        ),
        registry
      );
      saveGraphToStorage(finalGraph);
      set({ graph: finalGraph, compileResult: null, quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
    },

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
