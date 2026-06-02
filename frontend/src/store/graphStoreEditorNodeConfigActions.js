import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage
} from "./graphStoreHelpers";

export function createGraphStoreEditorNodeConfigActions(set, get) {
  return {
    updateNodeConfig(nodeId, key, value) {
      const registry = get().registry;
      const graph = get().graph;
      const finalGraph = attachValidationWithRegistry(
        recordRecentNodeIds(
          {
            ...graph,
            metadata: { ...graph.metadata, updated_at: Date.now() },
            nodes: graph.nodes.map((node) =>
              node.id === nodeId ? { ...node, config: { ...node.config, [key]: value } } : node
            )
          },
          [nodeId]
        ),
        registry
      );
      saveGraphToStorage(finalGraph);
      set({
        graph: finalGraph,
        compileResult: null,
        quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || ""
      });
    },

    updateNodeName(nodeId, value) {
      const registry = get().registry;
      const graph = get().graph;
      const nextGraph = attachValidationWithRegistry(
        recordRecentNodeIds(
          {
            ...graph,
            metadata: { ...graph.metadata, updated_at: Date.now() },
            nodes: graph.nodes.map((node) => (node.id === nodeId ? { ...node, name: value } : node))
          },
          [nodeId]
        ),
        registry
      );
      saveGraphToStorage(nextGraph);
      set({
        graph: nextGraph,
        compileResult: null,
        quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || ""
      });
    }
  };
}
