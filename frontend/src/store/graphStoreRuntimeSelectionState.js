export function buildPersistedRuntimeSelection({
  runtime,
  runId,
  runKind,
  account,
  backtestArtifacts = null,
  diagnostics = null,
  events = [],
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
    backtestArtifacts,
    diagnostics,
    events,
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
