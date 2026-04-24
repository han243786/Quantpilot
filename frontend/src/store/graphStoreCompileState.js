export function buildFormalQuantScriptDraftState(state, draft) {
  return {
    formalQuantScriptDraft: draft,
    formalQuantScriptOverride: draft,
    selectedCompileDiagnosticTarget: null,
    compileResult: null,
    runtime: {
      ...state.runtime
    }
  };
}

export function buildApplyStrategyIrState(
  state,
  graph,
  normalizedSource,
  strategyIrDraft
) {
  return {
    graph,
    selectedNodeId: null,
    selectedEdgeId: null,
    selectedCompileDiagnosticTarget: null,
    compileResult: null,
    quantScriptDraft:
      graph.metadata?.artifacts?.quantscript?.graph_source || state.quantScriptDraft,
    strategyIrDraft: strategyIrDraft || normalizedSource,
    runtime: {
      ...state.runtime
    }
  };
}

export function buildCompileValidationFailureState(
  localResult,
  nextGraph,
  strategyIrDraft
) {
  return {
    graph: nextGraph,
    compileResult: {
      ...localResult,
      backend_compile_error: null
    },
    quantScriptDraft:
      nextGraph.metadata?.artifacts?.quantscript?.graph_source || localResult.quantscript || "",
    strategyIrDraft
  };
}

export function buildCompileSuccessState(
  localResult,
  nextGraph,
  runtimeConfig,
  runtimeTargets,
  backendCompile,
  strategyIrCompile,
  strategyIrDraft
) {
  return {
    graph: nextGraph,
    compileResult: {
      ...localResult,
      runtime_config: runtimeConfig,
      runtime_targets: runtimeTargets,
      backend_compile: backendCompile,
      strategy_ir_compile: strategyIrCompile
    },
    quantScriptDraft:
      nextGraph.metadata?.artifacts?.quantscript?.graph_source || localResult.quantscript || "",
    strategyIrDraft
  };
}

export function buildCompileFailureState(
  localResult,
  nextGraph,
  error,
  strategyIrDraft
) {
  return {
    graph: nextGraph,
    compileResult: {
      ...localResult,
      backend_compile_error: error?.compile_source === "formal_quantscript" ? error : null
    },
    quantScriptDraft:
      nextGraph.metadata?.artifacts?.quantscript?.graph_source || localResult.quantscript || "",
    strategyIrDraft
  };
}

export function buildRuntimeExportFallback(state) {
  return {
    ...(state.compileResult || {}),
    graph: state.graph,
    compile_summary:
      state.graph?.compile_summary || state.compileResult?.compile_summary || {
        compilable: false,
        errors: ["导出运行配置失败。"]
      }
  };
}
