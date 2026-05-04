import { useGraphStore } from "../store/graphStore";
import { buildActionFailureMessage } from "../utils/actionFailure";

export function useStrategyResearchActions(uiState, { onNotice } = {}) {
  const refreshRunHistory = useGraphStore((state) => state.refreshRunHistory);
  const refreshBacktestHistory = useGraphStore((state) => state.refreshBacktestHistory);
  const loadRunDetail = useGraphStore((state) => state.loadRunDetail);
  const loadBacktestDetail = useGraphStore((state) => state.loadBacktestDetail);
  const saveCurrentRuntimeArtifact = useGraphStore((state) => state.saveCurrentRuntimeArtifact);
  const discardCurrentRuntimeArtifact = useGraphStore(
    (state) => state.discardCurrentRuntimeArtifact
  );
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

  async function handleSaveCurrentRuntimeArtifact() {
    const saved = await saveCurrentRuntimeArtifact();
    const nextRuntime = useGraphStore.getState().runtime;
    if (!saved) {
      pushNotice(
        "error",
        nextRuntime.backendError ||
          buildActionFailureMessage(
            "runtime_artifact_save",
            "没有可保存的运行结果。",
            "先完成一次模拟或回测，再保存进入 storage。"
          )
      );
      return null;
    }
    pushNotice("success", "运行结果已保存。");
    return saved;
  }

  async function handleDiscardCurrentRuntimeArtifact() {
    const discarded = await discardCurrentRuntimeArtifact();
    const nextRuntime = useGraphStore.getState().runtime;
    if (!discarded) {
      pushNotice(
        "error",
        nextRuntime.backendError ||
          buildActionFailureMessage(
            "runtime_artifact_discard",
            "没有可丢弃的临时结果。",
            "先完成一条未保存的模拟或回测结果，再执行丢弃。"
          )
      );
      return null;
    }
    pushNotice("success", "临时结果已丢弃。");
    return discarded;
  }

  return {
    handleRefreshRunHistory,
    handleRefreshBacktestHistory,
    handleSaveCurrentRuntimeArtifact,
    handleDiscardCurrentRuntimeArtifact,
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
