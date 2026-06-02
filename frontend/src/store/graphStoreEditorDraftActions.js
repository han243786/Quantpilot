import { parseGraphQuantScript } from "../graph/quantscript";
import {
  attachValidationWithRegistry,
  resolveStrategyIrDraft,
  saveGraphToStorage
} from "./graphStoreHelpers";

export function createGraphStoreEditorDraftActions(set, get) {
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
    }
  };
}
