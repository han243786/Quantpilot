import { translateText } from "../i18n";

export function formatValue(value) {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(4);
  }
  return String(value);
}

export function stringifyJson(value) {
  if (value === null || value === undefined) return "";
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return "";
  }
}

export function compileOutputsText(outputs) {
  if (!outputs) return "-";
  return `${outputs.data_sources || 0}/${outputs.intent_generators || 0}/${outputs.agents || 0}/${outputs.risk_controls || 0}/${outputs.executions || 0}`;
}

export function strategyIrSourceFromGraph(graph) {
  const artifact = graph.metadata?.artifacts?.strategy_ir;
  if (!artifact) return "";
  if (typeof artifact === "string") return artifact;
  if (typeof artifact !== "object") return "";
  if (typeof artifact.source === "string") return artifact.source;
  if (artifact.document && typeof artifact.document === "object") {
    return stringifyJson(artifact.document);
  }
  if (artifact.ir_version) {
    return stringifyJson(artifact);
  }
  return "";
}

export function findTargetRangeInSource(source, target) {
  if (!source || !target) return null;

  const searchTerms = Array.isArray(target.search_terms) ? target.search_terms.filter(Boolean) : [];
  if (searchTerms.length > 0) {
    let anchor = 0;
    let matchIndex = -1;
    let matchText = "";
    for (const term of searchTerms) {
      const nextIndex = source.indexOf(term, anchor);
      if (nextIndex === -1) {
        matchIndex = -1;
        break;
      }
      matchIndex = nextIndex;
      matchText = term;
      anchor = nextIndex + term.length;
    }
    if (matchIndex >= 0) {
      return [matchIndex, matchIndex + matchText.length];
    }
  }

  const fallbackTerms = [
    target.field ? `"${String(target.field).split(".").pop()}"` : "",
    target.label || "",
    target.field || ""
  ].filter(Boolean);
  for (const term of fallbackTerms) {
    const index = source.indexOf(term);
    if (index >= 0) {
      return [index, index + term.length];
    }
  }
  return null;
}

export function diagnosticSeverityCounts(diagnostics = []) {
  return diagnostics.reduce(
    (summary, diagnostic) => {
      if (diagnostic?.severity === "warning") {
        summary.warning += 1;
      } else if (diagnostic?.severity === "info") {
        summary.info += 1;
      } else {
        summary.blocker += 1;
      }
      return summary;
    },
    { blocker: 0, warning: 0, info: 0 }
  );
}

export function booleanStatusTone(value) {
  if (value === true) return "success";
  if (value === false) return "danger";
  return "muted";
}

export function booleanStatusText(value) {
  if (value === true) return translateText("是");
  if (value === false) return translateText("否");
  return "-";
}

export function strategyIrRoleText(artifactResolution) {
  return artifactResolution?.strategy_ir_role_label || "-";
}

export function runtimeSourceText(artifactResolution) {
  return artifactResolution?.runtime_source_label || "-";
}

export function runtimeSourceOfTruthText(artifactResolution) {
  return artifactResolution?.source_of_truth_label || "-";
}

