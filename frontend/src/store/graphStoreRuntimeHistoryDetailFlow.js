import {
  fetchBacktestDetail,
  fetchExperimentDetail,
  fetchRunDetail,
  fetchRuntimeMutations
} from "./graphStoreRuntimeHistoryApi";
import { resolveGraphForDetail, saveGraphToStorage } from "./graphStoreHelpers";
import { normalizeExperimentDetail } from "./graphStoreRuntimeHelpers";
import { buildRuntimeHistoryFailureMessage } from "./graphStoreRuntimeHistoryFailure";
import {
  projectBacktestDetailGraph,
  projectRunDetailGraph
} from "./graphStoreRuntimeHistoryProjection";
import {
  buildBacktestDetailSelectionState,
  buildExperimentDetailErrorState,
  buildExperimentDetailLoadingState,
  buildExperimentDetailReadyState,
  buildRunDetailSelectionState,
  buildRuntimeHistoryErrorState
} from "./graphStoreRuntimeHistoryState";

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
