import { createNodeFromModule } from "../graph/createNode";
import { parseGraphQuantScript } from "../graph/quantscript";
import { buildStrategyTemplateGraph } from "../templates/strategyTemplates";
import {
  attachValidationWithRegistry,
  normalizeCompileDiagnosticTarget,
  recordRecentNodeIds,
  resolveStrategyIrDraft,
  saveGraphToStorage,
  withRecentNodeIds
} from "./graphStoreHelpers";
import { createGraphStoreCompileActions } from "./graphStoreCompileActions";
import { createGraphStorePersistenceActions } from "./graphStorePersistenceActions";

export function createGraphStoreEditorActions(set, get) {
  return {
  updateQuantScriptDraft(source) {
    set({ quantScriptDraft: source });
  },

  updateFormalQuantScriptDraft(source) {
    set({ formalQuantScriptDraft: source });
  },

  updateStrategyIrDraft(source) {
    set({ strategyIrDraft: source });
  },

  resetQuantScriptDraft() {
    const graph = get().graph;
    set({ quantScriptDraft: graph.metadata?.artifacts?.quantscript?.graph_source || "" });
  },

  resetFormalQuantScriptDraft() {
    set({
      formalQuantScriptDraft: null,
      formalQuantScriptOverride: null,
      selectedCompileDiagnosticTarget: null,
      compileResult: null
    });
  },

  resetStrategyIrDraft() {
    const graph = get().graph;
    set({ strategyIrDraft: resolveStrategyIrDraft(graph, ""), selectedCompileDiagnosticTarget: null });
  },

  applyQuantScriptSource(source = null) {
    const draft = source ?? get().quantScriptDraft;
    const registry = get().registry;
    const parsed = parseGraphQuantScript(draft, registry, get().graph);
    const graph = attachValidationWithRegistry({
      ...parsed,
      metadata: {
        ...parsed.metadata,
        updated_at: Date.now()
      }
    }, registry);
    saveGraphToStorage(graph);
    set((state) => ({
      graph,
      selectedNodeId: null,
      selectedEdgeId: null,
      selectedCompileDiagnosticTarget: null,
      compileResult: null,
      quantScriptDraft: graph.metadata?.artifacts?.quantscript?.graph_source || draft,
      strategyIrDraft: resolveStrategyIrDraft(graph, state.strategyIrDraft),
      runtime: {
        ...state.runtime
      }
    }));
    return graph;
  },

  setSelectedNode(nodeId) {
    set({ selectedNodeId: nodeId, selectedEdgeId: null, selectedCompileDiagnosticTarget: null });
  },

  setSelectedEdge(edgeId) {
    set({ selectedNodeId: null, selectedEdgeId: edgeId, selectedCompileDiagnosticTarget: null });
  },

  focusCompileDiagnostic(target) {
    const normalizedTarget = normalizeCompileDiagnosticTarget(target, get().graph);
    if (normalizedTarget?.scope === "node" && normalizedTarget.node_id) {
      set({ selectedNodeId: normalizedTarget.node_id, selectedEdgeId: null, selectedCompileDiagnosticTarget: null });
      return;
    }
    if (normalizedTarget?.scope === "edge" && normalizedTarget.edge_id) {
      set({ selectedNodeId: null, selectedEdgeId: normalizedTarget.edge_id, selectedCompileDiagnosticTarget: null });
      return;
    }
    if (normalizedTarget?.scope === "strategy_ir") {
      set({
        selectedNodeId: null,
        selectedEdgeId: null,
        selectedCompileDiagnosticTarget: normalizedTarget,
        strategyIrDraft: resolveStrategyIrDraft(get().graph, get().strategyIrDraft)
      });
      return;
    }
    set({ selectedNodeId: null, selectedEdgeId: null, selectedCompileDiagnosticTarget: normalizedTarget || null });
  },

  loadStrategyTemplate(templateId) {
    const registry = get().registry;
    const graph = attachValidationWithRegistry(buildStrategyTemplateGraph(templateId, registry), registry);
    saveGraphToStorage(graph);
    set((state) => ({
      graph,
      selectedNodeId: null,
      selectedEdgeId: null,
      selectedCompileDiagnosticTarget: null,
      compileResult: null,
      formalQuantScriptDraft: null,
      formalQuantScriptOverride: null,
      quantScriptDraft: graph.metadata?.artifacts?.quantscript?.graph_source || "",
      strategyIrDraft: resolveStrategyIrDraft(graph, ""),
      graphVersions: [],
      graphVersionsStatus: "idle",
      graphVersionsMessage: "",
      graphVersionPreview: null,
      graphVersionPreviewStatus: "idle",
      graphVersionPreviewMessage: "",
      graphVersionCompare: null,
      graphVersionCompareStatus: "idle",
      graphVersionCompareMessage: "",
      runtime: {
        ...state.runtime,
        runId: null,
        runKind: null,
        status: "idle",
        connectionState: "disconnected",
        account: null,
        backtestArtifacts: null,
        diagnostics: null,
        governance: null,
        events: [],
        timeline: [],
        retainedKeyEventIndex: null,
        compactEvidence: null,
        backendError: null,
        backtestCompareSelection: [],
        selectedHistoryRunId: null,
        selectedBacktestId: null,
        selectedExperimentId: null,
        selectedExperiment: null,
        selectedExperimentStatus: "idle",
        highlightedNodeIds: []
      }
    }));
    return graph;
  },
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
    set({ graph: nextGraph, quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
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
    set({ graph: finalGraph, quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
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
    set({ graph: nextGraph, quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
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
    set({ graph: nextGraph, quantScriptDraft: nextGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
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
    set({ graph: finalGraph, quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "" });
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
