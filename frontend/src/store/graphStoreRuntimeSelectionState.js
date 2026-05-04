import { normalizeRuntimeGovernanceSnapshot } from "../utils/runtimeGovernance";

export function buildPersistedRuntimeSelection({
  runtime,
  runId,
  runKind,
  account,
  artifactPersistenceStatus = "saved",
  backtestArtifacts = null,
  diagnostics = null,
  governance = null,
  events = [],
  timeline = [],
  retainedKeyEventIndex = null,
  compactEvidence = null,
  parameterMutations = [],
  selectedHistoryRunId = null,
  selectedBacktestId = null,
  highlightedNodeIds = []
}) {
  return {
    ...runtime,
    runId,
    runKind,
    status: "completed",
    connectionState: "disconnected",
    account,
    artifactPersistenceStatus,
    backtestArtifacts,
    diagnostics,
    governance: normalizeRuntimeGovernanceSnapshot(
      governance || backtestArtifacts?.manifest?.governance || null
    ),
    events,
    timeline,
    retainedKeyEventIndex,
    compactEvidence,
    parameterMutations,
    backendError: null,
    selectedHistoryRunId,
    selectedBacktestId,
    highlightedNodeIds
  };
}

export function buildPersistedRuntimeSelectionState(state, nextGraph, selection) {
  return {
    graph: nextGraph,
    selectedNodeId: selection.highlightedNodeIds[0] || state.selectedNodeId,
    runtime: buildPersistedRuntimeSelection({
      runtime: state.runtime,
      ...selection
    })
  };
}
