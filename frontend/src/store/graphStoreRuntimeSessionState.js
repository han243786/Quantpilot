import {
  accountFromPortfolioPayload,
  applyEventsToGraphNodes,
  applyRuntimeEventToNode,
  collectHighlightedNodeIds,
  resetNodeRuntimeState,
  resolveBacktestEvents,
  updateRuntimeNode,
  withRuntimeBinding
} from "./graphStoreHelpers";
import { buildPersistedRuntimeSelection } from "./graphStoreRuntimeSelectionState";

export function resolveRuntimeTargets(result) {
  return (
    result.runtime_targets ||
    result.backend_compile?.runtime_targets || {
      source_to_node: {},
      runtime_node_id: null,
      execution_node_id: null
    }
  );
}

export function buildRuntimeConnectingState(state, runKind, message) {
  return {
    runtime: {
      ...state.runtime,
      runId: null,
      runKind,
      status: "connecting",
      connectionState: "connecting",
      backendError: null,
      events: [],
      timeline: [],
      retainedKeyEventIndex: null,
      compactEvidence: null,
      v4_memory_snapshot: null,
      output: null,
      v4_runtime_handoff: null,
      account: null,
      artifactPersistenceStatus: "idle",
      backtestArtifacts: null,
      diagnostics: null,
      governance: null,
      selectedHistoryRunId: null,
      selectedBacktestId: null,
      highlightedNodeIds: []
    },
    graph: {
      ...state.graph,
      nodes: updateRuntimeNode(state.graph.nodes, "running", message)
    }
  };
}

export function buildRuntimeBindingGraph(graph, runId, compileId) {
  return withRuntimeBinding(graph, {
    current_run_id: runId,
    last_compile_id: compileId
  });
}

export function applyRuntimeStreamState(state, runId, event) {
  const nextAccount =
    event.event_type === "PortfolioUpdated"
      ? accountFromPortfolioPayload(event.payload, state.runtime.account)
      : state.runtime.account;
  const nodes = state.graph.nodes
    .map((node) => {
      if (node.id === event.node_id) return applyRuntimeEventToNode(node, event);
      return node;
    })
    .map((node) =>
      node.type === "runtime"
        ? {
            ...node,
            runtime_state: {
              ...node.runtime_state,
              status: "running",
              last_event_type: event.event_type,
              last_event_time: event.event_time_ms,
              last_message: event.summary
            }
          }
        : node
    );

  return {
    graph: { ...state.graph, nodes },
    runtime: {
      ...state.runtime,
      runId,
      runKind: "simulation",
      status: "running",
      connectionState: "connected",
      account: nextAccount,
      backtestArtifacts: null,
      diagnostics: null,
      events: [event, ...state.runtime.events].slice(0, 200),
      backendError: null,
      selectedHistoryRunId: runId,
      selectedBacktestId: null,
      highlightedNodeIds: event.node_id
        ? [...new Set([...state.runtime.highlightedNodeIds, event.node_id])].slice(0, 50)
        : state.runtime.highlightedNodeIds
    }
  };
}

export function buildRuntimeAccountState(runtime, account) {
  return {
    ...runtime,
    account
  };
}

export function buildRuntimeCompletionState(runtime, runId = null) {
  const resolvedRunId = runtime.runId || runId;
  return {
    ...runtime,
    runId: resolvedRunId,
    selectedHistoryRunId: runtime.selectedHistoryRunId || resolvedRunId,
    status: "completed",
    connectionState: "connected",
    artifactPersistenceStatus: resolvedRunId ? "transient" : runtime.artifactPersistenceStatus,
    backendError: null
  };
}

export function buildRuntimeFailureState(runtime, message) {
  return {
    ...runtime,
    status: runtime.status === "completed" ? "completed" : "error",
    backendError: runtime.status === "completed" ? null : message
  };
}

export function buildBacktestCompletionState(state, graph, response, compileId) {
  const events = resolveBacktestEvents(response).slice(0, 200);
  const highlightedNodeIds = collectHighlightedNodeIds(events);
  const nextGraph = buildRuntimeBindingGraph(
    {
      ...graph,
      nodes: applyEventsToGraphNodes(graph.nodes, events)
    },
    null,
    response.compile_id || compileId
  );

  return {
    nextGraph,
    selectedNodeId: highlightedNodeIds[0] || state.selectedNodeId,
    runtime: buildPersistedRuntimeSelection({
      runtime: state.runtime,
      runId: response.backtest_id || `backtest_${response.compile_id}`,
      runKind: "backtest",
      account: response.account || null,
      artifactPersistenceStatus: "transient",
      backtestArtifacts: response.backtest_artifacts,
      diagnostics: response.runtime_diagnostics || null,
      governance: response.governance || response.backtest_artifacts?.manifest?.governance || null,
      events,
      timeline: (response.timeline || []).slice(0, 200),
      retainedKeyEventIndex: response.retained_key_event_index || null,
      compactEvidence: response.compact_evidence || null,
      selectedHistoryRunId: null,
      selectedBacktestId: response.backtest_id || null,
      highlightedNodeIds
    })
  };
}

export function buildRuntimeStoppedState(state, message) {
  return {
    runtimeController: null,
    runtime: {
      ...state.runtime,
      status: "stopped",
      connectionState: "disconnected"
    },
    graph: {
      ...state.graph,
      nodes: updateRuntimeNode(state.graph.nodes, "stopped", message)
    }
  };
}

export function buildRuntimeResetState(state) {
  return {
    runtimeController: null,
    runtime: {
      ...state.runtime,
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
      v4_memory_snapshot: null,
      output: null,
      v4_runtime_handoff: null,
      backendError: null,
      selectedHistoryRunId: null,
      selectedBacktestId: null,
      highlightedNodeIds: []
    },
    graph: buildRuntimeBindingGraph(
      {
        ...state.graph,
        nodes: state.graph.nodes.map(resetNodeRuntimeState)
      },
      null,
      state.graph.metadata?.runtime_binding?.last_compile_id || null
    ),
    quantScriptDraft:
      state.graph.metadata?.artifacts?.quantscript?.graph_source || ""
  };
}
