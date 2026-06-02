import {
  defaultCapabilities,
  defaultRegistry,
  fallbackRunnableGraph,
  resolveStrategyIrDraft
} from "./graphStoreHelpers";

export function createInitialRuntimeState() {
  return {
    runId: null,
    runKind: null,
    status: "idle",
    connectionState: "disconnected",
    account: null,
    artifactPersistenceStatus: "idle",
    backtestArtifacts: null,
    diagnostics: null,
    governance: null,
    events: [],
    timeline: [],
    retainedKeyEventIndex: null,
    compactEvidence: null,
    parameterMutations: [],
    backendError: null,
    history: [],
    historyStatus: "idle",
    backtestHistory: [],
    backtestHistoryStatus: "idle",
    experiments: [],
    experimentsStatus: "idle",
    backtestCompareSelection: {},
    actionLock: null,
    compileResultNotice: null,
    diagnosticFocusRequested: false,
    selectedHistoryRunId: null,
    selectedRunStatus: "idle",
    selectedBacktestId: null,
    selectedExperimentId: null,
    selectedExperiment: null,
    selectedExperimentStatus: "idle",
    highlightedNodeIds: []
  };
}

export function createInitialGraphStoreState({ registry = defaultRegistry } = {}) {
  const graph = fallbackRunnableGraph(registry);

  return {
    registry,
    capabilities: defaultCapabilities,
    capabilityStatus: "ready",
    capabilitySource: "remote",
    capabilityMessage: "",
    graphIndex: [],
    graphIndexStatus: "idle",
    graphIndexMessage: "",
    graphVersions: [],
    graphVersionsStatus: "idle",
    graphVersionsMessage: "",
    graphVersionPreview: null,
    graphVersionPreviewStatus: "idle",
    graphVersionPreviewMessage: "",
    graphVersionCompare: null,
    graphVersionCompareStatus: "idle",
    graphVersionCompareMessage: "",
    graphAuditHistory: [],
    graphAuditHistoryStatus: "idle",
    graphAuditHistoryMessage: "",
    graph,
    selectedNodeId: null,
    selectedEdgeId: null,
    selectedCompileDiagnosticTarget: null,
    runtime: createInitialRuntimeState(),
    compileResult: null,
    compileStatus: "idle",
    runtimeController: null,
    quantScriptDraft: "",
    formalQuantScriptDraft: null,
    formalQuantScriptOverride: null,
    strategyIrDraft: resolveStrategyIrDraft(graph, "")
  };
}
