import {
  clearBacktestCompareSelectionState,
  replaceBacktestCompareSelectionState,
  toggleBacktestCompareSelectionState
} from "./graphStoreRuntimeHistoryState";
import {
  loadBacktestDetailFlow,
  loadExperimentDetailFlow,
  loadRunDetailFlow,
  refreshExperimentHistoryFlow,
  refreshBacktestHistoryFlow,
  refreshRunHistoryFlow,
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
    }
  };
}
