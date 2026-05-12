import { buildPersistedRuntimeSelectionState } from "./graphStoreRuntimeSelectionState";

function sanitizeCompareSelection(backtestIds) {
  return Array.isArray(backtestIds)
    ? [...new Set(backtestIds.filter(Boolean))].slice(0, 2)
    : [];
}

// v1.0.5: 策略作用域 — 用 state.graph.metadata.graph_id 作为 key
function _strategyId(state) {
  return state.graph?.metadata?.graph_id || "_global";
}

export function getCompareSelection(state) {
  const raw = state.runtime.backtestCompareSelection;
  if (!raw) return [];
  if (Array.isArray(raw)) return raw;
  return raw[_strategyId(state)] || [];
}

export function toggleBacktestCompareSelectionState(state, backtestId) {
  const existing = getCompareSelection(state);
  const next = existing.includes(backtestId)
    ? existing.filter((id) => id !== backtestId)
    : existing.length >= 2
      ? existing
      : [...existing, backtestId];
  const raw = state.runtime.backtestCompareSelection;
  const base = Array.isArray(raw) ? {} : { ...raw };
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: { ...base, [_strategyId(state)]: next }
    }
  };
}

export function clearBacktestCompareSelectionState(state) {
  const raw = state.runtime.backtestCompareSelection;
  const base = Array.isArray(raw) ? {} : { ...raw };
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: { ...base, [_strategyId(state)]: [] }
    }
  };
}

export function replaceBacktestCompareSelectionState(state, backtestIds) {
  const raw = state.runtime.backtestCompareSelection;
  const base = Array.isArray(raw) ? {} : { ...raw };
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: { ...base, [_strategyId(state)]: sanitizeCompareSelection(backtestIds) }
    }
  };
}

export function buildRunHistoryLoadingState(state) {
  return {
    runtime: {
      ...state.runtime,
      historyStatus: "loading"
    }
  };
}

export function buildRunHistoryReadyState(state, history) {
  return {
    runtime: {
      ...state.runtime,
      history,
      historyStatus: "ready"
    }
  };
}

export function buildRunHistoryErrorState(state, message) {
  return {
    runtime: {
      ...state.runtime,
      historyStatus: "error",
      backendError: message
    }
  };
}

export function buildBacktestHistoryLoadingState(state) {
  return {
    runtime: {
      ...state.runtime,
      backtestHistoryStatus: "loading"
    }
  };
}

export function buildBacktestHistoryReadyState(state, backtestHistory) {
  const current = getCompareSelection(state);
  const raw = state.runtime.backtestCompareSelection;
  const base = Array.isArray(raw) ? {} : { ...raw };
  return {
    runtime: {
      ...state.runtime,
      backtestHistory,
      backtestHistoryStatus: "ready",
      backtestCompareSelection: { ...base, [_strategyId(state)]: current.filter(
        (backtestId) => backtestHistory.some((item) => item.backtest_id === backtestId)
      )}
    }
  };
}

export function buildBacktestHistoryErrorState(state, message) {
  return {
    runtime: {
      ...state.runtime,
      backtestHistoryStatus: "error",
      backendError: message
    }
  };
}

export function buildRuntimeHistoryErrorState(state, message) {
  return {
    runtime: {
      ...state.runtime,
      backendError: message
    }
  };
}

export function buildExperimentHistoryLoadingState(state) {
  return {
    runtime: {
      ...state.runtime,
      experimentsStatus: "loading"
    }
  };
}

export function buildExperimentHistoryReadyState(state, experiments) {
  const selectedExperimentId = state.runtime.selectedExperimentId;
  const hasSelection =
    selectedExperimentId &&
    experiments.some((entry) => entry.experiment_id === selectedExperimentId);
  return {
    runtime: {
      ...state.runtime,
      experiments,
      experimentsStatus: "ready",
      selectedExperimentId: hasSelection ? selectedExperimentId : null,
      selectedExperiment: hasSelection ? state.runtime.selectedExperiment : null,
      selectedExperimentStatus: hasSelection ? state.runtime.selectedExperimentStatus : "idle"
    }
  };
}

export function buildExperimentHistoryErrorState(state, message) {
  return {
    runtime: {
      ...state.runtime,
      experimentsStatus: "error",
      backendError: message
    }
  };
}

export function buildExperimentDetailLoadingState(state, experimentId) {
  return {
    runtime: {
      ...state.runtime,
      selectedExperimentId: experimentId,
      selectedExperimentStatus: "loading"
    }
  };
}

export function buildExperimentDetailReadyState(state, detail) {
  return {
    runtime: {
      ...state.runtime,
      selectedExperimentId: detail.experiment_id,
      selectedExperiment: detail,
      selectedExperimentStatus: "ready",
      backendError: null
    }
  };
}

export function buildExperimentDetailErrorState(state, message) {
  return {
    runtime: {
      ...state.runtime,
      selectedExperimentStatus: "error",
      backendError: message
    }
  };
}

export function buildRunDetailSelectionState(
  state,
  nextGraph,
  detail,
  highlightedNodeIds,
  parameterMutations = []
) {
  return buildPersistedRuntimeSelectionState(state, nextGraph, {
    runId: detail.run_id,
    runKind: "simulation",
    account: detail.account,
    artifactPersistenceStatus: "saved",
    backtestArtifacts: null,
    diagnostics: detail.runtime_diagnostics || null,
    governance: detail.governance || null,
    events: detail.events,
    timeline: (detail.timeline || []).slice(0, 200),
    retainedKeyEventIndex: detail.retained_key_event_index || null,
    compactEvidence: detail.compact_evidence || null,
    parameterMutations,
    selectedHistoryRunId: detail.run_id,
    selectedBacktestId: null,
    highlightedNodeIds
  });
}

export function buildBacktestDetailSelectionState(
  state,
  nextGraph,
  detail,
  events,
  highlightedNodeIds
) {
  return buildPersistedRuntimeSelectionState(state, nextGraph, {
    runId: detail.backtest_id,
    runKind: "backtest",
    account: detail.account,
    artifactPersistenceStatus: "saved",
    backtestArtifacts: detail.backtest_artifacts,
    diagnostics: detail.runtime_diagnostics || null,
    governance: detail.governance || detail.backtest_artifacts?.manifest?.governance || null,
    events,
    timeline: (detail.timeline || []).slice(0, 200),
    retainedKeyEventIndex: detail.retained_key_event_index || null,
    compactEvidence: detail.compact_evidence || null,
    parameterMutations: [],
    selectedHistoryRunId: null,
    selectedBacktestId: detail.backtest_id,
    highlightedNodeIds
  });
}
