import { COMPILE_CONTRACT } from "../utils/compileContract";
import { humanizeErrorText } from "../utils/errorText";
import {
  compileDiagnosticsFromBackendError,
  normalizeCompileDiagnostic
} from "./graphStoreCompileDiagnostics";

export function mergeCompileSummary(localSummary, backendCompile, graph = null, source = null) {
  const localDiagnostics = Array.isArray(localSummary.diagnostics) ? localSummary.diagnostics : [];
  const backendDiagnostics = Array.isArray(backendCompile.diagnostics)
    ? backendCompile.diagnostics.map((diagnostic) =>
        normalizeCompileDiagnostic(
          { ...diagnostic, source: diagnostic?.source || source || "runtime" },
          graph
        )
      )
    : [];
  const diagnostics = [...localDiagnostics, ...backendDiagnostics];

  return {
    ...localSummary,
    compilable: backendCompile?.compilable ?? true,
    backend_verified: true,
    backend_verified_at: Date.now(),
    protocol_name: backendCompile?.protocol_name ?? localSummary.protocol_name,
    config_hash: backendCompile?.config_hash ?? localSummary.config_hash,
    outputs: backendCompile?.counts ?? localSummary.outputs,
    diagnostics,
    warnings: diagnostics
      .filter((diagnostic) => diagnostic.severity === "warning")
      .map((diagnostic) => diagnostic.message),
    errors: diagnostics
      .filter((diagnostic) => diagnostic.severity === "error")
      .map((diagnostic) => diagnostic.message)
  };
}

export function buildStrategyIrCheckSummary(strategyIrCompile = null) {
  if (!strategyIrCompile) {
    return {
      performed: false,
      compilable: null,
      compile_id: null,
      has_core_ir: false
    };
  }
  return {
    performed: true,
    compilable: Boolean(strategyIrCompile.compilable),
    compile_id: strategyIrCompile.compile_id || null,
    has_core_ir: Boolean(strategyIrCompile.core_ir)
  };
}

export function buildArtifactResolutionSummary({ hasStrategyIrArtifact, runtimeSource }) {
  const resolvedRuntimeSource =
    runtimeSource === "formal_quantscript" || runtimeSource === "runtime_fallback"
      ? runtimeSource
      : "runtime";
  const runtimeSourceLabel =
    resolvedRuntimeSource === "formal_quantscript"
      ? "Formal QuantScript 代码转换输入"
      : resolvedRuntimeSource === "runtime_fallback"
        ? "图生成的 runtime_config 回退输入"
        : "图生成的 runtime_config 输入";

  return {
    strategy_ir_role: hasStrategyIrArtifact ? "semantic_preflight" : "not_used",
    strategy_ir_role_label: hasStrategyIrArtifact
      ? "只作语义预检，不决定可运行输出"
      : "未启用策略中间表示预检",
    runtime_source: resolvedRuntimeSource,
    runtime_source_label: runtimeSourceLabel,
    source_of_truth: "runtime_compile",
    source_of_truth_label: COMPILE_CONTRACT.runtimeSourceOfTruthLabel,
    notes: [
      hasStrategyIrArtifact
        ? "策略中间表示会先执行语义预检。它可以提前阻断编译，但不决定最终可运行输出。"
        : "当前没有提供策略中间表示工件，因此运行时编译会直接从图生成工件开始。",
      resolvedRuntimeSource === "formal_quantscript"
        ? "Formal QuantScript 代码转换提供运行时编译输入，但最终可运行结果仍以运行时编译输出为准。"
        : resolvedRuntimeSource === "runtime_fallback"
          ? "Formal QuantScript 代码转换不可用，因此运行时编译回退到图生成的 runtime_config；最终可运行结果仍以运行时编译输出为准。"
          : "运行时编译直接使用图生成的 runtime_config 作为输入，最终可运行结果仍以运行时编译输出为准。"
    ]
  };
}

export function buildCompileFailureSummary(localSummary, error, graph = null, source = null) {
  const message = humanizeErrorText(error, "后端编译失败。");
  if (error && typeof error === "object") {
    error.message = message;
  }
  const localDiagnostics = Array.isArray(localSummary.diagnostics) ? localSummary.diagnostics : [];
  const backendDiagnostics = compileDiagnosticsFromBackendError(error, graph, source);
  const diagnostics =
    backendDiagnostics.length > 0
      ? [...localDiagnostics, ...backendDiagnostics]
      : [...localDiagnostics, normalizeCompileDiagnostic({ severity: "error", message }, graph)];

  return {
    ...localSummary,
    compilable: false,
    backend_verified: false,
    backend_verified_at: null,
    backend_error: message,
    diagnostics,
    warnings: diagnostics
      .filter((diagnostic) => diagnostic.severity === "warning")
      .map((diagnostic) => diagnostic.message),
    errors: diagnostics
      .filter((diagnostic) => diagnostic.severity === "error")
      .map((diagnostic) => diagnostic.message)
  };
}
