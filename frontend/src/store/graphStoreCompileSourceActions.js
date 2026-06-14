import {
  buildApplyStrategyIrState,
  buildFormalQuantScriptDraftState
} from "./graphStoreCompileState";
import {
  attachValidationWithRegistry,
  buildStrategyIrLabelTargets,
  parseJsonValue,
  resolveStrategyIrDraft,
  saveGraphToStorage,
  stringifyJson
} from "./graphStoreHelpers";

export function createGraphStoreCompileSourceActions(set, get) {
  return {
    applyFormalQuantScriptSource(source = null) {
      const graph = get().graph;
      const graphFormalSource = graph.metadata?.artifacts?.quantscript?.formal_source || "";
      const draft = source ?? get().formalQuantScriptDraft ?? graphFormalSource;
      if (!String(draft || "").trim()) {
        throw new Error("Formal QuantScript 不能为空。");
      }
      set((state) => buildFormalQuantScriptDraftState(state, draft));
      return draft;
    },

    applyStrategyIrSource(source = null) {
      const draft = source ?? get().strategyIrDraft;
      const parsed = parseJsonValue(draft);
      if (!parsed || typeof parsed !== "object") {
        throw new Error("策略中间表示 JSON 解析失败。");
      }

      const registry = get().registry;
      const currentGraph = get().graph;
      const normalizedSource = stringifyJson(parsed);
      const graph = attachValidationWithRegistry(
        {
          ...currentGraph,
          metadata: {
            ...currentGraph.metadata,
            source_mode: "strategy_ir",
            updated_at: Date.now(),
            artifacts: {
              ...(currentGraph.metadata?.artifacts || {}),
              strategy_ir: {
                document: parsed,
                source: normalizedSource,
                label_targets: buildStrategyIrLabelTargets(parsed),
                generated_at: Date.now()
              }
            }
          }
        },
        registry
      );
      saveGraphToStorage(graph);
      set((state) =>
        buildApplyStrategyIrState(
          state,
          graph,
          normalizedSource,
          resolveStrategyIrDraft(graph, normalizedSource)
        )
      );
      return graph;
    }
  };
}
