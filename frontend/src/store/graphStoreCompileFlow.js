import {
  buildCompileFailureOutcome,
  buildCompileSuccessOutcome,
  buildLocalCompileContext,
  buildValidationFailureOutcome
} from "./graphStoreCompileOutcomeMapping";
import {
  compileRuntimeSource,
  verifyStrategyIrArtifact
} from "./graphStoreCompileProtocolFlow";

export async function runGraphCompileFlow({
  graph,
  registry,
  formalQuantScriptOverride,
  strategyIrDraft
}) {
  const context = buildLocalCompileContext({
    graph,
    registry,
    formalQuantScriptOverride
  });

  if (!context.nextGraph.compile_summary.compilable) {
    return buildValidationFailureOutcome(context, strategyIrDraft);
  }

  let strategyIrCompile = null;
  let verifiedSummary = context.nextGraph.compile_summary;
  try {
    ({ strategyIrCompile, verifiedSummary } = await verifyStrategyIrArtifact(context));
    const runtimeResult = await compileRuntimeSource(context);
    return buildCompileSuccessOutcome(
      context,
      registry,
      verifiedSummary,
      runtimeResult,
      strategyIrCompile,
      strategyIrDraft
    );
  } catch (error) {
    return buildCompileFailureOutcome(
      context,
      registry,
      verifiedSummary,
      strategyIrCompile,
      error,
      strategyIrDraft
    );
  }
}
