import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage
} from "./graphStoreHelpers";
import { createGraphStoreEditorNodeCreationActions } from "./graphStoreEditorNodeCreationActions";

export function createGraphStoreEditorNodeActions(set, get) {
  return {
    ...createGraphStoreEditorNodeCreationActions(set, get),

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
      set({ graph: nextGraph, compileResult: null, quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
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
    },

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
      set({ graph: finalGraph, compileResult: null, quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
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
      set({ graph: nextGraph, compileResult: null, quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
    },

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
