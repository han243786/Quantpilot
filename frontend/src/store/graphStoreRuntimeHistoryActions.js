import {
  clearBacktestCompareSelectionState,
  replaceBacktestCompareSelectionState,
  toggleBacktestCompareSelectionState
} from "./graphStoreRuntimeHistoryCompareSelection";
import {
  discardBacktestRecordFlow,
  discardExperimentRecordFlow,
  discardRunRecordFlow,
  loadBacktestDetailFlow,
  loadExperimentDetailFlow,
  loadRunDetailFlow,
  refreshExperimentHistoryFlow,
  refreshBacktestHistoryFlow,
  refreshRunHistoryFlow,
  saveBacktestRecordFlow,
  saveExperimentRecordFlow,
  saveRunRecordFlow,
  warmRuntimeSidebarDataFlow
} from "./graphStoreRuntimeHistoryFlow";

export function createGraphStoreRuntimeHistoryActions(set, get) {
  return {
    async warmRuntimeSidebarData() {
      return warmRuntimeSidebarDataFlow(get);
    },

    toggleBacktestCompareSelection(backtestId) {
      set((state) => toggleBacktestCompareSelectionState(state, backtestId));
    },

    clearBacktestCompareSelection() {
      set((state) => clearBacktestCompareSelectionState(state));
    },

    replaceBacktestCompareSelection(backtestIds) {
      set((state) => replaceBacktestCompareSelectionState(state, backtestIds));
    },

    async refreshRunHistory() {
      return refreshRunHistoryFlow(set);
    },

    async refreshBacktestHistory() {
      return refreshBacktestHistoryFlow(set);
    },

    async refreshExperimentHistory() {
      return refreshExperimentHistoryFlow(set);
    },

    async loadRunDetail(runId) {
      return loadRunDetailFlow(set, get, runId);
    },

    async loadBacktestDetail(backtestId) {
      return loadBacktestDetailFlow(set, get, backtestId);
    },

    async loadExperimentDetail(experimentId) {
      return loadExperimentDetailFlow(set, get, experimentId);
    },

    async saveCurrentRuntimeArtifact() {
      const runtime = get().runtime;
      if (runtime.runKind === "backtest" && (runtime.selectedBacktestId || runtime.runId)) {
        return saveBacktestRecordFlow(set, get, runtime.selectedBacktestId || runtime.runId);
      }
      if (runtime.runKind === "simulation" && runtime.runId) {
        return saveRunRecordFlow(set, get, runtime.runId);
      }
      if (runtime.selectedExperimentId) {
        return saveExperimentRecordFlow(set, get, runtime.selectedExperimentId);
      }
      return null;
    },

    async discardCurrentRuntimeArtifact() {
      const runtime = get().runtime;
      if (runtime.artifactPersistenceStatus !== "transient") {
        return null;
      }
      if (runtime.runKind === "backtest" && (runtime.selectedBacktestId || runtime.runId)) {
        return discardBacktestRecordFlow(set, get, runtime.selectedBacktestId || runtime.runId);
      }
      if (runtime.runKind === "simulation" && runtime.runId) {
        return discardRunRecordFlow(set, get, runtime.runId);
      }
      if (runtime.selectedExperimentId) {
        return discardExperimentRecordFlow(set, get, runtime.selectedExperimentId);
      }
      return null;
    }
  };
}
