import { buildActionFailureMessage } from "../utils/actionFailure";
import {
  fetchBacktestDetail,
  fetchBacktestHistoryList,
  fetchExperimentDetail,
  fetchExperimentHistoryList,
  fetchRunDetail,
  fetchRunHistoryList
} from "./graphStoreRuntimeHistoryApi";
import { resolveGraphForDetail, saveGraphToStorage } from "./graphStoreHelpers";
import {
  normalizeExperimentDetail,
  normalizeExperimentList
} from "./graphStoreRuntimeHelpers";
import {
  projectBacktestDetailGraph,
  projectRunDetailGraph
} from "./graphStoreRuntimeHistoryProjection";
import {
  buildBacktestDetailSelectionState,
  buildBacktestHistoryErrorState,
  buildBacktestHistoryLoadingState,
  buildBacktestHistoryReadyState,
  buildExperimentDetailErrorState,
  buildExperimentDetailLoadingState,
  buildExperimentDetailReadyState,
  buildExperimentHistoryErrorState,
  buildExperimentHistoryLoadingState,
  buildExperimentHistoryReadyState,
  buildRunDetailSelectionState,
  buildRunHistoryErrorState,
  buildRunHistoryLoadingState,
  buildRunHistoryReadyState,
  buildRuntimeHistoryErrorState
} from "./graphStoreRuntimeHistoryState";

export function buildRuntimeHistoryFailureMessage(kind, error) {
  const fallbackMessages = {
    run_history: "加载运行历史失败。",
    backtest_history: "加载回测历史失败。",
    experiment_history: "加载实验历史失败。",
    run_detail: "加载运行详情失败。",
    backtest_detail: "加载回测详情失败。",
    experiment_detail: "加载实验详情失败。"
  };
  return buildActionFailureMessage(kind, error, fallbackMessages[kind]);
}

export async function warmRuntimeSidebarDataFlow(get) {
  const runtime = get().runtime;
  const runHistoryReady =
    runtime.historyStatus === "ready" &&
    Array.isArray(runtime.history) &&
    runtime.history.length > 0;
  const backtestHistoryReady =
    runtime.backtestHistoryStatus === "ready" &&
    Array.isArray(runtime.backtestHistory) &&
    runtime.backtestHistory.length > 0;
  const experimentsReady =
    runtime.experimentsStatus === "ready" &&
    Array.isArray(runtime.experiments) &&
    runtime.experiments.length > 0;

  const tasks = [];
  if (!runHistoryReady && runtime.historyStatus !== "loading") {
    tasks.push(get().refreshRunHistory());
  }
  if (!backtestHistoryReady && runtime.backtestHistoryStatus !== "loading") {
    tasks.push(get().refreshBacktestHistory());
  }
  if (!experimentsReady && runtime.experimentsStatus !== "loading") {
    tasks.push(get().refreshExperimentHistory());
  }
  if (tasks.length === 0) {
    return [];
  }
  return Promise.all(tasks);
}

export async function refreshRunHistoryFlow(set) {
  set((state) => buildRunHistoryLoadingState(state));

  try {
    const history = await fetchRunHistoryList();
    set((state) => buildRunHistoryReadyState(state, history));
    return history;
  } catch (error) {
    set((state) =>
      buildRunHistoryErrorState(state, buildRuntimeHistoryFailureMessage("run_history", error))
    );
    return [];
  }
}

export async function refreshBacktestHistoryFlow(set) {
  set((state) => buildBacktestHistoryLoadingState(state));

  try {
    const backtestHistory = await fetchBacktestHistoryList();
    set((state) => buildBacktestHistoryReadyState(state, backtestHistory));
    return backtestHistory;
  } catch (error) {
    set((state) =>
      buildBacktestHistoryErrorState(
        state,
        buildRuntimeHistoryFailureMessage("backtest_history", error)
      )
    );
    return [];
  }
}

export async function refreshExperimentHistoryFlow(set) {
  set((state) => buildExperimentHistoryLoadingState(state));

  try {
    const experiments = normalizeExperimentList(await fetchExperimentHistoryList());
    set((state) => buildExperimentHistoryReadyState(state, experiments));
    return experiments;
  } catch (error) {
    set((state) =>
      buildExperimentHistoryErrorState(
        state,
        buildRuntimeHistoryFailureMessage("experiment_history", error)
      )
    );
    return [];
  }
}

export async function loadRunDetailFlow(set, get, runId) {
  try {
    const detail = await fetchRunDetail(runId);
    const graph = await resolveGraphForDetail(detail.graph_id, get().graph, get().registry);
    const { nextGraph, highlightedNodeIds } = projectRunDetailGraph(graph, detail);
    saveGraphToStorage(nextGraph);
    set((state) => buildRunDetailSelectionState(state, nextGraph, detail, highlightedNodeIds));
    return detail;
  } catch (error) {
    set((state) =>
      buildRuntimeHistoryErrorState(state, buildRuntimeHistoryFailureMessage("run_detail", error))
    );
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
