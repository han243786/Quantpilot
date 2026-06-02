import { getCapabilityActionBlockReason } from "../capabilities/supportMatrix";
import { runGraphCompileFlow } from "./graphStoreCompileFlow";
import {
  buildCompileFailureState,
  buildCompileSuccessState,
  buildCompileValidationFailureState
} from "./graphStoreCompileState";
import { createGraphStoreCompileExportActions } from "./graphStoreCompileExportActions";
import { createGraphStoreCompileSourceActions } from "./graphStoreCompileSourceActions";
import { saveGraphToStorage } from "./graphStoreHelpers";

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
    ...createGraphStoreCompileSourceActions(set, get),
    ...createGraphStoreCompileExportActions(set, get),

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
