import {
  discardBacktestRecord,
  discardExperimentRecord,
  discardRunRecord,
  fetchBacktestDetail,
  fetchExperimentDetail,
  fetchRunDetail,
  fetchRuntimeMutations,
  saveBacktestRecord,
  saveExperimentRecord,
  saveRunRecord
} from "./graphStoreRuntimeHistoryApi";
import { resolveGraphForDetail, saveGraphToStorage } from "./graphStoreHelpers";
import { buildRuntimeResetState } from "./graphStoreRuntimeSessionState";
import { normalizeExperimentDetail } from "./graphStoreRuntimeHelpers";
import { buildRuntimeHistoryFailureMessage } from "./graphStoreRuntimeHistoryFailure";
import {
  projectBacktestDetailGraph,
  projectRunDetailGraph
} from "./graphStoreRuntimeHistoryProjection";
import {
  refreshBacktestHistoryFlow,
  refreshExperimentHistoryFlow,
  refreshRunHistoryFlow
} from "./graphStoreRuntimeHistoryRefreshFlow";
import {
  buildBacktestDetailSelectionState,
  buildExperimentDetailErrorState,
  buildExperimentDetailLoadingState,
  buildExperimentDetailReadyState,
  buildRunDetailSelectionState,
  buildRuntimeHistoryErrorState
} from "./graphStoreRuntimeHistoryState";

export { buildRuntimeHistoryFailureMessage } from "./graphStoreRuntimeHistoryFailure";
export {
  refreshBacktestHistoryFlow,
  refreshExperimentHistoryFlow,
  refreshRunHistoryFlow,
  warmRuntimeSidebarDataFlow
} from "./graphStoreRuntimeHistoryRefreshFlow";

export async function loadRunDetailFlow(set, get, runId) {
  set({ selectedRunStatus: "loading" });
  try {
    const detail = await fetchRunDetail(runId);
    const parameterMutations = await fetchRuntimeMutations({
      source_kind: "run",
      source_id: runId
    });
    const graph = await resolveGraphForDetail(detail.graph_id, get().graph, get().registry);
    const { nextGraph, highlightedNodeIds } = projectRunDetailGraph(graph, detail);
    saveGraphToStorage(nextGraph);
    set((state) =>
      buildRunDetailSelectionState(
        state,
        nextGraph,
        detail,
        highlightedNodeIds,
        parameterMutations
      )
    );
    set({ selectedRunStatus: "ready" });
    return detail;
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(state, buildRuntimeHistoryFailureMessage("run_detail", error))
    );
    set({ selectedRunStatus: "error" });
    return null;
  }
}

export async function loadBacktestDetailFlow(set, get, backtestId) {
  try {
    const detail = await fetchBacktestDetail(backtestId);
    const graph = await resolveGraphForDetail(detail.graph_id, get().graph, get().registry);
    const { nextGraph, events, highlightedNodeIds } = projectBacktestDetailGraph(graph, detail);
    saveGraphToStorage(nextGraph);
    set((state) =>
      buildBacktestDetailSelectionState(state, nextGraph, detail, events, highlightedNodeIds)
    );
    return detail;
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(
        state,
        buildRuntimeHistoryFailureMessage("backtest_detail", error)
      )
    );
    return null;
  }
}

export async function loadExperimentDetailFlow(set, _get, experimentId) {
  set((state) => buildExperimentDetailLoadingState(state, experimentId));
  try {
    const detail = normalizeExperimentDetail(await fetchExperimentDetail(experimentId));
    set((state) => buildExperimentDetailReadyState(state, detail));
    return detail;
  } catch (error) {
    set((state) =>
      buildExperimentDetailErrorState(
        state,
        buildRuntimeHistoryFailureMessage("experiment_detail", error)
      )
    );
    return null;
  }
}

export async function saveRunRecordFlow(set, get, runId) {
  try {
    await saveRunRecord(runId);
    await refreshRunHistoryFlow(set);
    return loadRunDetailFlow(set, get, runId);
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(state, buildRuntimeHistoryFailureMessage("run_save", error))
    );
    return null;
  }
}

export async function saveBacktestRecordFlow(set, get, backtestId) {
  try {
    await saveBacktestRecord(backtestId);
    await refreshBacktestHistoryFlow(set);
    return loadBacktestDetailFlow(set, get, backtestId);
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(
        state,
        buildRuntimeHistoryFailureMessage("backtest_save", error)
      )
    );
    return null;
  }
}

export async function saveExperimentRecordFlow(set, get, experimentId) {
  try {
    await saveExperimentRecord(experimentId);
    await refreshExperimentHistoryFlow(set);
    return loadExperimentDetailFlow(set, get, experimentId);
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(
        state,
        buildRuntimeHistoryFailureMessage("experiment_save", error)
      )
    );
    return null;
  }
}

export async function discardRunRecordFlow(set, _get, runId) {
  try {
    const response = await discardRunRecord(runId);
    set((state) => buildRuntimeResetState(state));
    return response;
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(state, buildRuntimeHistoryFailureMessage("run_discard", error))
    );
    return null;
  }
}

export async function discardBacktestRecordFlow(set, _get, backtestId) {
  try {
    const response = await discardBacktestRecord(backtestId);
    set((state) => buildRuntimeResetState(state));
    return response;
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(
        state,
        buildRuntimeHistoryFailureMessage("backtest_discard", error)
      )
    );
    return null;
  }
}

export async function discardExperimentRecordFlow(set, _get, experimentId) {
  try {
    const response = await discardExperimentRecord(experimentId);
    set((state) => ({
      runtime: {
        ...state.runtime,
        selectedExperimentId: null,
        selectedExperiment: null,
        selectedExperimentStatus: "idle",
        backendError: null
      }
    }));
    return response;
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(
        state,
        buildRuntimeHistoryFailureMessage("experiment_discard", error)
      )
    );
    return null;
  }
}
