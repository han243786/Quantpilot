import { createNodeFromModule } from "../graph/createNode";
import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage
} from "./graphStoreHelpers";

export function createGraphStoreEditorNodeCreationActions(set, get) {
  return {
    createNode(moduleKey) {
      const registry = get().registry;
      const moduleDef = registry.getByKey(moduleKey);
      if (!moduleDef) return;
      if (moduleDef.availability?.status === "unsupported") return;
      const graph = get().graph;
      const node = createNodeFromModule(moduleDef);
      const finalGraph = attachValidationWithRegistry(
        recordRecentNodeIds(
          {
            ...graph,
            metadata: { ...graph.metadata, updated_at: Date.now() },
            nodes: [...graph.nodes, node]
          },
          [node.id]
        ),
        registry
      );
      saveGraphToStorage(finalGraph);
      set({
        graph: finalGraph,
        selectedNodeId: node.id,
        selectedEdgeId: null,
        quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || ""
      });
    }
  };
}
