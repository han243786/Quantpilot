import { humanizeErrorText, sanitizeDisplayText } from "../utils/errorText";
import { COMPILE_CONTRACT } from "../utils/compileContract";
import {
  attachCoreIrArtifact,
  buildStrategyIrLabelTargets,
  parseJsonValue,
  quantScriptLabelTargets,
  resolveStrategyIrArtifact,
  resolveStrategyIrCompileSource,
  resolveStrategyIrDocument,
  resolveStrategyIrDraft,
  strategyIrLabelTargets,
  stringifyJson
} from "./graphStoreCompileProtocolMapping";

function sanitizeText(value, fallback) {
  return sanitizeDisplayText(value, fallback);
}

function mergeCompileSummary(localSummary, backendCompile, graph = null, source = null) {
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

function normalizeCompileDiagnosticSource(source) {
  if (source === "strategy_ir" || source === "formal_quantscript" || source === "runtime") {
    return source;
  }
  return "graph";
}

function buildStrategyIrCheckSummary(strategyIrCompile = null) {
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

function buildArtifactResolutionSummary({ hasStrategyIrArtifact, runtimeSource }) {
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

function buildCompileFailureSummary(localSummary, error, graph = null, source = null) {
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

export function resolveCompileDiagnosticTargetFromGraphArtifacts(graph, label) {
  if (!label) return null;
  const target = quantScriptLabelTargets(graph)?.[label] || strategyIrLabelTargets(graph)?.[label];
  if (target && typeof target === "object") {
    return {
      scope: target.scope || "graph",
      node_id: target.node_id || null,
      edge_id: target.edge_id || null,
      field: target.field || null,
      label: sanitizeText(target.label, label),
      search_terms: Array.isArray(target.search_terms) ? target.search_terms.filter(Boolean) : []
    };
  }
  if (resolveStrategyIrArtifact(graph)) {
    const parts = String(label).split(".");
    const lastPart = parts[parts.length - 1] || label;
    const signalScopedCustomExpr =
      parts.length >= 3 && parts[1] === "params" && parts[2] === "custom_expr";
    return {
      scope: "strategy_ir",
      node_id: null,
      edge_id: null,
      field: label,
      label,
      search_terms: signalScopedCustomExpr
        ? [`"signal_id": "${parts[0]}"`, '"custom_expr"']
        : [`"${lastPart.replace(/\[\d+\]/g, "")}"`]
    };
  }
  return null;
}

function normalizeCompileDiagnostic(diagnostic, graph = null) {
  return {
    code: diagnostic?.code || "COMPILE_DIAGNOSTIC",
    source: normalizeCompileDiagnosticSource(diagnostic?.source),
    severity: diagnostic?.severity || "error",
    message: sanitizeText(diagnostic?.message, "Compile diagnostic."),
    span_label: sanitizeText(diagnostic?.span_label, ""),
    target: normalizeCompileDiagnosticTarget(diagnostic?.target, graph, diagnostic?.span_label),
    hint: sanitizeText(diagnostic?.hint, "")
  };
}

function normalizeCompileDiagnosticTarget(target, graph = null, spanLabel = null) {
  const mappedFromSpan = resolveCompileDiagnosticTargetFromGraphArtifacts(graph, spanLabel);
  if (!target) return mappedFromSpan;
  if (typeof target === "string") {
    const mappedTarget = resolveCompileDiagnosticTargetFromGraphArtifacts(graph, target);
    if (mappedTarget) return mappedTarget;
    return {
      scope: "graph",
      node_id: null,
      edge_id: null,
      field: target,
      label: target
    };
  }
  if (typeof target !== "object") {
    return mappedFromSpan;
  }

  return {
    scope: target.scope || "graph",
    node_id: target.node_id || null,
    edge_id: target.edge_id || null,
    field: target.field || null,
    label: sanitizeText(
      target.label,
      target.node_id || target.edge_id || target.field || spanLabel || "Compile target"
    ),
    search_terms: Array.isArray(target.search_terms) ? target.search_terms.filter(Boolean) : []
  };
}

export function parseQuantScriptDiagnosticsFromMessage(
  message,
  graph = null,
  source = "formal_quantscript"
) {
  if (!message) return [];

  return String(message)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const match = line.match(/^(QS\d{4}|Q[A-Z0-9_]+):\s*(.+?)(?:\s+\[([^\]]+)\])?$/);
      if (!match) return null;
      const [, code, diagnosticMessage, spanLabel] = match;
      return normalizeCompileDiagnostic(
        {
          code,
          source,
          severity: "error",
          message: diagnosticMessage,
          span_label: spanLabel || null
        },
        graph
      );
    })
    .filter(Boolean);
}

function compileDiagnosticsFromBackendError(error, graph = null, source = null) {
  if (!Array.isArray(error?.details) || error.details.length === 0) {
    return parseQuantScriptDiagnosticsFromMessage(
      error?.message,
      graph,
      source || error?.compile_source || "runtime"
    );
  }

  const diagnostics = error.details.map((detail) =>
    normalizeCompileDiagnostic(
      {
        code: detail?.code || "BACKEND_COMPILE_ERROR",
        source: detail?.source || source || error?.compile_source || "runtime",
        severity: "error",
        message: detail?.message || error?.message || "后端编译失败。",
        target: detail?.target || null,
        span_label: detail?.span_label || null,
        hint: detail?.reason || ""
      },
      graph
    )
  );

  return diagnostics.length > 0
    ? diagnostics
    : parseQuantScriptDiagnosticsFromMessage(
        error?.message,
        graph,
        source || error?.compile_source || "runtime"
      );
}

export {
  attachCoreIrArtifact,
  buildArtifactResolutionSummary,
  buildCompileFailureSummary,
  buildStrategyIrCheckSummary,
  buildStrategyIrLabelTargets,
  compileDiagnosticsFromBackendError,
  mergeCompileSummary,
  normalizeCompileDiagnostic,
  normalizeCompileDiagnosticSource,
  normalizeCompileDiagnosticTarget,
  parseJsonValue,
  quantScriptLabelTargets,
  resolveStrategyIrArtifact,
  resolveStrategyIrCompileSource,
  resolveStrategyIrDocument,
  resolveStrategyIrDraft,
  strategyIrLabelTargets,
  stringifyJson
};
