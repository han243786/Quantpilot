import { createNodeFromModule } from "../graph/createNode";
import {
  attachValidationWithRegistry,
  recordRecentNodeIds,
  saveGraphToStorage,
  withRecentNodeIds
} from "./graphStoreHelpers";
import { createGraphStoreCompileActions } from "./graphStoreCompileActions";
import { createGraphStoreEditorDraftActions } from "./graphStoreEditorDraftActions";
import { createGraphStoreEditorSelectionActions } from "./graphStoreEditorSelectionActions";
import { createGraphStoreEditorTemplateActions } from "./graphStoreEditorTemplateActions";
import { createGraphStorePersistenceActions } from "./graphStorePersistenceActions";

export function createGraphStoreEditorActions(set, get) {
  return {
    ...createGraphStoreEditorDraftActions(set, get),
    ...createGraphStoreEditorSelectionActions(set, get),
    ...createGraphStoreEditorTemplateActions(set, get),
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
    set({ graph: finalGraph, selectedNodeId: node.id, selectedEdgeId: null, quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
  },

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
  },

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
  },
    ...createGraphStoreCompileActions(set, get),
    ...createGraphStorePersistenceActions(set, get)
  };
}
