import { compileGraph } from "../graph/compileGraph";
import {
  attachValidationWithRegistry,
  buildArtifactResolutionSummary,
  buildStrategyIrCheckSummary,
  resolveStrategyIrCompileSource,
} from "./graphStoreHelpers";
export {
  buildCompileFailureOutcome,
  buildCompileSuccessOutcome,
  buildStrategyIrCompileOutcome,
  buildValidationFailureOutcome,
  inferCompileFailureSource
} from "./graphStoreCompileOutcomeProjection";

export function buildLocalCompileContext({ graph, registry, formalQuantScriptOverride }) {
  const localResult = compileGraph(graph, registry);
  const strategyIr = resolveStrategyIrCompileSource(localResult.graph);
  const hasStrategyIrArtifact = strategyIr !== null && strategyIr !== undefined;
  const graphFormalSource = localResult.graph.metadata?.artifacts?.quantscript?.formal_source || "";
  const formalSource = formalQuantScriptOverride ?? graphFormalSource;
  const runtimeCompileSource = formalSource.trim() ? "formal_quantscript" : "runtime";
  const compileResolution = buildArtifactResolutionSummary({
    hasStrategyIrArtifact,
    runtimeSource: runtimeCompileSource
  });
  const nextGraph = attachValidationWithRegistry(
    {
      ...localResult.graph,
      metadata: {
        ...localResult.graph.metadata,
        runtime_binding: {
          ...localResult.graph.metadata.runtime_binding,
          last_compile_id: localResult.compile_id
        }
      },
      compile_summary: {
        ...localResult.compile_summary,
        strategy_ir_check: buildStrategyIrCheckSummary(),
        artifact_resolution: compileResolution
      }
    },
    registry
  );

  return {
    localResult,
    nextGraph,
    hasStrategyIrArtifact,
    strategyIr,
    formalSource,
    runtimeCompileSource,
    compileResolution
  };
}
