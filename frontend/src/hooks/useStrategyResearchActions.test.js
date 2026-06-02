import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "../store/graphStore";
import { useStrategyResearchActions } from "./useStrategyResearchActions";

function buildUiState() {
  return {
    setRunHistoryFilter: vi.fn(),
    setRunHistoryCompileFilter: vi.fn(),
    setRunHistoryFromTime: vi.fn(),
    setRunHistoryToTime: vi.fn(),
    setRunHistoryStatusFilter: vi.fn(),
    setRunHistorySortOrder: vi.fn(),
    setRunHistoryPage: vi.fn(),
    setRunHistoryPageSize: vi.fn(),
    setBacktestHistoryFilter: vi.fn(),
    setBacktestCompileFilter: vi.fn(),
    setBacktestDatasetFilter: vi.fn(),
    setBacktestParameterFilter: vi.fn(),
    setBacktestFromTime: vi.fn(),
    setBacktestToTime: vi.fn(),
    setBacktestPage: vi.fn(),
    setBacktestPageSize: vi.fn(),
    setEventNodeScope: vi.fn(),
    setEventTypeFilter: vi.fn(),
    setEventSearchTerm: vi.fn()
  };
}

describe("useStrategyResearchActions", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("converts run and backtest refresh outcomes into panel notices", async () => {
    const refreshRunHistory = vi.fn(async () => {
      useGraphStore.setState((state) => ({
        runtime: {
          ...state.runtime,
          historyStatus: "ready",
          backendError: ""
        }
      }));
    });
    const refreshBacktestHistory = vi.fn(async () => {
      useGraphStore.setState((state) => ({
        runtime: {
          ...state.runtime,
          backtestHistoryStatus: "error",
          backendError: "backend unavailable"
        }
      }));
    });
    const onNotice = vi.fn();

    act(() => {
      useGraphStore.setState((state) => ({
        refreshRunHistory,
        refreshBacktestHistory,
        runtime: {
          ...state.runtime,
          historyStatus: "loading",
          backtestHistoryStatus: "loading",
          backendError: ""
        }
      }));
    });

    const { result } = renderHook(() =>
      useStrategyResearchActions(buildUiState(), { onNotice })
    );

    await act(async () => {
      await result.current.handleRefreshRunHistory();
      await result.current.handleRefreshBacktestHistory();
    });

    expect(refreshRunHistory).toHaveBeenCalledTimes(1);
    expect(refreshBacktestHistory).toHaveBeenCalledTimes(1);
    expect(onNotice).toHaveBeenNthCalledWith(1, "success", "Run history refreshed.");
    expect(onNotice).toHaveBeenNthCalledWith(2, "error", "backend unavailable");
  });

  it("returns transient artifact save and discard payloads with success notices", async () => {
    const saveCurrentRuntimeArtifact = vi.fn(async () => ({ run_id: "run_saved_001" }));
    const discardCurrentRuntimeArtifact = vi.fn(async () => ({ discarded: true }));
    const onNotice = vi.fn();

    act(() => {
      useGraphStore.setState((state) => ({
        saveCurrentRuntimeArtifact,
        discardCurrentRuntimeArtifact,
        runtime: {
          ...state.runtime,
          backendError: ""
        }
      }));
    });

    const { result } = renderHook(() =>
      useStrategyResearchActions(buildUiState(), { onNotice })
    );

    let saved;
    let discarded;
    await act(async () => {
      saved = await result.current.handleSaveCurrentRuntimeArtifact();
      discarded = await result.current.handleDiscardCurrentRuntimeArtifact();
    });

    expect(saved).toEqual({ run_id: "run_saved_001" });
    expect(discarded).toEqual({ discarded: true });
    expect(onNotice).toHaveBeenNthCalledWith(1, "success", expect.any(String));
    expect(onNotice).toHaveBeenNthCalledWith(2, "success", expect.any(String));
  });

  it("exposes ui-state setters and store selection actions without rebinding them", () => {
    const uiState = buildUiState();
    const loadRunDetail = vi.fn();
    const loadBacktestDetail = vi.fn();
    const toggleBacktestCompareSelection = vi.fn();
    const clearBacktestCompareSelection = vi.fn();

    act(() => {
      useGraphStore.setState({
        loadRunDetail,
        loadBacktestDetail,
        toggleBacktestCompareSelection,
        clearBacktestCompareSelection
      });
    });

    const { result } = renderHook(() => useStrategyResearchActions(uiState));

    expect(result.current.setRunHistoryFilter).toBe(uiState.setRunHistoryFilter);
    expect(result.current.setBacktestDatasetFilter).toBe(uiState.setBacktestDatasetFilter);
    expect(result.current.setEventSearchTerm).toBe(uiState.setEventSearchTerm);
    expect(result.current.loadRunDetail).toBe(loadRunDetail);
    expect(result.current.loadBacktestDetail).toBe(loadBacktestDetail);
    expect(result.current.toggleBacktestCompareSelection).toBe(toggleBacktestCompareSelection);
    expect(result.current.clearBacktestCompareSelection).toBe(clearBacktestCompareSelection);
  });
});
