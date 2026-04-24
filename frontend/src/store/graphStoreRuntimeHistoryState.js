import { buildPersistedRuntimeSelectionState } from "./graphStoreRuntimeSelectionState";

function sanitizeCompareSelection(backtestIds) {
  return Array.isArray(backtestIds)
    ? [...new Set(backtestIds.filter(Boolean))].slice(0, 2)
    : [];
}

export function toggleBacktestCompareSelectionState(state, backtestId) {
  const existing = state.runtime.backtestCompareSelection || [];
  const next = existing.includes(backtestId)
    ? existing.filter((id) => id !== backtestId)
    : existing.length >= 2
      ? existing
      : [...existing, backtestId];
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: next
    }
  };
}

export function clearBacktestCompareSelectionState(state) {
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: []
    }
  };
}

export function replaceBacktestCompareSelectionState(state, backtestIds) {
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: sanitizeCompareSelection(backtestIds)
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
      backendError: state.runtime.backendError || message
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
  return {
    runtime: {
      ...state.runtime,
      backtestHistory,
      backtestHistoryStatus: "ready",
      backtestCompareSelection: (state.runtime.backtestCompareSelection || []).filter(
        (backtestId) => backtestHistory.some((item) => item.backtest_id === backtestId)
      )
    }
  };
}

export function buildBacktestHistoryErrorState(state, message) {
  return {
    runtime: {
      ...state.runtime,
      backtestHistoryStatus: "error",
      backendError: state.runtime.backendError || message
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
      backendError: state.runtime.backendError || message
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
      backendError: state.runtime.backendError || message
    }
  };
}

export function buildRunDetailSelectionState(
  state,
  nextGraph,
  detail,
  highlightedNodeIds
) {
  return buildPersistedRuntimeSelectionState(state, nextGraph, {
    runId: detail.run_id,
    runKind: "simulation",
    account: detail.account,
    backtestArtifacts: null,
    diagnostics: detail.runtime_diagnostics || null,
    events: detail.events,
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
    backtestArtifacts: detail.backtest_artifacts,
    diagnostics: detail.runtime_diagnostics || null,
    events,
    selectedHistoryRunId: null,
    selectedBacktestId: detail.backtest_id,
    highlightedNodeIds
  });
}
