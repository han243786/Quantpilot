import {
  requestFormalQuantScriptCompile,
  requestRuntimeCompile,
  requestStrategyIrCompile
} from "./graphStoreCompileApi";
import { buildArtifactResolutionSummary } from "./graphStoreHelpers";
import { buildStrategyIrCompileOutcome } from "./graphStoreCompileOutcomeMapping";

export async function verifyStrategyIrArtifact(context) {
  if (!context.hasStrategyIrArtifact) {
    return {
      strategyIrCompile: null,
      verifiedSummary: context.nextGraph.compile_summary
    };
  }

  const strategyIrCompile = await requestStrategyIrCompile(
    context.nextGraph.metadata.graph_id,
    context.localResult.compile_id,
    context.strategyIr
  );

  return {
    strategyIrCompile,
    verifiedSummary: buildStrategyIrCompileOutcome(context, strategyIrCompile).verifiedSummary
  };
}

export async function compileRuntimeSource(context) {
  const formalRuntimeTargets =
    context.nextGraph.metadata?.artifacts?.quantscript?.runtime_targets || {
      source_to_node: {},
      runtime_node_id: null,
      execution_node_id: null
    };

  if (!context.formalSource.trim()) {
    // v2.4.0 G2: Formal QS 无法生成时记录原因, 避免用户无感知降级
    console.warn("[compile] Formal QuantScript 源码为空, 使用 runtime 路径编译");
    return buildRuntimeCompileResult(
      await requestRuntimeCompile(context.localResult.runtime_config, context.nextGraph),
      context.runtimeCompileSource,
      context.compileResolution,
      formalRuntimeTargets,
      context.localResult.runtime_config
    );
  }

  try {
    return buildRuntimeCompileResult(
      await requestFormalQuantScriptCompile({
        graphId: context.nextGraph.metadata.graph_id,
        compileId: context.localResult.compile_id,
        source: context.formalSource,
        runtimeTemplate: context.localResult.runtime_config,
        runtimeTargets: formalRuntimeTargets
      }),
      context.runtimeCompileSource,
      context.compileResolution,
      formalRuntimeTargets,
      context.localResult.runtime_config
    );
  } catch (error) {
    if (error?.status && error.status !== 404 && error.status < 500) {
      error.compile_source = "formal_quantscript";
      throw error;
    }
    // v2.4.0 G3: 5xx/网络错误静默降级到 runtime 路径时记录警告
    console.warn(
      "[compile] Formal QuantScript 编译失败 (5xx/网络), 降级到 runtime 路径:",
      error?.message || error
    );
    return buildRuntimeCompileResult(
      await requestRuntimeCompile(context.localResult.runtime_config, context.nextGraph),
      "runtime_fallback",
      buildArtifactResolutionSummary({
        hasStrategyIrArtifact: context.hasStrategyIrArtifact,
        runtimeSource: "runtime_fallback"
      }),
      formalRuntimeTargets,
      context.localResult.runtime_config
    );
  }
}

function buildRuntimeCompileResult(
  backendCompile,
  runtimeCompileSource,
  compileResolution,
  formalRuntimeTargets,
  localRuntimeConfig
) {
  return {
    backendCompile,
    runtimeCompileSource,
    compileResolution,
    // v2.4.0 G7: 后端 runtime_targets 为权威来源, 前端版本仅为近似
    runtimeTargets: backendCompile.runtime_targets || formalRuntimeTargets,
    runtimeConfig: backendCompile.runtime_config || localRuntimeConfig,
    // 检测前后端 ID 映射差异, 便于发现 sanitize 规则不一致
    _runtimeTargetSource: backendCompile.runtime_targets ? "backend" : "frontend_local"
  };
}
