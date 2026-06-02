import { compileGraph } from "../graph/compileGraph";
import { buildRuntimeExportFallback } from "./graphStoreCompileState";
import {
  attachValidationWithRegistry,
  resolveStrategyIrDraft
} from "./graphStoreHelpers";

export function createGraphStoreCompileExportActions(set, get) {
  return {
    async exportRuntimeConfig() {
      const compiled = await get().compileCurrentGraph();
      if (compiled) return compiled;
      return buildRuntimeExportFallback(get());
    },

    exportQuantScript() {
      const registry = get().registry;
      const result = compileGraph(get().graph, registry);
      const graph = attachValidationWithRegistry(
        { ...result.graph, compile_summary: result.compile_summary },
        registry
      );
      set({
        compileResult: result,
        graph,
        quantScriptDraft:
          graph.metadata?.artifacts?.quantscript?.graph_source || result.quantscript || "",
        strategyIrDraft: resolveStrategyIrDraft(graph, get().strategyIrDraft)
      });
      return result.quantscript;
    }
  };
}
