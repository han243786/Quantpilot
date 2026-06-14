import {
  fetchBacktestHistoryList,
  fetchExperimentHistoryList,
  fetchRunHistoryList
} from "./graphStoreRuntimeHistoryApi";
import { normalizeExperimentList } from "./graphStoreRuntimeHelpers";
import { buildRuntimeHistoryFailureMessage } from "./graphStoreRuntimeHistoryFailure";
import {
  buildBacktestHistoryErrorState,
  buildBacktestHistoryLoadingState,
  buildBacktestHistoryReadyState,
  buildExperimentHistoryErrorState,
  buildExperimentHistoryLoadingState,
  buildExperimentHistoryReadyState,
  buildRunHistoryErrorState,
  buildRunHistoryLoadingState,
  buildRunHistoryReadyState
} from "./graphStoreRuntimeHistoryState";

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
