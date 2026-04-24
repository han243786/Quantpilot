import {
  attachCoreIrArtifact,
  attachValidationWithRegistry,
  buildCompileFailureSummary,
  buildStrategyIrCheckSummary,
  mergeCompileSummary,
  resolveStrategyIrDraft,
  withRuntimeBinding
} from "./graphStoreHelpers";

export function buildStrategyIrCompileOutcome(context, strategyIrCompile) {
  return {
    strategyIrCompile,
    verifiedSummary: {
      ...mergeCompileSummary(
        context.nextGraph.compile_summary,
        strategyIrCompile,
        context.nextGraph,
        "strategy_ir"
      ),
      strategy_ir_check: buildStrategyIrCheckSummary(strategyIrCompile),
      artifact_resolution: context.compileResolution
    }
  };
}

export function buildValidationFailureOutcome(context, strategyIrDraft) {
  return {
    status: "validation_failure",
    localResult: context.localResult,
    nextGraph: context.nextGraph,
    strategyIrDraft: resolveStrategyIrDraft(context.nextGraph, strategyIrDraft)
  };
}

export function buildCompileSuccessOutcome(
  context,
  registry,
  verifiedSummary,
  runtimeResult,
  strategyIrCompile,
  strategyIrDraft
) {
  const nextGraph = attachValidationWithRegistry(
    withRuntimeBinding(
      attachCoreIrArtifact(
        {
          ...context.nextGraph,
          compile_summary: {
            ...mergeCompileSummary(
              verifiedSummary,
              runtimeResult.backendCompile,
              context.nextGraph,
              runtimeResult.runtimeCompileSource === "formal_quantscript"
                ? "formal_quantscript"
                : "runtime"
            ),
            strategy_ir_check: buildStrategyIrCheckSummary(strategyIrCompile),
            artifact_resolution: runtimeResult.compileResolution
          }
        },
        runtimeResult.backendCompile.core_ir
      ),
      { last_compile_id: runtimeResult.backendCompile.compile_id }
    ),
    registry
  );

  return {
    status: "success",
    localResult: context.localResult,
    nextGraph,
    runtimeConfig: runtimeResult.runtimeConfig,
    runtimeTargets: runtimeResult.runtimeTargets,
    backendCompile: runtimeResult.backendCompile,
    strategyIrCompile,
    strategyIrDraft: resolveStrategyIrDraft(nextGraph, strategyIrDraft),
    result: {
      ...context.localResult,
      runtime_config: runtimeResult.runtimeConfig,
      runtime_targets: runtimeResult.runtimeTargets,
      backend_compile: runtimeResult.backendCompile,
      strategy_ir_compile: strategyIrCompile,
      graph: nextGraph
    }
  };
}

export function inferCompileFailureSource(context, strategyIrCompile, error) {
  if (error?.compile_source) {
    return error.compile_source;
  }
  if (context.hasStrategyIrArtifact && !strategyIrCompile) {
    return "strategy_ir";
  }
  return context.runtimeCompileSource === "formal_quantscript"
    ? "formal_quantscript"
    : "runtime";
}

export function buildCompileFailureOutcome(
  context,
  registry,
  verifiedSummary,
  strategyIrCompile,
  error,
  strategyIrDraft
) {
  const compileSource = inferCompileFailureSource(context, strategyIrCompile, error);
  error.compile_source = compileSource;

  const nextGraph = attachValidationWithRegistry(
    {
      ...context.nextGraph,
      compile_summary: {
        ...buildCompileFailureSummary(
          verifiedSummary,
          error,
          context.nextGraph,
          compileSource
        ),
        strategy_ir_check:
          context.hasStrategyIrArtifact && !strategyIrCompile
            ? {
                ...buildStrategyIrCheckSummary({
                  compilable: false,
                  compile_id: context.localResult.compile_id,
                  core_ir: null
                }),
                performed: true
              }
            : buildStrategyIrCheckSummary(strategyIrCompile),
        artifact_resolution: context.compileResolution
      }
    },
    registry
  );

  return {
    status: "failure",
    localResult: context.localResult,
    nextGraph,
    error,
    strategyIrDraft: resolveStrategyIrDraft(nextGraph, strategyIrDraft)
  };
}
