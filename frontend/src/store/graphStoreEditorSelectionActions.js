import {
  normalizeCompileDiagnosticTarget,
  resolveStrategyIrDraft
} from "./graphStoreHelpers";

export function createGraphStoreEditorSelectionActions(set, get) {
  return {
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
    }
  };
}
