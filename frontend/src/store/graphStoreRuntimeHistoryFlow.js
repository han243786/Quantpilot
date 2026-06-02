import {
  discardBacktestRecord,
  discardExperimentRecord,
  discardRunRecord,
  saveBacktestRecord,
  saveExperimentRecord,
  saveRunRecord
} from "./graphStoreRuntimeHistoryApi";
import { buildRuntimeResetState } from "./graphStoreRuntimeSessionState";
import {
  loadBacktestDetailFlow,
  loadExperimentDetailFlow,
  loadRunDetailFlow
} from "./graphStoreRuntimeHistoryDetailFlow";
import { buildRuntimeHistoryFailureMessage } from "./graphStoreRuntimeHistoryFailure";
import {
  refreshBacktestHistoryFlow,
  refreshExperimentHistoryFlow,
  refreshRunHistoryFlow
} from "./graphStoreRuntimeHistoryRefreshFlow";
import { buildRuntimeHistoryErrorState } from "./graphStoreRuntimeHistoryState";

export { buildRuntimeHistoryFailureMessage } from "./graphStoreRuntimeHistoryFailure";
export {
  loadBacktestDetailFlow,
  loadExperimentDetailFlow,
  loadRunDetailFlow
} from "./graphStoreRuntimeHistoryDetailFlow";
export {
  refreshBacktestHistoryFlow,
  refreshExperimentHistoryFlow,
  refreshRunHistoryFlow,
  warmRuntimeSidebarDataFlow
} from "./graphStoreRuntimeHistoryRefreshFlow";

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
