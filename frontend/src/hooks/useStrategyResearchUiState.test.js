import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useStrategyResearchUiState } from "./useStrategyResearchUiState";

describe("useStrategyResearchUiState", () => {
  it("initializes runtime filters from graph id and resets scoped state on graph changes", () => {
    const { result, rerender } = renderHook(
      ({ graphId }) => useStrategyResearchUiState(graphId),
      {
        initialProps: { graphId: "graph_alpha" }
      }
    );

    expect(result.current.runFilters).toMatchObject({
      historyFilter: "graph_alpha",
      historyStatusFilter: "all",
      historySortOrder: "desc",
      historyPage: 1,
      historyPageSize: 6
    });
    expect(result.current.backtestFilters).toMatchObject({
      backtestHistoryFilter: "graph_alpha",
      backtestPage: 1,
      backtestPageSize: 6
    });
    expect(result.current.eventFilters).toEqual({
      eventNodeScope: "auto",
      eventTypeFilter: "all",
      eventSearchTerm: ""
    });

    act(() => {
      result.current.setRunHistoryPage(4);
      result.current.setRunHistoryFilter("manual_run_graph");
      result.current.setRunHistoryCompileFilter("compile_001");
      result.current.setRunHistoryPageSize(12);
      result.current.setBacktestPage(3);
      result.current.setBacktestDatasetFilter("BTCUSDT");
      result.current.setBacktestParameterFilter("fee=10");
      result.current.setEventNodeScope("node_entry");
      result.current.setEventTypeFilter("ExecutionFilled");
      result.current.setEventSearchTerm("latency");
    });

    expect(result.current.runFilters).toMatchObject({
      historyFilter: "manual_run_graph",
      historyCompileFilter: "compile_001",
      historyPage: 1,
      historyPageSize: 12
    });
    expect(result.current.backtestFilters).toMatchObject({
      backtestHistoryFilter: "graph_alpha",
      backtestDatasetFilter: "BTCUSDT",
      backtestParameterFilter: "fee=10",
      backtestPage: 1
    });
    expect(result.current.eventFilters).toEqual({
      eventNodeScope: "node_entry",
      eventTypeFilter: "ExecutionFilled",
      eventSearchTerm: "latency"
    });

    rerender({ graphId: "graph_beta" });

    expect(result.current.runFilters).toMatchObject({
      historyFilter: "graph_beta",
      historyCompileFilter: "compile_001",
      historyPage: 1,
      historyPageSize: 12
    });
    expect(result.current.backtestFilters).toMatchObject({
      backtestHistoryFilter: "graph_beta",
      backtestDatasetFilter: "BTCUSDT",
      backtestParameterFilter: "fee=10",
      backtestPage: 1
    });
    expect(result.current.eventFilters).toEqual({
      eventNodeScope: "auto",
      eventTypeFilter: "all",
      eventSearchTerm: ""
    });
  });
});
