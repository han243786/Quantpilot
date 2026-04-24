export function attachCoreIrArtifact(graph, coreIr) {
  if (!coreIr) return graph;
  return {
    ...graph,
    metadata: {
      ...graph.metadata,
      artifacts: {
        ...(graph.metadata?.artifacts || {}),
        core_ir: coreIr
      }
    }
  };
}

export function parseJsonValue(source) {
  if (typeof source !== "string") return null;
  try {
    return JSON.parse(source.trimStart().replace(/^\uFEFF/, ""));
  } catch {
    return null;
  }
}

export function stringifyJson(value) {
  if (value === null || value === undefined) return "";
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return "";
  }
}

export function resolveStrategyIrArtifact(graph) {
  return graph?.metadata?.artifacts?.strategy_ir || null;
}

export function resolveStrategyIrDocument(graph) {
  const artifact = resolveStrategyIrArtifact(graph);
  if (!artifact) return null;
  if (typeof artifact === "string") {
    return parseJsonValue(artifact);
  }
  if (typeof artifact !== "object") {
    return null;
  }
  if (artifact.document && typeof artifact.document === "object") {
    return artifact.document;
  }
  if (artifact.strategy_ir && typeof artifact.strategy_ir === "object") {
    return artifact.strategy_ir;
  }
  if (artifact.source && typeof artifact.source === "string") {
    return parseJsonValue(artifact.source);
  }
  if (artifact.ir_version) {
    return artifact;
  }
  return null;
}

export function buildStrategyIrLabelTargets(strategyIr) {
  if (!strategyIr || typeof strategyIr !== "object") return {};

  const labelTargets = {
    "metadata.strategy_id": {
      scope: "strategy_ir",
      field: "metadata.strategy_id",
      label: "metadata.strategy_id",
      search_terms: ['"strategy_id"']
    },
    "metadata.name": {
      scope: "strategy_ir",
      field: "metadata.name",
      label: "metadata.name",
      search_terms: ['"name"']
    },
    "metadata.summary": {
      scope: "strategy_ir",
      field: "metadata.summary",
      label: "metadata.summary",
      search_terms: ['"summary"']
    },
    execution: {
      scope: "strategy_ir",
      field: "execution",
      label: "execution",
      search_terms: ['"execution"']
    },
    risk_rules: {
      scope: "strategy_ir",
      field: "risk_rules",
      label: "risk_rules",
      search_terms: ['"risk_rules"']
    }
  };

  if (Array.isArray(strategyIr.data_requirements)) {
    strategyIr.data_requirements.forEach((requirement, index) => {
      const dataId = requirement?.data_id || `data_requirements.${index}`;
      labelTargets[dataId] = {
        scope: "strategy_ir",
        field: `data_requirements.${index}`,
        label: dataId,
        search_terms: [`"data_id": "${dataId}"`]
      };
    });
  }

  if (Array.isArray(strategyIr.signals)) {
    strategyIr.signals.forEach((signal, index) => {
      const signalId = signal?.signal_id || `signals.${index}`;
      const signalSearch = [`"signal_id": "${signalId}"`];
      labelTargets[signalId] = {
        scope: "strategy_ir",
        field: `signals.${index}`,
        label: signalId,
        search_terms: signalSearch
      };
      labelTargets[`${signalId}.indicator`] = {
        scope: "strategy_ir",
        field: `${signalId}.indicator`,
        label: `${signalId}.indicator`,
        search_terms: [...signalSearch, '"indicator"']
      };
      labelTargets[`${signalId}.params.custom_expr`] = {
        scope: "strategy_ir",
        field: `${signalId}.params.custom_expr`,
        label: `${signalId}.params.custom_expr`,
        search_terms: [...signalSearch, '"custom_expr"']
      };
      labelTargets[`${signalId}.indicator.params.custom_expr`] = {
        scope: "strategy_ir",
        field: `${signalId}.indicator.params.custom_expr`,
        label: `${signalId}.indicator.params.custom_expr`,
        search_terms: [...signalSearch, '"custom_expr"']
      };
    });
  }

  return labelTargets;
}

export function strategyIrLabelTargets(graph) {
  const artifact = resolveStrategyIrArtifact(graph);
  const explicitTargets =
    artifact && typeof artifact === "object" && artifact.label_targets && typeof artifact.label_targets === "object"
      ? artifact.label_targets
      : {};
  return {
    ...buildStrategyIrLabelTargets(resolveStrategyIrDocument(graph)),
    ...explicitTargets
  };
}

export function resolveStrategyIrDraft(graph, fallback = "") {
  const artifact = resolveStrategyIrArtifact(graph);
  if (!artifact) return fallback;
  if (typeof artifact === "string") return artifact;
  if (typeof artifact !== "object") return fallback;
  if (typeof artifact.source === "string") return artifact.source;
  const document = resolveStrategyIrDocument(graph);
  return stringifyJson(document) || fallback;
}

export function resolveStrategyIrCompileSource(graph) {
  const artifact = resolveStrategyIrArtifact(graph);
  if (!artifact) return null;
  if (typeof artifact === "string" || Array.isArray(artifact)) {
    return artifact;
  }
  if (typeof artifact !== "object") {
    return null;
  }
  if (Object.prototype.hasOwnProperty.call(artifact, "document")) {
    return artifact.document;
  }
  if (Object.prototype.hasOwnProperty.call(artifact, "source")) {
    return artifact.source;
  }
  if (Object.prototype.hasOwnProperty.call(artifact, "strategy_ir")) {
    return artifact.strategy_ir;
  }
  if (artifact.ir_version) {
    return artifact;
  }
  return null;
}

export function quantScriptLabelTargets(graph) {
  return graph?.metadata?.artifacts?.quantscript?.label_targets || {};
}
