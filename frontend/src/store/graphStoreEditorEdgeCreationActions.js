import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage
} from "./graphStoreHelpers";

export function createGraphStoreEditorEdgeCreationActions(set, get) {
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
      set({
        graph: finalGraph,
        compileResult: null,
        quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || ""
      });
    }
  };
}
