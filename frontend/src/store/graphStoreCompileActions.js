import { compileGraph } from "../graph/compileGraph";
import { getCapabilityActionBlockReason } from "../capabilities/supportMatrix";
import { runGraphCompileFlow } from "./graphStoreCompileFlow";
import {
  buildCompileFailureState,
  buildCompileSuccessState,
  buildCompileValidationFailureState,
  buildFormalQuantScriptDraftState,
  buildApplyStrategyIrState,
  buildRuntimeExportFallback
} from "./graphStoreCompileState";
import {
  attachValidationWithRegistry,
  buildStrategyIrLabelTargets,
  parseJsonValue,
  resolveStrategyIrDraft,
  saveGraphToStorage,
  stringifyJson
} from "./graphStoreHelpers";

function buildCapabilityBlockedCompileState(state, graph, reason) {
  const compileSummary = {
    compilable: false,
    backend_verified: false,
    errors: [reason],
    diagnostics: [
      {
        code: "CAPABILITY_BOUNDARY",
        severity: "error",
        target: "capabilities.permission_boundary",
        message: reason
      }
    ]
  };

  return {
    graph: {
      ...graph,
      compile_summary: compileSummary
    },
    compileResult: {
      graph,
      compile_summary: compileSummary,
      backend_compile_error: null
    },
    runtime: {
      ...state.runtime,
      status: "error",
      backendError: reason
    }
  };
}

export function createGraphStoreCompileActions(set, get) {
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
        throw new Error("Strategy IR JSON 解析失败。");
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
    },

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
    },

    async compileCurrentGraph() {
      const graph = get().graph;
      if (!graph.validation_state.is_valid) return null;
      if (get().actionLock) return null;
      set({ actionLock: "compiling" });
      try {
        const capabilityBlockReason = getCapabilityActionBlockReason({
          actionKey: "compile",
        capabilityStatus: get().capabilityStatus,
        capabilitySource: get().capabilitySource,
        capabilityMessage: get().capabilityMessage,
        capabilities: get().capabilities
      });
      if (capabilityBlockReason) {
        set((state) =>
          buildCapabilityBlockedCompileState(state, graph, capabilityBlockReason)
        );
        return null;
      }

      const outcome = await runGraphCompileFlow({
        graph,
        registry: get().registry,
        formalQuantScriptOverride: get().formalQuantScriptOverride,
        strategyIrDraft: get().strategyIrDraft
      });

      saveGraphToStorage(outcome.nextGraph);

      // 编译期间graph若被并发修改，放弃过期结果
      if (get().graph.metadata?.graph_id !== graph.metadata?.graph_id) {
        return null;
      }

      if (outcome.status === "validation_failure") {
        const errCount = outcome.localResult?.diagnostics?.length || 0;
        set({
          compileResultNotice: { type: "error", text: `编译失败 — ${errCount} 个错误`, time: Date.now() }
        });
        set(
          buildCompileValidationFailureState(
            outcome.localResult,
            outcome.nextGraph,
            outcome.strategyIrDraft
          )
        );
        return null;
      }

      if (outcome.status === "failure") {
        set({
          compileResultNotice: { type: "error", text: "编译异常 — 请查看诊断", time: Date.now() }
        });
        set(
          buildCompileFailureState(
            outcome.localResult,
            outcome.nextGraph,
            outcome.error,
            outcome.strategyIrDraft
          )
        );
        return null;
      }

      set({
        compileResultNotice: { type: "success", text: "编译成功", time: Date.now() }
      });
      set(
        buildCompileSuccessState(
          outcome.localResult,
          outcome.nextGraph,
          outcome.runtimeConfig,
          outcome.runtimeTargets,
          outcome.backendCompile,
          outcome.strategyIrCompile,
          outcome.strategyIrDraft
        )
      );
      return outcome.result;
      } finally {
        set({ actionLock: null });
      }
    }
  };
}
