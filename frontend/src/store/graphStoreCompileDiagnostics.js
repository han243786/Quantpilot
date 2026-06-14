import { sanitizeDisplayText } from "../utils/errorText";
import {
  quantScriptLabelTargets,
  resolveStrategyIrArtifact,
  strategyIrLabelTargets
} from "./graphStoreCompileProtocolMapping";

function sanitizeText(value, fallback) {
  return sanitizeDisplayText(value, fallback);
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

export function normalizeCompileDiagnostic(diagnostic, graph = null) {
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

export function normalizeCompileDiagnosticSource(source) {
  if (source === "strategy_ir" || source === "formal_quantscript" || source === "runtime") {
    return source;
  }
  return "graph";
}

export function normalizeCompileDiagnosticTarget(target, graph = null, spanLabel = null) {
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

export function compileDiagnosticsFromBackendError(error, graph = null, source = null) {
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
