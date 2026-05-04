import { describe, expect, it } from "vitest";
import {
  filterBacktestHistoryRecords,
  filterRunHistoryRecords
} from "./strategyResearchSelectors";

describe("strategyResearchSelectors history filtering", () => {
  it("keeps the selected saved run visible while stale filters are still applied", () => {
    const history = [
      {
        run_id: "run_saved_001",
        graph_id: "graph_saved",
        compile_id: "compile_saved",
        created_at_ms: 1_700_000_000_000,
        status: "completed"
      },
      {
        run_id: "run_other_001",
        graph_id: "graph_other",
        compile_id: "compile_other",
        created_at_ms: 1_700_100_000_000,
        status: "completed"
      }
    ];

    const result = filterRunHistoryRecords(
      history,
      {
        runId: "run_saved_001",
        runKind: "simulation",
        selectedHistoryRunId: "run_saved_001",
        status: "completed"
      },
      {
        historyFilter: "graph_that_would_hide_the_saved_run",
        historyCompileFilter: "compile_that_would_hide_the_saved_run",
        historyStatusFilter: "error",
        historyFromTime: "2024-01-01T00:00",
        historyToTime: "2024-01-01T00:01",
        historySortOrder: "desc"
      }
    );

    expect(result.map((run) => run.run_id)).toEqual(["run_saved_001"]);
  });

  it("keeps the selected saved backtest visible while stale filters are still applied", () => {
    const history = [
      {
        backtest_id: "backtest_saved_001",
        graph_id: "graph_saved",
        compile_id: "compile_saved",
        created_at_ms: 1_700_000_000_000,
        filters: {
          dataset_labels: ["Binance:BTCUSDT:1m"],
          execution_assumptions_tag: {
            label: "fee=10 slip=5 lat=0",
            sources_label: "fee:backend slip:backend lat:backend"
          },
          started_at_ms: 1_700_000_000_000,
          ended_at_ms: 1_700_000_060_000
        }
      },
      {
        backtest_id: "backtest_other_001",
        graph_id: "graph_other",
        compile_id: "compile_other",
        created_at_ms: 1_700_100_000_000,
        filters: {
          dataset_labels: ["Binance:ETHUSDT:5m"],
          execution_assumptions_tag: {
            label: "fee=20 slip=8 lat=25",
            sources_label: "fee:backend slip:backend lat:backend"
          },
          started_at_ms: 1_700_100_000_000,
          ended_at_ms: 1_700_100_060_000
        }
      }
    ];

    const result = filterBacktestHistoryRecords(
      history,
      {
        runId: "backtest_saved_001",
        runKind: "backtest",
        selectedBacktestId: "backtest_saved_001"
      },
      {
        backtestHistoryFilter: "graph_that_would_hide_the_saved_backtest",
        backtestCompileFilter: "compile_that_would_hide_the_saved_backtest",
        backtestDatasetFilter: "SOLUSDT",
        backtestParameterFilter: "fee=99",
        backtestFromTime: "2024-01-01T00:00",
        backtestToTime: "2024-01-01T00:01"
      }
    );

    expect(result.map((item) => item.backtest_id)).toEqual(["backtest_saved_001"]);
  });
});
