import { useGraphStore } from "../store/graphStore";
import { buildActionFailureMessage } from "../utils/actionFailure";

export function useStrategyResearchActions(uiState, { onNotice } = {}) {
  const refreshRunHistory = useGraphStore((state) => state.refreshRunHistory);
  const refreshBacktestHistory = useGraphStore((state) => state.refreshBacktestHistory);
  const loadRunDetail = useGraphStore((state) => state.loadRunDetail);
  const loadBacktestDetail = useGraphStore((state) => state.loadBacktestDetail);
  const toggleBacktestCompareSelection = useGraphStore(
    (state) => state.toggleBacktestCompareSelection
  );
  const clearBacktestCompareSelection = useGraphStore(
    (state) => state.clearBacktestCompareSelection
  );

  function pushNotice(type, message) {
    if (typeof onNotice === "function") {
      onNotice(type, message);
    }
  }

  async function handleRefreshRunHistory() {
    await refreshRunHistory();
    const nextRuntime = useGraphStore.getState().runtime;
    if (nextRuntime.historyStatus === "error") {
      pushNotice(
        "error",
        nextRuntime.backendError ||
          buildActionFailureMessage(
            "run_history",
            "Failed to load run history.",
            "Failed to load run history."
          )
      );
      return;
    }
    pushNotice("success", "Run history refreshed.");
  }

  async function handleRefreshBacktestHistory() {
    await refreshBacktestHistory();
    const nextRuntime = useGraphStore.getState().runtime;
    if (nextRuntime.backtestHistoryStatus === "error") {
      pushNotice(
        "error",
        nextRuntime.backendError ||
          buildActionFailureMessage(
            "backtest_history",
            "Failed to load backtest history.",
            "Failed to load backtest history."
          )
      );
      return;
    }
    pushNotice("success", "Backtest history refreshed.");
  }

  return {
    handleRefreshRunHistory,
    handleRefreshBacktestHistory,
    loadRunDetail,
    loadBacktestDetail,
    setRunHistoryFilter: uiState.setRunHistoryFilter,
    setRunHistoryCompileFilter: uiState.setRunHistoryCompileFilter,
    setRunHistoryFromTime: uiState.setRunHistoryFromTime,
    setRunHistoryToTime: uiState.setRunHistoryToTime,
    setRunHistoryStatusFilter: uiState.setRunHistoryStatusFilter,
    setRunHistorySortOrder: uiState.setRunHistorySortOrder,
    setRunHistoryPage: uiState.setRunHistoryPage,
    setRunHistoryPageSize: uiState.setRunHistoryPageSize,
    setBacktestHistoryFilter: uiState.setBacktestHistoryFilter,
    setBacktestCompileFilter: uiState.setBacktestCompileFilter,
    setBacktestDatasetFilter: uiState.setBacktestDatasetFilter,
    setBacktestParameterFilter: uiState.setBacktestParameterFilter,
    setBacktestFromTime: uiState.setBacktestFromTime,
    setBacktestToTime: uiState.setBacktestToTime,
    setBacktestPage: uiState.setBacktestPage,
    setBacktestPageSize: uiState.setBacktestPageSize,
    toggleBacktestCompareSelection,
    clearBacktestCompareSelection,
    setEventNodeScope: uiState.setEventNodeScope,
    setEventTypeFilter: uiState.setEventTypeFilter,
    setEventSearchTerm: uiState.setEventSearchTerm
  };
}
